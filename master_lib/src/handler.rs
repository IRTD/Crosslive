use crate::*;
use cross_messages::*;

#[async_trait::async_trait]
pub trait MessageHandler: Clone + Send {
    async fn handle(&mut self, ctx: Context<'_>) -> anyhow::Result<()>;
}

pub struct Context<'a> {
    pub message: Message,
    pub register: &'a Register,
    pub broadcast: &'a mut broadcast::Sender<Message>,
    pub id_ref: &'a mut ID,
}

#[derive(Clone)]
pub struct DefaultMessageHandler;

#[async_trait::async_trait]
impl MessageHandler for DefaultMessageHandler {
    async fn handle(&mut self, mut ctx: Context<'_>) -> anyhow::Result<()> {
        match ctx.message.header.kind {
            MessageKind::Register => {
                log::info!("Registering new Device");
                default_register(&mut ctx).await?;
            }

            MessageKind::GetRegDevices => {
                default_get_reg_devices(&mut ctx).await?;
            }

            MessageKind::Close => {
                log::info!("Closing Connection to {:#?}", ctx.id_ref);
                default_close(&mut ctx).await?;
                return Err(anyhow::anyhow!("Close connection"));
            }

            _ => {}
        }

        Ok(())
    }
}

pub async fn default_register(ctx: &mut Context<'_>) -> anyhow::Result<()> {
    let new_id = ID::new_slave();
    *ctx.id_ref = new_id.clone();

    let mut write_reg = ctx.register.write().await;
    log::info!("Writing new ID into Register");
    write_reg.push(new_id.clone());
    drop(write_reg);

    let new_id_str = serde_json::to_string(&new_id)?;

    let header = Header {
        kind: MessageKind::Reply,
        target: new_id,
    };
    let tail = Tail { from: ID::Master };
    let msg = Message {
        header,
        body: new_id_str,
        tail,
    };

    ctx.broadcast.send(msg)?;
    inform_update_reg(ctx, MessageKind::NewRegDevice).await?;
    Ok(())
}

pub async fn default_get_reg_devices(ctx: &mut Context<'_>) -> anyhow::Result<()> {
    let read_reg = ctx.register.read().await;
    let list = serde_json::to_string(
        &*read_reg
            .iter()
            .filter(|item| item != &ctx.id_ref)
            .map(|id| id.clone())
            .collect::<Vec<ID>>(),
    )?;
    drop(read_reg);

    let header = Header {
        kind: MessageKind::Reply,
        target: ctx.message.tail.from.clone(),
    };

    let tail = Tail { from: ID::Master };

    let msg = Message {
        header,
        body: list,
        tail,
    };

    ctx.broadcast.send(msg)?;

    Ok(())
}

pub async fn default_close(ctx: &mut Context<'_>) -> anyhow::Result<()> {
    let mut write_reg = ctx.register.write().await;
    let mut i = None;
    for (index, id) in write_reg.iter().enumerate() {
        if id == ctx.id_ref {
            i = Some(index);
            break;
        }
    }

    log::info!("Removing {:#?} from Register", ctx.id_ref);
    match i {
        Some(i) => write_reg.remove(i),
        None => {
            log::warn!("ID '{:?}' Not Found in Register", ctx.id_ref);
            return Err(anyhow::anyhow!("ID Not In Register"));
        }
    };

    drop(write_reg);
    inform_update_reg(ctx, MessageKind::ClosedRegDevice).await?;

    Ok(())
}

pub async fn inform_update_reg(ctx: &mut Context<'_>, kind: MessageKind) -> anyhow::Result<()> {
    log::info!("Updating other registered Devices");
    let reg = ctx.register.read().await;
    for id in reg.iter().filter(|i| i != &ctx.id_ref) {
        let header = Header {
            target: id.clone(),
            kind,
        };
        let tail = Tail { from: ID::Master };
        let body = serde_json::to_string(&ctx.id_ref)?;
        let msg = Message { header, tail, body };

        ctx.broadcast.send(msg)?;
    }

    Ok(())
}
