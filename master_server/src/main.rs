mod master_config;

#[tokio::main]
async fn main() {
    crosslogging::init_fern_logger().unwrap();

    let config = match master_config::MasterConfig::get() {
        Ok(c) => c,
        Err(e) => {
            log::error!("Failed to load Config due to {}", e);
            std::process::exit(1)
        }
    };

    log::info!("Loaded config");

    log::info!(
        "Creating new Server Instance with IpV4 Address {}",
        config.master_addr()
    );

    let mut server = match master_lib::MasterServer::new(
        config.master_addr(),
        master_lib::handler::DefaultMessageHandler,
    )
    .await
    {
        Ok(s) => s,
        Err(e) => {
            log::error!("Failed to create Server Instance due to {}", e);
            std::process::exit(2);
        }
    };

    log::info!("Starting Server Instance");
    match server.run().await {
        Ok(_) => log::info!("Server closed"),
        Err(e) => {
            log::info!("Internal Server error {}", e);
            std::process::exit(3);
        }
    }
}
