use crate::models::{Config, Uptime};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub(crate) struct Data {
    uptimes: Vec<Uptime>,
}

impl Data {
    pub(crate) fn load_or_default(config: &Config) -> Self {
        match std::fs::read_to_string(config.data_file()) {
            Ok(val) => match serde_json::from_str(&val) {
                Ok(data) => data,
                Err(e) => {
                    log::error!("Failed to parse data, due to {e}");
                    Data { uptimes: vec![] }
                }
            },
            Err(e) => {
                log::error!("Failed to load data file, due to {e}");
                Data { uptimes: vec![] }
            }
        }
    }

    pub(crate) fn save(&self, config: &Config) {
        match serde_json::to_string(&self) {
            Ok(data) => {
                match std::fs::write(config.data_file(), data) {
                    Ok(_) => {}
                    Err(e) => log::error!("Failed to write data to file, {e}"),
                };
            }
            Err(e) => log::error!("Failed to convert data to string, {e}"),
        }
    }

    pub(crate) fn uptime_len(&self) -> usize {
        self.uptimes.len()
    }

    pub(crate) fn delete_first_uptime(&mut self) -> Uptime {
        self.uptimes.remove(0)
    }

    pub(crate) fn create_new_uptime(&mut self) {
        let uptime = Uptime::new();
        self.uptimes.push(uptime);
    }

    pub(crate) fn beat(&mut self) {
        self.uptimes.last_mut().expect("No uptimes").beat();
    }
}
