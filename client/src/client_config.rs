use config::{File, FileFormat};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ClientConfig {
    pub master_addr: String,
    pub master_port: u16,
}

impl ClientConfig {
    pub fn get_from(path: &str) -> Result<Self, config::ConfigError> {
        config::Config::builder()
            .add_source(File::new(path, FileFormat::Toml))
            .build()?
            .try_deserialize()
    }

    pub fn master_addr(&self) -> String {
        format!("{}:{}", self.master_addr, self.master_port)
    }

    pub fn get() -> Result<Self, config::ConfigError> {
        if let Ok(v) = std::env::var("CROSSCONFIG") {
            return Self::get_from(&v);
        }

        let path = format!(
            "{}/crosslive_client.toml",
            dirs::config_local_dir()
                .unwrap()
                .as_path()
                .to_str()
                .unwrap()
        );

        Self::get_from(&path)
    }
}
