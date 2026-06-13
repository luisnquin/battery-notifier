use clap::Parser;
use log::{debug, error, info, warn, LevelFilter};
use notify_rust::NotificationHandle;
use std::{
    thread,
    time::{self, Instant},
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
    config.validate();

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

    let start_time = Instant::now();
    let sleep_time = time::Duration::from_millis(config.interval_ms);

    let mut last_notification_level = BatteryNotificationLevel::NoConflict;
    let mut last_notification_handler: Option<NotificationHandle> = None;
    let mut psc = PowerSupplyClass::new(args.debug_file);

    loop {
        let capacity = match psc.get_capacity() {
            Ok(capacity) => capacity,
            Err(error) => {
                warn!("could not read battery capacity, skipping: {}", error);
                thread::sleep(sleep_time);
                continue;
            }
        };

        let status = match psc.get_status() {
            Ok(status) => status,
            Err(error) => {
                warn!("could not read battery status, skipping: {}", error);
                thread::sleep(sleep_time);
                continue;
            }
        };

        info!("current capacity: {} Status: {}", capacity, status);

        // The kernel reports several statuses for a plugged-in adapter ("Charging"
        // while filling, "Full" once topped off, "Not charging" when held below a
        // charge limit), and may sit on "Unknown" for a few seconds while the EC
        // settles after plug-in. Anything that is not "Discharging" or "Unknown"
        // means the adapter is connected.
        let plugged_in = matches!(status.as_str(), "Charging" | "Full" | "Not charging");

        // This double check is necessary to don't perform the same action repeated times
        if plugged_in && last_notification_level != BatteryNotificationLevel::Charging {
            info!("now the battery is plugged in (status: {})...", status);
            info!(
                "the last notified capacity will be restarted to 0 (it was {})",
                last_notification_level
            );

            if start_time.elapsed().as_secs() > 5 {
                last_notification_handler.take().map(|h| h.close());
                send_sound_notification(CHARGING_BATTERY_SOUND);
            } else {
                warn!("the app started with the computer plugged in, nothing to do");
            }

            last_notification_level = BatteryNotificationLevel::Charging
        } else if status == "Discharging" {
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
                    last_notification_level = current_notification_level;
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
