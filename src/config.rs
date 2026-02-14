use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Config {
    /// Tone frequency in Hz
    #[serde(default = "default_frequency")]
    pub frequency: f64,

    /// Duration of each tone in seconds
    #[serde(default = "default_duration")]
    pub duration: u64,

    /// Interval between tones in seconds
    #[serde(default = "default_interval")]
    pub interval: u64,

    /// Fade in/out duration in seconds
    #[serde(default = "default_fade_duration")]
    pub fade_duration: f64,

    /// Volume (0.0 to 1.0, where 0.05 = 5%)
    #[serde(default = "default_volume")]
    pub volume: f64,

    /// Audio output device name (empty = default)
    #[serde(default)]
    pub device: String,
}

fn default_frequency() -> f64 {
    20.0
}
fn default_duration() -> u64 {
    15
}
fn default_interval() -> u64 {
    540
}
fn default_fade_duration() -> f64 {
    1.0
}
fn default_volume() -> f64 {
    0.05
}

impl Default for Config {
    fn default() -> Self {
        Self {
            frequency: default_frequency(),
            duration: default_duration(),
            interval: default_interval(),
            fade_duration: default_fade_duration(),
            volume: default_volume(),
            device: String::new(),
        }
    }
}

impl Config {
    pub fn load(path: Option<&str>) -> Self {
        if let Some(p) = path {
            return Self::load_from_path(&PathBuf::from(p));
        }

        if let Some(config_dir) = dirs::config_dir() {
            let config_path = config_dir.join("wake-speaker").join("config.toml");
            if config_path.exists() {
                return Self::load_from_path(&config_path);
            }
        }

        log::info!("No config file found, using defaults");
        Self::default()
    }

    fn load_from_path(path: &PathBuf) -> Self {
        match std::fs::read_to_string(path) {
            Ok(contents) => match toml::from_str(&contents) {
                Ok(config) => {
                    log::info!("Loaded config from {}", path.display());
                    config
                }
                Err(e) => {
                    log::warn!("Failed to parse config {}: {}", path.display(), e);
                    Self::default()
                }
            },
            Err(e) => {
                log::warn!("Failed to read config {}: {}", path.display(), e);
                Self::default()
            }
        }
    }

    pub fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join("wake-speaker").join("config.toml"))
    }
}
