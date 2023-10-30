use std::time::Duration;

use copypasta::{ClipboardContext, ClipboardProvider};

use super::AsyncClipboard;

#[async_trait::async_trait]
impl AsyncClipboard for copypasta::ClipboardContext {
    async fn new() -> anyhow::Result<Self> {
        Ok(ClipboardContext::new().unwrap())
    }

    async fn get_new(&mut self) -> anyhow::Result<String> {
        let mut start_content = self.get_contents().unwrap();
        loop {
            let c = self.get_contents().unwrap();
            if c == start_content {
                tokio::time::sleep(Duration::from_millis(500)).await;
                continue;
            }

            return Ok(c);
        }
    }

    async fn set(&mut self, new: String) -> anyhow::Result<()> {
        self.set_contents(new).unwrap();
        Ok(())
    }
}
