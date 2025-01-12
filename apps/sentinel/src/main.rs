mod config;
mod utils;
mod monitor;

use config::Settings;
use utils::logger::{init_root_logger, create_child_logger};
use monitor::collection::Collection;
use slog::info;
use tokio;

#[tokio::main]
async fn main() {
    let settings = Settings::new().expect("Failed to load configuration");
    let root_logger = init_root_logger();
    let app_logger = create_child_logger(&root_logger, "app");

    info!(app_logger, "Application started with settings: {:?}", settings);

    let collection = Collection::new(app_logger, settings.collection_frequency_ms);
    collection.start().await;

    // Wait for a termination signal
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for termination signal");
}
