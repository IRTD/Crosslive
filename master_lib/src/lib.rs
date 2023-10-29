pub mod handler;

use crate::handler::*;
use cross_messages::*;

use std::sync::Arc;
pub use tokio;
pub(crate) use tokio::{
    net::ToSocketAddrs,
    sync::{broadcast, RwLock},
};

pub type Register = Arc<RwLock<Vec<ID>>>;

pub struct MasterServer<T>
where
    T: MessageHandler,
{
    listener: MessageListener,
    register: Register,
    sender: broadcast::Sender<Message>,
    handler: T,
}

impl<T> MasterServer<T>
where
    T: MessageHandler + 'static,
{
    pub async fn new(addr: impl ToSocketAddrs, handler: T) -> std::io::Result<Self> {
        Ok(MasterServer {
            listener: MessageListener::bind(addr).await?,
            register: Register::default(),
            sender: broadcast::channel(12).0,
            handler,
        })
    }

    pub fn set_listener(&mut self, listener: MessageListener) {
        self.listener = listener;
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        loop {
            let msg_stream = self.listener.accept().await?;
            let cloned_reg = self.register.clone();
            let cloned_board = self.sender.clone();
            let stream_handler = StreamHandler {
                stream: msg_stream,
                register: cloned_reg,
                broadcast: cloned_board,
                handler: self.handler.clone(),
                id: ID::Unregistered,
            };

            tokio::spawn(async move { stream_handler.handle().await });
        }
    }
}

pub struct StreamHandler<T>
where
    T: MessageHandler,
{
    stream: MessageStream,
    register: Register,
    broadcast: broadcast::Sender<Message>,
    handler: T,
    id: ID,
}

impl<T> StreamHandler<T>
where
    T: MessageHandler,
{
    pub async fn handle(mut self) -> anyhow::Result<()> {
        let mut broad_recv = self.broadcast.subscribe();
        loop {
            tokio::select! {
                res = self.stream.recv() => {

                    let msg = res?;

                    match msg.header.target {
                        ID::Master => {
                            let ctx = Context {
                                message: msg,
                                register: &self.register,
                                broadcast: &mut self.broadcast,
                                id_ref: &mut self.id,
                            };

                            self.handler.handle(ctx).await?;
                        }

                        ID::Unregistered => continue,

                        _ => {
                            let _ = self.broadcast.send(msg)?;
                        }
                    }
                }

                res = broad_recv.recv() => {
                    let msg = res?;

                    if msg.header.target != self.id {
                        continue
                    }

                    self.stream.send(msg).await?;
                }
            }
        }
    }
}
