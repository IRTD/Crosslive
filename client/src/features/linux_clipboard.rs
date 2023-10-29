use copypasta::{ClipboardContext, ClipboardProvider};

use super::AsyncClipboard;

#[async_trait::async_trait]
impl AsyncClipboard for copypasta::ClipboardContext {
    async fn new() -> anyhow::Result<Self> {
        Ok(ClipboardContext::new().unwrap())
    }

    async fn get(&mut self) -> anyhow::Result<String> {
        Ok(self.get_contents().unwrap())
    }

    async fn set(&mut self, new: String) -> anyhow::Result<()> {
        self.set_contents(new).unwrap();
        Ok(())
    }
}
