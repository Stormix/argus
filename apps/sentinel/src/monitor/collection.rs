use chrono::Utc;
use device_query::{DeviceQuery, DeviceState, Keycode};
use slog::{Logger, debug, info};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{self, Duration};

use crate::utils::math::euclidean_distance;
use crate::utils::windows::{Application, get_open_applications};

#[derive(Debug, Default)]
pub struct DataFrame {
    pub keys: Vec<Keycode>,
    pub mouse_coords: (i32, i32),
    pub timestamp: u64,
    pub mouse_buttons: Vec<bool>,
    pub mouse_distance: i32,
    pub open_applications: Vec<Application>,
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

    async fn monitor_keyboard(
        data: Arc<Mutex<DataFrame>>,
        logger: Logger,
        collection_frequency_ms: u64,
    ) {
        let device_state = DeviceState::new();
        loop {
            let keys = device_state.get_keys();
            {
                let mut data = data.lock().await;
                data.keys = keys.clone();
                data.timestamp = Utc::now().timestamp_millis() as u64;
            }
            debug!(logger, "Captured keys: {:?}", keys);
            time::sleep(Duration::from_millis(collection_frequency_ms)).await;
        }
    }

    async fn monitor_mouse(
        data: Arc<Mutex<DataFrame>>,
        logger: Logger,
        collection_frequency_ms: u64,
    ) {
        let device_state = DeviceState::new();
        let mut last_mouse_coords = (0, 0);
        loop {
            let mouse = device_state.get_mouse();
            {
                let mut data = data.lock().await;
                data.mouse_coords = mouse.coords;
                data.mouse_buttons = mouse.button_pressed.clone().split_off(1);
                data.timestamp = Utc::now().timestamp_millis() as u64;
                data.mouse_distance = euclidean_distance(last_mouse_coords, mouse.coords);
                last_mouse_coords = mouse.coords;
            }
            debug!(logger, "Captured mouse coords: {:?}", mouse.coords);
            time::sleep(Duration::from_millis(collection_frequency_ms)).await;
        }
    }

    async fn monitor_windows(
        data: Arc<Mutex<DataFrame>>,
        logger: Logger,
        collection_frequency_ms: u64,
    ) {
        loop {
            let windows = get_open_applications();
            info!(logger, "Open windows: {:?}", windows);
            if let Ok(windows) = windows {
                {
                    let mut data = data.lock().await;
                    data.open_applications = windows;
                }
            }
            time::sleep(Duration::from_millis(collection_frequency_ms)).await;
        }
    }

    async fn log_data(data: Arc<Mutex<DataFrame>>, logger: Logger) {
        let mut interval = time::interval(Duration::from_secs(1));
        loop {
            interval.tick().await;
            let data = data.lock().await;
            info!(logger, "DataFrame: {:?}", *data);
        }
    }

    pub async fn start(&self) {
        info!(self.logger, "Starting collection");
        let data_clone = Arc::clone(&self.data);
        let collection_frequency_ms = self.collection_frequency_ms;

        // Spawn a task for keyboard monitoring
        tokio::spawn(Self::monitor_keyboard(
            Arc::clone(&data_clone),
            self.logger.clone(),
            collection_frequency_ms,
        ));

        // Spawn a task for mouse monitoring
        tokio::spawn(Self::monitor_mouse(
            Arc::clone(&data_clone),
            self.logger.clone(),
            collection_frequency_ms,
        ));

        // Spawn a task for window monitoring
        tokio::spawn(Self::monitor_windows(
            Arc::clone(&data_clone),
            self.logger.clone(),
            collection_frequency_ms,
        ));

        // Log the data every second
        tokio::spawn(Self::log_data(Arc::clone(&data_clone), self.logger.clone()));
    }
}
