mod client_config;
mod features;

use client_lib::*;
use cross_messages::*;

use features::AsyncClipboard;

#[cfg(target_os = "windows")]
use features::clipboard;

#[tokio::main]
async fn main() {
    crosslogging::init_fern_logger().unwrap();

    let config = client_config::ClientConfig::get().unwrap();
    let (mut client, handle) = CrossClient::new(config.master_addr())
        .await
        .unwrap_or_else(|e| {
            log::error!("Failed to connect to Master Server due to '{}'", e);
            std::process::exit(1);
        })
        .register()
        .await
        .unwrap_or_else(|e| {
            log::error!("Failed to register at Master Server due to '{}'", e);
            std::process::exit(2);
        });

    let thread_handle = tokio::spawn(async move { client.run().await.expect("Hello from thread") });

    let mut client = Client::new(handle).await.unwrap();

    if let Err(e) = client.start().await {
        log::error!("Interal Client error '{}'", e);
    }

    client
        .handle
        .send(ID::Master, MessageKind::Close, String::new())
        .await
        .unwrap_or_else(|e| {
            log::error!("Failed to properly Close due to '{}'", e);
            std::process::exit(3);
        });

    thread_handle.await.unwrap();
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
    async fn start(&mut self) -> anyhow::Result<()> {
        loop {
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {
                    return Ok(())
                },

                res = self.handle.recv() => {
                    match res {
                        Some(msg) => self.handle_message(msg).await?,
                        None => return Err(anyhow::anyhow!("None from handler.recv()"))
                    }
                },

                res = self.clipboard.get_new() => {
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
            MessageKind::NewRegDevice => {
                self.other_devices.push(serde_json::from_str(&msg.body)?);
            }
            MessageKind::ClosedRegDevice => {
                remove_on_match(&mut self.other_devices, &serde_json::from_str(&msg.body)?)
            }
            _ => {}
        }

        log::debug!("Peers:\n{:#?}", self.other_devices);

        Ok(())
    }
}

#[cfg(target_os = "linux")]
impl Client<copypasta::ClipboardContext> {
    async fn new(mut handle: CrossHandle) -> anyhow::Result<Self> {
        handle
            .send(ID::Master, MessageKind::GetRegDevices, String::new())
            .await?;

        let other_devices_msg = handle.recv().await.unwrap();
        let other_devices: Vec<ID> = serde_json::from_str(&other_devices_msg.body)?;
        let clipboard = AsyncClipboard::new().await?;

        log::debug!("Peers:\n{:#?}", other_devices);

        Ok(Client {
            handle,
            clipboard,
            old_clipboard_content: String::new(),
            other_devices,
        })
    }
}

#[cfg(target_os = "windows")]
impl Client<clipboard::WindowsClipboardWrapper> {
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
}

fn remove_on_match(v: &mut Vec<ID>, target: &ID) {
    for (index, item) in v.clone().iter().enumerate() {
        if item == target {
            v.remove(index);
        }
    }
}
