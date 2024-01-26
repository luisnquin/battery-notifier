use serde::Deserialize;
use std::{env, error, fs, path::Path};

#[derive(Clone, Deserialize)]
pub struct Config {
    pub reminder_threshold: u8,
    pub threat_threshold: u8,
    pub warn_threshold: u8,
    pub sleep_ms: u64,
}

impl Config {
    fn default() -> Self {
        Config {
            sleep_ms: 700,
            reminder_threshold: 30,
            threat_threshold: 15,
            warn_threshold: 5,
        }
    }

    pub fn parse(config_path: String) -> Result<Self, Box<dyn error::Error>> {
        let content = fs::read_to_string(config_path)?;
        let config: Config = toml::from_str(&content)?;

        Ok(Config::default().merge(config))
    }

    pub fn parse_or_default(config_path: String) -> Self {
        Config::parse(config_path).unwrap_or_else(|_| Config::default())
    }

    fn merge(mut self, other: Config) -> Config {
        if other.reminder_threshold != 0 {
            self.reminder_threshold = other.reminder_threshold
        }

        if other.threat_threshold != 0 {
            self.threat_threshold = other.threat_threshold
        }

        if other.warn_threshold != 0 {
            self.warn_threshold = other.warn_threshold
        }

        if other.sleep_ms != 0 {
            self.sleep_ms = other.sleep_ms
        }

        self
    }
}

pub fn get_config_file(file_path: Option<String>) -> String {
    file_path.unwrap_or_else(|| {
        let config_path = match env::var("XDG_CONFIG_FILE") {
            Ok(p) => String::from(p),
            Err(_) => {
                let fallback_path = Path::new(&env::var("HOME").unwrap())
                    .join(".config")
                    .to_owned();

                fallback_path.to_string_lossy().to_string()
            }
        };

        return Path::new(config_path.as_str())
            .join("battery-notifier")
            .join("config.toml")
            .to_str()
            .unwrap()
            .to_owned();
    })
}
