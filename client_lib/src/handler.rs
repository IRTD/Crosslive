use super::*;

#[async_trait::async_trait]
pub trait EventHandler {
    async fn incoming_clipboard(&mut self, _: &mut MessageStream) -> anyhow::Result<()> {
        Ok(())
    }
}

pub struct DefaultEventHandler;
#[async_trait::async_trait]
impl EventHandler for DefaultEventHandler {}
