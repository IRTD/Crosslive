use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};

use super::*;

pub struct MessageListener {
    inner: TcpListener,
}

impl MessageListener {
    pub async fn bind(addr: impl ToSocketAddrs) -> std::io::Result<Self> {
        Ok(MessageListener {
            inner: TcpListener::bind(addr).await?,
        })
    }

    pub fn with(inner: TcpListener) -> Self {
        MessageListener { inner }
    }

    pub async fn accept(&self) -> std::io::Result<MessageStream> {
        Ok(MessageStream {
            inner: self.inner.accept().await?.0,
        })
    }
}

pub struct MessageStream {
    inner: TcpStream,
}

impl MessageStream {
    pub async fn connect(addr: impl ToSocketAddrs) -> std::io::Result<Self> {
        Ok(MessageStream {
            inner: TcpStream::connect(addr).await?,
        })
    }

    pub async fn send(&mut self, msg: Message) -> std::io::Result<usize> {
        self.inner.write(&serde_json::to_vec(&msg)?).await
    }

    pub async fn recv(&mut self) -> anyhow::Result<Message> {
        let mut buffer = [0; 2048];
        let len = self.inner.read(&mut buffer).await?;
        Ok(serde_json::from_slice(&buffer[..len])?)
    }
}
