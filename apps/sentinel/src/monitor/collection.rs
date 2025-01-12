use device_query::{DeviceQuery, DeviceState, Keycode};
use slog::{info, Logger};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{self, Duration};

#[derive(Debug, Default)]
pub struct DataFrame {
    pub keys: Vec<Keycode>,
    pub mouse_coords: (i32, i32),
}

pub struct Collection {
    data: Arc<Mutex<DataFrame>>,
    logger: Logger,
    collection_frequency_ms: u64,
}

impl Collection {
    pub fn new(logger: Logger, collection_frequency_ms: u64) -> Self {
        Self {
            data: Arc::new(Mutex::new(DataFrame::default())),
            logger,
            collection_frequency_ms,
        }
    }

    pub async fn start(&self) {
        info!(self.logger, "Starting collection");
        let data_clone = Arc::clone(&self.data);
        let collection_frequency_ms = self.collection_frequency_ms;

        // Spawn a task for keyboard monitoring
        let data_clone_kb = Arc::clone(&data_clone);
        let logger_clone_kb = self.logger.clone();
        tokio::spawn(async move {
            let device_state = DeviceState::new();
            loop {
                let keys = device_state.get_keys();
                {
                    let mut data = data_clone_kb.lock().await;
                    data.keys = keys.clone();
                }
                info!(logger_clone_kb, "Captured keys: {:?}", keys);
                time::sleep(Duration::from_millis(collection_frequency_ms)).await;
            }
        });

        // Spawn a task for mouse monitoring
        let data_clone_mouse = Arc::clone(&data_clone);
        let logger_clone_mouse = self.logger.clone();
        tokio::spawn(async move {
            let device_state = DeviceState::new();
            loop {
                let mouse = device_state.get_mouse();
                {
                    let mut data = data_clone_mouse.lock().await;
                    data.mouse_coords = mouse.coords;
                }
                info!(logger_clone_mouse, "Captured mouse coords: {:?}", mouse.coords);
                time::sleep(Duration::from_millis(collection_frequency_ms)).await;
            }
        });

        // Log the data every second
        let data_clone_log = Arc::clone(&data_clone);
        let logger_clone_log = self.logger.clone();
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(1));
            loop {
                interval.tick().await;
                let data = data_clone_log.lock().await;
                info!(logger_clone_log, "DataFrame: {:?}", *data);
            }
        });
    }
}
