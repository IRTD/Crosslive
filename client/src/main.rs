mod client_config;
mod features;

use client_lib::*;
use cross_messages::*;

use features::AsyncClipboard;

#[tokio::main]
async fn main() {
    let config = client_config::ClientConfig::get().unwrap();
    let (mut client, handle) = CrossClient::new(config.master_addr())
        .await
        .unwrap()
        .register()
        .await
        .unwrap();

    tokio::spawn(async move { client.run().await.unwrap() });

    let mut client = Client::<copypasta::ClipboardContext>::new(handle)
        .await
        .unwrap();

    client.start().await.expect("yeah here");
}

struct Client<T>
where
    T: AsyncClipboard,
{
    handle: CrossHandle,
    clipboard: T,
    old_clipboard_content: String,
    other_devices: Vec<ID>,
}

impl<T> Client<T>
where
    T: AsyncClipboard,
{
    async fn new(mut handle: CrossHandle) -> anyhow::Result<Self> {
        handle
            .send(ID::Master, MessageKind::GetRegDevices, String::new())
            .await?;

        let other_devices_msg = handle.recv().await.unwrap();
        let other_devices: Vec<ID> = serde_json::from_str(&other_devices_msg.body)?;
        let clipboard = AsyncClipboard::new().await?;

        Ok(Client {
            handle,
            clipboard,
            old_clipboard_content: String::new(),
            other_devices,
        })
    }

    async fn start(&mut self) -> anyhow::Result<()> {
        loop {
            tokio::select! {
                res = self.handle.recv() => {
                    self.handle_message(res.unwrap()).await?;
                },

                res = self.clipboard.get() => {
                    let s = res?;

                    if s == self.old_clipboard_content {
                        continue;
                    }

                    for other in &self.other_devices {
                        self.handle.send(other.clone(), MessageKind::Clipboard, s.clone()).await?;
                    }
                }
            }
        }
    }

    async fn handle_message(&mut self, msg: Message) -> anyhow::Result<()> {
        match msg.header.kind {
            MessageKind::Clipboard => self.clipboard.set(msg.body).await?,
            _ => {}
        }

        Ok(())
    }
}
