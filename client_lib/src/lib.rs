use cross_messages::*;
use std::io::ErrorKind;
use tokio::{net::ToSocketAddrs, sync::mpsc};

pub struct CrossClient {
    master_stream: MessageStream,
}

impl CrossClient {
    pub async fn new(addr: impl ToSocketAddrs) -> std::io::Result<Self> {
        Ok(CrossClient {
            master_stream: MessageStream::connect(addr).await?,
        })
    }

    pub async fn register(mut self) -> anyhow::Result<(RegisteredClient, CrossHandle)> {
        log::info!("Attempting to Register to Master Server");
        let reg_msg = Message::register();
        self.master_stream.send(reg_msg).await?;
        let repl = self.master_stream.recv().await?;
        let registered_id = ID::from_register_reply(repl)?;

        log::info!("Registered with {:?}", registered_id);

        let (reg_tx, rx) = mpsc::channel::<Message>(16);
        let (tx, reg_rx) = mpsc::channel::<Message>(16);

        let client_handle = CrossHandle {
            tx,
            rx,
            registered_id,
        };

        let reg_client = RegisteredClient {
            master_stream: self.master_stream,
            tx: reg_tx,
            rx: reg_rx,
        };

        Ok((reg_client, client_handle))
    }
}

pub struct CrossHandle {
    tx: mpsc::Sender<Message>,
    rx: mpsc::Receiver<Message>,
    pub registered_id: ID,
}

impl CrossHandle {
    pub async fn send(&self, to: ID, kind: MessageKind, body: String) -> anyhow::Result<()> {
        let header = Header { kind, target: to };
        let tail = Tail {
            from: self.registered_id.clone(),
        };
        let message = Message { header, body, tail };

        log::info!("Sending {:#?}", message);

        self.tx.send(message).await?;
        Ok(())
    }

    pub fn blocking_send(&self, to: ID, kind: MessageKind, body: String) -> anyhow::Result<()> {
        let header = Header { kind, target: to };
        let tail = Tail {
            from: self.registered_id.clone(),
        };
        let message = Message { header, body, tail };

        log::info!("Sending {:#?}", message);

        self.tx.blocking_send(message)?;
        Ok(())
    }

    pub async fn recv(&mut self) -> Option<Message> {
        self.rx.recv().await
    }
}

pub struct RegisteredClient {
    master_stream: MessageStream,
    tx: mpsc::Sender<Message>,
    rx: mpsc::Receiver<Message>,
}

impl RegisteredClient {
    pub async fn run(&mut self) -> anyhow::Result<()> {
        loop {
            tokio::select! {
                res = self.master_stream.recv() => {
                    let msg: Message = match res {
                        Ok(m) => m,
                        Err(ref e) if e.kind() == ErrorKind::UnexpectedEof => return Ok(()),
                        Err(e) => return Err(e.into())
                    };
                    log::info!("New from Master {:#?}", msg);

                    if msg.header.kind == MessageKind::Close {
                        self.tx.send(msg).await?;
                        return Ok(());
                    }

                    self.tx.send(msg).await?;
                },

                res = self.rx.recv() => {
                    self.master_stream.send(res.unwrap()).await?;
                }
            }
        }
    }
}
