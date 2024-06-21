use eroteme::setup_store;
use eroteme::Config;
use std::env;

#[tokio::main]
async fn main() -> Result<(), handle_errors::Error> {
    dotenv::dotenv().ok();

    let config = Config::new().expect("failed to set config");
    let store = setup_store(&config).await?;

    tracing::info!("Eroteme build ID {}", env!("EROTEME_VERSION"));
    eroteme::run(config, store).await;

    Ok(())
}
