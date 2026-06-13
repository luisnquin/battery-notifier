use chrono::Utc;
use linuxver::version as get_linux_version;
use log::{info, warn};
use serde::Deserialize;
use std::{fmt, fs, io, ops::Index};

const POWER_SUPPLY_BASE: &str = "/sys/class/power_supply";

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

        let path = Self::detect_battery_path().unwrap_or_else(|| {
            let fallback = format!("{}/BAT0", POWER_SUPPLY_BASE);
            warn!("no battery node found under {POWER_SUPPLY_BASE}, falling back to {fallback}");
            fallback
        });
        info!("using battery node at {path}");

        PowerSupplyClass {
            path,
            debug: debug_file_path.map(|p| {
                let settings = DebugSettings::parse(p);
                Debug::new(settings)
            }),
        }
    }

    // Scans the power supply class for the first node reporting type "Battery"
    // and exposing a capacity file, instead of guessing BAT0/BAT1 from the OS.
    fn detect_battery_path() -> Option<String> {
        fs::read_dir(POWER_SUPPLY_BASE).ok()?.flatten().find_map(|entry| {
            let path = entry.path();
            let kind = fs::read_to_string(path.join("type")).ok()?;

            if kind.trim() == "Battery" && path.join("capacity").exists() {
                Some(path.to_string_lossy().into_owned())
            } else {
                None
            }
        })
    }

    pub fn get_capacity(&mut self) -> io::Result<u8> {
        if self.debug.is_some() {
            let debug = self.debug.as_mut().unwrap();
            let now = Utc::now().time();

            if debug.should_move_to_next_state(now) {
                debug.last_update_at = now;
                debug.next_state();
            };

            return Ok(debug.get_current_state().capacity);
        }

        let raw_capacity = fs::read_to_string(self.get_capacity_path())?.replace("\n", "");

        raw_capacity.parse::<u8>().map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("battery capacity file does not contain a number ({raw_capacity:?}): {e}"),
            )
        })
    }

    pub fn get_status(&mut self) -> io::Result<String> {
        if self.debug.is_some() {
            let debug = self.debug.as_mut().unwrap();
            let now = Utc::now().time();

            if debug.should_move_to_next_state(now) {
                debug.last_update_at = now;
                debug.next_state();
            };

            return Ok(debug.get_current_state().status);
        }

        Ok(fs::read_to_string(self.get_status_path())?.replace("\n", ""))
    }

    fn get_capacity_path(&self) -> String {
        format!("{}/capacity", self.path)
    }

    fn get_status_path(&self) -> String {
        format!("{}/status", self.path)
    }
}

#[derive(PartialEq, Clone, Copy)]
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

    fn get_current_state(&self) -> DebugState {
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
