mod models;

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
    let duration = uptime
        .last_heartbeat()
        .signed_duration_since(*uptime.start());
    let event = format!("Restarted after {}", duration_string(duration));
    let notification = Notification::new(
        config.prowl_api_keys().to_owned(),
        None,
        None,
        hostname,
        event,
        "The host was restarted and just came back online.".to_string(),
    )
    .expect("Failed to create notification");
    match notification.add().await {
        Ok(_) => {}
        Err(e) => log::error!("Failed to add notification due to {e}"),
    };
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let config = Config::load(std::env::args().nth(1));
    let mut data = Data::load_or_default(&config);
    // TODO: background queue for the notifications incase of no internet.
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
