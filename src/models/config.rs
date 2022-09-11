use derive_getters::Getters;
use serde::Deserialize;
use std::{fs::File, io::BufReader};

#[derive(Deserialize, Getters)]
pub(crate) struct Config {
    data_file: String,
    prowl_api_keys: Vec<String>,
}

impl Config {
    pub(crate) fn load(filename: Option<String>) -> Self {
        let filename = match filename {
            Some(x) => {
                log::debug!("Using argument for config file: '{x}'.");
                x
            }
            None => {
                log::debug!("Using default config file path, ./config.json");
                "config.json".to_string()
            }
        };

        let config_file =
            File::open(&filename).unwrap_or_else(|_| panic!("Faild to find config {filename}"));
        let config_reader = BufReader::new(config_file);
        serde_json::from_reader(config_reader).expect("Error reading configuration.")
    }
}
