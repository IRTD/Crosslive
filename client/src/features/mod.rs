#[cfg(target_os = "linux")]
pub mod linux_clipboard;
#[cfg(target_os = "windows")]
pub mod win_clipboard;

#[cfg(target_os = "linux")]
pub use linux_clipboard as clipboard;
#[cfg(target_os = "windows")]
pub use win_clipboard as clipboard;

#[async_trait::async_trait]
pub trait AsyncClipboard {
    async fn new() -> anyhow::Result<Self>
    where
        Self: Sized;

    async fn get_new(&mut self) -> anyhow::Result<String>;
    async fn set(&mut self, _: String) -> anyhow::Result<()>;
}
