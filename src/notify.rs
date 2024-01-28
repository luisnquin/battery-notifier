use log::{debug, error};
use notify_rust::{error, Hint, Notification, NotificationHandle};
use soloud::{audio::Wav, AudioExt, LoadExt, Soloud};
use std::{env, fmt, path::Path, thread, time};

pub const BATTERY_DANGER_PATH: &str = "./assets/battery-danger.png";

pub const CHARGING_BATTERY_SOUND: &[u8] = include_bytes!("./../assets/sounds/charging.mp3");
pub const REMINDER_BATTERY_SOUND: &[u8] = include_bytes!("./../assets/sounds/30.mp3");
pub const THREAT_BATTERY_SOUND: &[u8] = include_bytes!("./../assets/sounds/5.mp3");
pub const WARN_BATTERY_SOUND: &[u8] = include_bytes!("./../assets/sounds/15.mp3");

#[derive(Clone, Copy)]
pub enum Urgency {
    CRITICAL,
    NORMAL,
    LOW,
}

impl Urgency {
    pub fn get_sound(&self) -> &[u8] {
        match self {
            Urgency::CRITICAL => THREAT_BATTERY_SOUND,
            Urgency::NORMAL => WARN_BATTERY_SOUND,
            Urgency::LOW => REMINDER_BATTERY_SOUND,
        }
    }

    fn get_for_third_party(&self) -> notify_rust::Urgency {
        match self {
            Urgency::CRITICAL => notify_rust::Urgency::Critical,
            Urgency::NORMAL => notify_rust::Urgency::Normal,
            Urgency::LOW => notify_rust::Urgency::Low,
        }
    }
}

impl fmt::Display for Urgency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Urgency::CRITICAL => write!(f, "critical"),
            Urgency::NORMAL => write!(f, "normal"),
            Urgency::LOW => write!(f, "low"),
        }
    }
}

pub fn send_desktop_notification(
    urgency: Urgency,
    title: &str,
    content: &str,
    icon_path: Option<String>,
) -> error::Result<NotificationHandle> {
    let result = Notification::new()
        .summary(title)
        .body(content)
        .icon(&get_icon_path_or_default(icon_path))
        .hint(Hint::Category("string:x-stack-tag:battery".to_string()))
        .hint(Hint::Urgency(urgency.get_for_third_party()))
        .show();

    result.map(|r| return r)
}

pub fn send_sound_notification(sound: &[u8]) {
    let rsl = Soloud::default();

    match rsl {
        Ok(sl) => {
            let mut wav = Wav::default();

            match wav.load_mem(sound) {
                Ok(r) => debug!("sound file has been loaded: {:#?}", r),
                Err(error) => error!("couldn't load sound file: {}", error.to_string()),
            };

            sl.play(&wav);
            while sl.voice_count() > 0 {
                thread::sleep(time::Duration::from_millis(500));
            }
        }
        Err(error) => error!(
            "[ERROR] soloud instance couldn't be correctly initialized: {}",
            error.to_string()
        ),
    }
}

pub fn get_icon_path_or_default(icon_path: Option<String>) -> String {
    if icon_path.is_some() {
        let p = icon_path.unwrap();
        if p != "" {
            return p;
        }
    };

    if Path::new(BATTERY_DANGER_PATH).is_relative() {
        let cwd = env::current_dir().expect("get current directory");

        Path::new(cwd.to_str().unwrap())
            .join(BATTERY_DANGER_PATH)
            .to_owned()
            .to_string_lossy()
            .to_string()
    } else {
        BATTERY_DANGER_PATH.to_owned()
    }
}

// Could a new romance ever _find you?
