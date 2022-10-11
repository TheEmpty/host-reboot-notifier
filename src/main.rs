mod models;

use chrono::Utc;
use models::{Config, Data, Uptime};
use prowl::Notification;
use prowl_queue::{ProwlQueue, ProwlQueueSender};
use tokio::time::{sleep, Duration};

fn hostname() -> String {
    match std::env::var("HOST_REBOOT_NOTIFIER_HOSTNAME") {
        Ok(val) => return val,
        Err(e) => log::trace!("Couldn't get HOST_REBOOT_NOTIFIER_HOSTNAME env variable. {e}"),
    };

    match hostname::get() {
        Ok(x) => return x.to_str().unwrap_or("Unknown").to_string(),
        Err(e) => {
            log::error!("Failed to get hostname: {e}");
        }
    };

    "Unknown".to_string()
}

fn singular_or_plural(singular_form: &str, num: i64) -> String {
    let pluralize = if num == 1 { "" } else { "s" };
    format!("{num} {singular_form}{pluralize}")
}

fn duration_string(dur: chrono::Duration) -> String {
    if dur.num_days() > 0 {
        singular_or_plural("day", dur.num_days())
    } else if dur.num_hours() > 0 {
        singular_or_plural("hour", dur.num_hours())
    } else {
        singular_or_plural("minute", dur.num_minutes())
    }
}

fn queue_notification(config: &Config, uptime: &Uptime, prowl_queue: &ProwlQueueSender) {
    let hostname = hostname();
    let uptime_duration = uptime
        .last_heartbeat()
        .signed_duration_since(*uptime.start());
    let reboot_duration = Utc::now().signed_duration_since(*uptime.last_heartbeat());
    let reboot_time_string = duration_string(reboot_duration);
    let uptime_string = duration_string(uptime_duration);
    let event = "Restarted".to_string();
    let description = format!("{hostname} was rebooted. Took {reboot_time_string} to come back up. Host was previously up for {uptime_string}.");
    let notification = Notification::new(
        config.prowl_api_keys().to_owned(),
        None,
        None,
        hostname,
        event,
        description,
    )
    .expect("Failed to create notification");
    log::debug!("Queueing notification {:?}", notification);
    prowl_queue
        .add(notification)
        .expect("Failed to queue notification");
}

fn queue_notifications(config: &Config, data: &mut Data, prowl_queue: ProwlQueueSender) {
    while data.uptime_len() > 1 {
        let uptime = data.delete_first_uptime();
        queue_notification(config, &uptime, &prowl_queue);
    }
}

// data with 1 event = 62 bytes

#[tokio::main]
async fn main() {
    env_logger::init();

    let config = Config::load(std::env::args().nth(1));
    let mut data = Data::load_or_default(&config);
    data.create_new_uptime();
    let (sender, reciever) = ProwlQueue::default().into_parts();
    queue_notifications(&config, &mut data, sender);
    tokio::spawn(reciever.async_loop());

    loop {
        data.beat();
        data.save(&config);
        sleep(Duration::from_secs(60)).await;
    }
}
