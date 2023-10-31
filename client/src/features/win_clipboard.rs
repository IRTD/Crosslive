use super::AsyncClipboard;

use clipboard_win::{self, formats};
use std::time::Duration;

pub struct WindowsClipboardWrapper;

#[async_trait::async_trait]
impl AsyncClipboard for WindowsClipboardWrapper {
    async fn new() -> anyhow::Result<Self> {
        Ok(WindowsClipboardWrapper)
    }

    async fn get_new(&mut self) -> anyhow::Result<String> {
        let old: String = clipboard_win::get_clipboard(formats::Unicode).unwrap();
        loop {
            let current: String = clipboard_win::get_clipboard(formats::Unicode).unwrap();
            if current == old {
                tokio::time::sleep(Duration::from_millis(500)).await;
                continue;
            }

            return Ok(current)
        }
    }

    async fn set(&mut self, new: String) -> anyhow::Result<()> {
        clipboard_win::set_clipboard(formats::Unicode, new).unwrap();
        Ok(())
    }
}