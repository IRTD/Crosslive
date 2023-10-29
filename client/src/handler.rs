#[async_trait::async_trait]
pub trait MessageHandler: Default {
    async fn incoming_clipboard(&mut self, _ctx: Context<'_>) -> anyhow::Result<()> {
        Ok(())
    }
}

#[derive(Default)]
pub struct DefaultMessageHandler;

#[async_trait::async_trait]
impl MessageHandler for DefaultMessageHandler {}
