mod models;

use chrono::Utc;
use models::{Config, Data, Uptime};
use prowl::Notification;
use tokio::time::{sleep, Duration};

fn duration_string(dur: chrono::Duration) -> String {
    if dur.num_days() > 0 {
        format!("{} days", dur.num_days())
    } else if dur.num_hours() > 0 {
        format!("{} hours", dur.num_hours())
    } else {
        format!("{} minutes", dur.num_minutes())
    }
}

async fn notify(config: &Config, uptime: &Uptime) {
    let hostname = match hostname::get() {
        Ok(x) => x.to_str().unwrap_or("Unknown").to_string(),
        Err(_) => "Unknown".to_string(),
    };
    let uptime_duration = uptime
        .last_heartbeat()
        .signed_duration_since(*uptime.start());
    let reboot_duration = Utc::now().signed_duration_since(*uptime.last_heartbeat());
    let reboot_time_string = duration_string(reboot_duration);
    let uptime_string = duration_string(uptime_duration);
    let event = format!("{hostname} restarted.");
    let description = format!("{hostname} was rebooted. Took {reboot_time_string} to come back up. Previous uptime was {uptime_string}.");
    let notification = Notification::new(
        config.prowl_api_keys().to_owned(),
        None,
        None,
        hostname,
        event,
        description,
    )
    .expect("Failed to create notification");
    match notification.add().await {
        Ok(_) => {}
        Err(e) => log::error!("Failed to add notification due to {e}"),
    };
}

// data with 1 event = 62 bytes

#[tokio::main]
async fn main() {
    env_logger::init();

    let config = Config::load(std::env::args().nth(1));
    let mut data = Data::load_or_default(&config);
    // TODO: background queue for the notifications incase of no internet.
    // create_new_uptime, and while len > 1, notify?
    if let Some(uptime) = data.first_uptime() {
        notify(&config, uptime).await;
        data.delete_first_uptime();
    }

    data.create_new_uptime();

    loop {
        data.beat();
        data.save(&config);
        sleep(Duration::from_secs(60)).await;
    }
}
