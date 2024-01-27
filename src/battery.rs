use chrono::Utc;
use linuxver::version as get_linux_version;
use serde::Deserialize;
use std::{fmt, fs, ops::Index};

pub struct PowerSupplyClass {
    path: String,
    debug: Option<Debug>,
}

impl PowerSupplyClass {
    pub fn new(debug_file_path: Option<String>) -> PowerSupplyClass {
        let kernel_version = get_linux_version().expect("must use a Linux kernel");
        if kernel_version.major == 2 && kernel_version.minor < 6 {
            panic!("This program requires Linux 2.6 or higher");
        }

        let class = match os_info::get().os_type() {
            os_info::Type::Ubuntu => "BAT0",
            _ => {
                if kernel_version.major < 3
                    || (kernel_version.major == 3 && kernel_version.minor < 19)
                {
                    "BAT0"
                } else {
                    "BAT1"
                }
            }
        };

        PowerSupplyClass {
            path: format!("/sys/class/power_supply/{}", class),
            debug: debug_file_path.map(|p| {
                let settings = DebugSettings::parse(p);
                Debug::new(settings)
            }),
        }
    }

    pub fn get_capacity(&mut self) -> u8 {
        if self.debug.is_some() {
            let debug = self.debug.as_mut().unwrap();
            let now = Utc::now().time();

            if debug.should_move_to_next_state(now) {
                debug.last_update_at = now;
                debug.next_state();
            };

            return debug.get_state().capacity;
        }

        let raw_capacity: String = fs::read_to_string(self.get_capacity_path())
            .expect("Read battery capacity file")
            .replace("\n", "");

        raw_capacity
            .parse::<u8>()
            .expect("BAT1 capacity file doesn't contains a number")
    }

    pub fn get_status(&mut self) -> String {
        if self.debug.is_some() {
            let debug = self.debug.as_mut().unwrap();
            let now = Utc::now().time();

            if debug.should_move_to_next_state(now) {
                debug.last_update_at = now;
                debug.next_state();
            };

            return debug.get_state().status;
        }

        fs::read_to_string(self.get_status_path())
            .expect("Read battery status file")
            .replace("\n", "")
    }

    fn get_capacity_path(&self) -> String {
        format!("{}/capacity", self.path)
    }

    fn get_status_path(&self) -> String {
        format!("{}/status", self.path)
    }
}

#[derive(PartialEq)]
pub enum BatteryNotificationLevel {
    NoConflict,
    Reminder,
    Warn,
    Threat,
    Charging,
}

impl fmt::Display for BatteryNotificationLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BatteryNotificationLevel::NoConflict => write!(f, "no conflict(0)"),
            BatteryNotificationLevel::Reminder => write!(f, "reminder(1)"),
            BatteryNotificationLevel::Warn => write!(f, "warn(2)"),
            BatteryNotificationLevel::Threat => write!(f, "threat(3)"),
            BatteryNotificationLevel::Charging => write!(f, "charging(-1)"),
        }
    }
}

#[derive(Clone, Deserialize)]
struct DebugState {
    status: String,
    capacity: u8,
}

#[derive(Clone, Deserialize)]
pub struct DebugSettings {
    states: Vec<DebugState>,
    seconds_between: i64,
}

impl DebugSettings {
    pub fn parse(debug_file_path: String) -> Self {
        let content = fs::read_to_string(debug_file_path).expect("read file path");
        let options: DebugSettings = serde_yaml::from_str(&content).expect("parse debug file");

        options
    }
}

struct Debug {
    settings: DebugSettings,
    current_state: usize,
    last_update_at: chrono::NaiveTime,
}

impl Debug {
    fn new(settings: DebugSettings) -> Self {
        Self {
            settings,
            current_state: 0,
            last_update_at: Utc::now().time(),
        }
    }

    fn get_state(&self) -> DebugState {
        self.settings.states.index(self.current_state).clone()
    }

    fn should_move_to_next_state(&self, now: chrono::NaiveTime) -> bool {
        let diff = now - self.last_update_at;
        diff.num_seconds() >= self.settings.seconds_between
    }

    fn next_state(&mut self) {
        let mut next_state = self.current_state + 1;

        if next_state >= self.settings.states.len() {
            next_state = 0
        }

        self.current_state = next_state
    }
}
