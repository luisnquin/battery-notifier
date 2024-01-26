use soloud::{audio::Wav, AudioExt, LoadExt, Soloud};
use std::{env, fmt, fs, io, io::ErrorKind, path::Path, process, thread, time};

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
) -> io::Result<process::Output> {
    let icon_path = if Path::new(BATTERY_DANGER_PATH).is_relative() {
        let cwd = env::current_dir().expect("get current directory");

        Path::new(cwd.to_str().unwrap())
            .join(BATTERY_DANGER_PATH)
            .to_owned()
            .to_string_lossy()
            .to_string()
    } else {
        BATTERY_DANGER_PATH.to_owned()
    };

    if is_program_in_path("notify-send") {
        return process::Command::new("notify-send")
            .arg(format!("--urgency={}", urgency.to_string()))
            .arg(format!("--hint={}", "string:x-dunst-stack-tag:battery"))
            .arg(format!("--icon={}", icon_path))
            .arg(title)
            .arg(content)
            .output();
    } else {
        let err = io::Error::new(ErrorKind::NotFound, "notify-send were not found in $PATH");
        return Result::Err(err);
    }
}

pub fn send_sound_notification(sound: &[u8]) {
    let rsl = Soloud::default();

    match rsl {
        Ok(sl) => {
            let mut wav = Wav::default();

            match wav.load_mem(sound) {
                Ok(r) => println!("[DEBUG] Sound file has been loaded: {:#?}", r),
                Err(error) => {
                    println!("[WARN] Couldn't load sound file: {}", error.to_string())
                }
            };

            sl.play(&wav);
            while sl.voice_count() > 0 {
                thread::sleep(time::Duration::from_millis(500));
            }
        }
        Err(error) => println!(
            "[ERROR] soloud instance couldn't be correctly initialized: {}",
            error.to_string()
        ),
    }
}

fn is_program_in_path(program_name: &str) -> bool {
    if let Ok(path) = env::var("PATH") {
        for p in path.split(":") {
            let p_str = format!("{}/{}", p, program_name);

            if fs::metadata(p_str).is_ok() {
                return true;
            }
        }
    }

    false
}
