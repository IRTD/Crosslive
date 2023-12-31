use config::{File, FileFormat};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct MasterConfig {
    pub host_ip: String,
    pub host_port: u16,
}

impl MasterConfig {
    pub fn master_addr(&self) -> String {
        format!("{}:{}", self.host_ip, self.host_port)
    }

    pub fn get() -> Result<Self, config::ConfigError> {
        if let Ok(v) = std::env::var("CROSSCONFIG") {
            return Self::get_from(&v);
        }

        let path = format!(
            "{}/master_config.toml",
            dirs::config_local_dir()
                .unwrap()
                .as_path()
                .to_str()
                .unwrap()
        );

        log::info!("No env path found, using default path {}", path);

        Self::get_from(&path)
    }

    pub fn get_from(path: &str) -> Result<Self, config::ConfigError> {
        log::info!("Attempting to load config from path: {}", path);
        config::Config::builder()
            .add_source(File::new(path, FileFormat::Toml))
            .build()?
            .try_deserialize()
    }
}
