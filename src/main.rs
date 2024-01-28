use chrono::Utc;
use clap::Parser;
use log::{debug, error, info, warn, LevelFilter};
use notify_rust::NotificationHandle;
use std::{
    thread,
    time::{self},
};

mod cli;

mod config;
use config::*;

mod notify;
use notify::*;

mod battery;
use battery::*;

fn main() {
    env_logger::builder()
        .filter_level(LevelFilter::Debug)
        .init();

    let args = cli::Args::parse();
    debug!("{:#?}", args);

    let cp = get_config_file(args.config_file);
    debug!("config file path is {}", cp);

    let config = Config::parse_or_default(cp);
    debug!("{:#?}", config);

    // Calculates the notification level based on the provided battery capacity.
    let get_notification_level = |capacity: u8| -> BatteryNotificationLevel {
        match capacity {
            c if c > config.warn.threshold && c <= config.reminder.threshold => {
                BatteryNotificationLevel::Reminder
            }
            c if c > config.threat.threshold && c <= config.warn.threshold => {
                BatteryNotificationLevel::Warn
            }
            c if c <= config.threat.threshold => BatteryNotificationLevel::Threat,
            _ => BatteryNotificationLevel::NoConflict,
        }
    };

    let start_time = Utc::now().time();
    let sleep_time = time::Duration::from_millis(config.interval_ms);

    let mut last_notification_level = BatteryNotificationLevel::NoConflict;
    let mut last_notification_handler: Option<NotificationHandle> = None;
    let mut psc = PowerSupplyClass::new(args.debug_file);

    loop {
        let capacity = psc.get_capacity();
        let status = psc.get_status();

        info!("current capacity: {} Status: {}", capacity, status);

        if status == "Charging" && last_notification_level != BatteryNotificationLevel::Charging {
            info!("now the battery is charging...");
            info!(
                "the last notified capacity will be restarted to 0 (it was {})",
                last_notification_level
            );

            let current_time = Utc::now().time();

            if (current_time - start_time).num_seconds() > 5 {
                last_notification_handler.take().map(|h| h.close());
                send_sound_notification(CHARGING_BATTERY_SOUND);
            } else {
                warn!("the app started with the computer plugged in, nothing to do");
            }

            last_notification_level = BatteryNotificationLevel::Charging
        } else if status == "Discharging" || status == "Not charging" {
            let current_notification_level = get_notification_level(capacity);

            if current_notification_level != BatteryNotificationLevel::NoConflict {
                let (urgency, bound) = match current_notification_level {
                    BatteryNotificationLevel::Reminder => (Urgency::LOW, &config.reminder),
                    BatteryNotificationLevel::Warn => (Urgency::NORMAL, &config.warn),
                    BatteryNotificationLevel::Threat => (Urgency::CRITICAL, &config.threat),
                    _ => panic!("unexpected battery notification level"),
                };

                debug!(
                    "last notification level: {}, current notification level: {}",
                    last_notification_level, current_notification_level
                );

                if last_notification_level != current_notification_level {
                    last_notification_handler.take().map(|h| h.close());

                    let result = send_desktop_notification(
                        urgency,
                        bound.render_title(capacity).as_str(),
                        bound.render_content(capacity).as_str(),
                        config.icon_path.to_owned(),
                    );

                    if result.is_ok() {
                        last_notification_handler = Some(result.unwrap());
                    } else {
                        error!(
                            "error sending desktop notification: {}",
                            result.unwrap_err()
                        )
                    }

                    send_sound_notification(urgency.get_sound());
                };

                info!(
                    "last notification level: {}, current notification level: {}",
                    last_notification_level, current_notification_level
                );
            }
        }

        thread::sleep(sleep_time);
    }
}
