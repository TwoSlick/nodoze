use std::time::Duration;

use crate::audio;
use crate::config::Config;

/// Run the wake-speaker daemon loop
pub fn run(config: &Config) {
    log::info!(
        "Starting wake-speaker daemon: {}Hz tone, {}s duration, every {}s",
        config.frequency,
        config.duration,
        config.interval
    );

    loop {
        match audio::play_tone(config) {
            Ok(()) => log::debug!("Tone played successfully"),
            Err(e) => log::error!("Failed to play tone: {}", e),
        }

        log::debug!("Sleeping for {}s until next tone", config.interval);
        std::thread::sleep(Duration::from_secs(config.interval));
    }
}
