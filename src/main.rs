mod models;

use chrono::Utc;
use models::{Config, Data, Uptime};
use prowl::Notification;
use tokio::{
    sync::mpsc,
    time::{sleep, Duration},
};

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

async fn queue_notification(
    config: &Config,
    uptime: &Uptime,
    sender: &mpsc::UnboundedSender<Notification>,
) {
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
    log::debug!("Queueing notification {:?}", notification);
    sender
        .send(notification)
        .expect("Failed to queue notification");
}

async fn send_notifications(mut reciever: mpsc::UnboundedReceiver<Notification>) {
    log::debug!("Notifications channel processor started.");
    while let Some(notification) = reciever.recv().await {
        'notification: loop {
            match notification.add().await {
                Ok(_) => break 'notification,
                Err(e) => {
                    log::error!("Failed to send notification due to {:?}", e);
                    sleep(Duration::from_secs(30)).await;
                }
            }
        }
    }
    log::warn!("Notification channel has been closed.");
}

async fn queue_notifications(
    config: &Config,
    data: &mut Data,
    sender: mpsc::UnboundedSender<Notification>,
) {
    while data.uptime_len() > 1 {
        let uptime = data.delete_first_uptime();
        queue_notification(config, &uptime, &sender).await;
    }
    drop(sender);
}

// data with 1 event = 62 bytes

#[tokio::main]
async fn main() {
    env_logger::init();

    let config = Config::load(std::env::args().nth(1));
    let mut data = Data::load_or_default(&config);
    let (sender, reciever) = mpsc::unbounded_channel();
    data.create_new_uptime();
    queue_notifications(&config, &mut data, sender).await;
    tokio::spawn(send_notifications(reciever));

    loop {
        data.beat();
        data.save(&config);
        sleep(Duration::from_secs(60)).await;
    }
}
