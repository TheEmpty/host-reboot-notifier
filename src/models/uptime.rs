use chrono::{serde::ts_seconds, DateTime, Utc};
use derive_getters::Getters;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Getters, Serialize)]
pub(crate) struct Uptime {
    #[serde(with = "ts_seconds")]
    start: DateTime<Utc>,
    #[serde(with = "ts_seconds")]
    last_heartbeat: DateTime<Utc>,
}

impl Uptime {
    pub(crate) fn beat(&mut self) {
        self.last_heartbeat = Utc::now();
    }

    pub(crate) fn new() -> Self {
        let now = Utc::now();
        Uptime {
            start: now,
            last_heartbeat: now,
        }
    }
}
