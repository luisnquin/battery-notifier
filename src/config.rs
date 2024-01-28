use log::{error, info, warn};
use serde::Deserialize;
use std::{env, fs, path::Path, process};

#[derive(Debug, Clone, Deserialize)]
pub struct Bound {
    #[serde(default)]
    pub threshold: u8,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub content: String,
}

impl Default for Bound {
    fn default() -> Self {
        Self {
            threshold: 0,
            title: String::new(),
            content: String::new(),
        }
    }
}

impl Bound {
    pub fn render_title(&self, capacity: u8) -> String {
        self.title
            .replace("${{capacity}}", capacity.to_string().as_str())
    }

    pub fn render_content(&self, capacity: u8) -> String {
        self.content
            .replace("${{capacity}}", capacity.to_string().as_str())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub icon_path: Option<String>,
    #[serde(default)]
    pub reminder: Bound,
    #[serde(default)]
    pub threat: Bound,
    #[serde(default)]
    pub warn: Bound,
    #[serde(default)]
    pub interval_ms: u64,
}

impl Default for Config {
    fn default() -> Self {
        let default_body = "Charge: ${{capacity}}%";

        Config {
            interval_ms: 700,
            icon_path: None,
            reminder: Bound {
                title: "Battery somewhat low".to_string(),
                content: default_body.to_string(),
                threshold: 30,
            },
            warn: Bound {
                title: "Battery low".to_string(),
                content: format!("{}.\nPlease connect your laptop", default_body),
                threshold: 15,
            },
            threat: Bound {
                title: "Battery very low".to_string(),
                content: format!(
                    "{}.\n\nYour computer will shut down soon! You'll regret this!",
                    default_body
                ),
                threshold: 5,
            },
        }
    }
}

impl Config {
    pub fn parse(config_path: String) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(config_path)?;
        let config: Config = toml::from_str(&content)?;

        Ok(config.merge(Config::default()))
    }

    pub fn parse_or_default(config_path: String) -> Self {
        match Config::parse(config_path) {
            Ok(config) => config,
            Err(error) => {
                error!("unable to parse user config: {}", error);
                info!("default config will be used");
                Config::default() // Provide a default value or handle it as needed
            }
        }
    }

    pub fn validate(&self) {
        if self.reminder.threshold <= self.warn.threshold {
            error!("reminder threshold must be higher than warn threshold");
            process::exit(1);
        }

        if self.warn.threshold <= self.threat.threshold {
            error!("warn threshold must be higher than threat threshold");
            process::exit(1);
        }
    }

    fn merge(mut self, other: Config) -> Config {
        let warn_if_not_zero = |threshold: u8, label: &str| {
            if threshold != 0 {
                warn!(
                    "invalid {} threshold '{}', default will be used",
                    label, threshold
                )
            }
        };

        if self.reminder.threshold <= 0 || self.reminder.threshold > 100 {
            warn_if_not_zero(self.reminder.threshold, "reminder");
            self.reminder.threshold = other.reminder.threshold
        }

        if self.reminder.title == "" {
            self.reminder.title = other.reminder.title
        }

        if self.reminder.content == "" {
            self.reminder.content = other.reminder.content
        }

        if self.threat.threshold <= 0 || self.threat.threshold > 100 {
            warn_if_not_zero(self.threat.threshold, "threat");
            self.threat.threshold = other.threat.threshold
        }

        if self.threat.title == "" {
            self.threat.title = other.threat.title
        }

        if self.threat.content == "" {
            self.threat.content = other.threat.content
        }

        if self.warn.threshold <= 0 || self.warn.threshold > 100 {
            warn_if_not_zero(self.warn.threshold, "warn");
            self.warn.threshold = other.warn.threshold
        }

        if self.warn.title == "" {
            self.warn.title = other.warn.title
        }

        if self.warn.content == "" {
            self.warn.content = other.warn.content
        }

        if self.interval_ms <= 0 {
            self.interval_ms = other.interval_ms
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
