use chrono::Utc;
use clap::Parser;
use std::{
    thread,
    time::{self},
};

mod config;
use config::*;

mod notify;
use notify::*;

mod battery;
use battery::*;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    /// To simulate battery states (yaml).
    debug_file: Option<String>,
    /// The config file path (toml).
    #[arg(short, long)]
    config_file: Option<String>,
}

fn main() {
    let args = Args::parse();

    let cp = get_config_file(args.config_file);
    let config = Config::parse_or_default(cp);

    let start_time = Utc::now().time();

    let sleep_time = time::Duration::from_millis(config.sleep_ms); // 0.7s
    let mut last_notification_level = BatteryNotificationLevel::NoConflict;
    let mut psc = PowerSupplyClass::new(args.debug_file);

    // Calculates the notification level based on the provided battery capacity.
    let get_notification_level = |capacity: u8| -> BatteryNotificationLevel {
        match capacity {
            c if c <= config.reminder_threshold => BatteryNotificationLevel::Reminder,
            c if c <= config.warn_threshold => BatteryNotificationLevel::Warn,
            c if c <= config.threat_threshold => BatteryNotificationLevel::Threat,
            _ => BatteryNotificationLevel::NoConflict,
        }
    };

    loop {
        let capacity = psc.get_capacity();
        let status = psc.get_status();

        println!("[DEBUG] Current capacity: {} Status: {}", capacity, status);

        if status == "Charging" && last_notification_level != BatteryNotificationLevel::Charging {
            println!("[DEBUG] Now the battery is charging...");
            println!(
                "[DEBUG] The last notified capacity will be restarted to 0 (it was {})",
                last_notification_level
            );

            let current_time = Utc::now().time();

            if (current_time - start_time).num_seconds() > 5 {
                send_sound_notification(CHARGING_BATTERY_SOUND);
            } else {
                println!("[WARNING] the app started with the computer plugged in, nothing to do");
            }

            last_notification_level = BatteryNotificationLevel::Charging
        } else if status == "Discharging" || status == "Not charging" {
            let default_content = format!("Charge: {}%", capacity);

            let mut notify_capacity = |urgency: Urgency, title: &str, content: &str| {
                let current_notification_level = get_notification_level(capacity);

                println!(
                    "[DEBUG] Last notification level: {}, Current notification level: {}",
                    last_notification_level, current_notification_level
                );

                if last_notification_level != current_notification_level {
                    last_notification_level = current_notification_level;

                    match send_desktop_notification(urgency, title, content) {
                        Ok(r) => println!("[DEBUG] Battery notification: {:#?}", r),
                        Err(error) => {
                            println!("[ERROR] Battery notification: {}", error.to_string())
                        }
                    };

                    send_sound_notification(urgency.get_sound())
                }
            };

            match get_notification_level(capacity) {
                BatteryNotificationLevel::Reminder => {
                    notify_capacity(Urgency::LOW, "Battery somewhat low", &default_content)
                }
                BatteryNotificationLevel::Warn => notify_capacity(
                    Urgency::NORMAL,
                    "Battery low",
                    format!("{}.\nPlease connect your laptop", default_content).as_str(),
                ),
                BatteryNotificationLevel::Threat => notify_capacity(
                    Urgency::CRITICAL,
                    "Battery very low",
                    format!(
                        "{}.\n\nYour computer will shut down soon! You'll regret this!",
                        default_content
                    )
                    .as_str(),
                ),
                _ => (),
            }
        }

        thread::sleep(sleep_time);
    }
}
