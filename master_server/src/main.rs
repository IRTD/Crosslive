mod master_config;

#[tokio::main]
async fn main() {
    let config = master_config::MasterConfig::get().unwrap();

    let mut server = master_lib::MasterServer::new(
        config.master_addr(),
        master_lib::handler::DefaultMessageHandler,
    )
    .await
    .unwrap();

    server.run().await.unwrap();
}
