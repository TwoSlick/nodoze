use std::time::{Duration, SystemTime};

use crate::audio;
use crate::config::Config;

const POLL_INTERVAL: Duration = Duration::from_secs(1);
const RETRY_DELAY: Duration = Duration::from_secs(5);

/// Run the nodoze daemon loop.
///
/// Uses wall-clock time (SystemTime) to track intervals rather than
/// monotonic sleep. This correctly handles system sleep/wake:
/// - Monotonic clocks pause during sleep, so thread::sleep(540s) would
///   not account for time spent in system sleep
/// - Wall-clock time advances during sleep, so after waking we
///   immediately detect that the interval has elapsed and play a tone
pub fn run(config: &Config) {
    log::info!(
        "Starting nodoze daemon: {}Hz tone, {}s duration, every {}s",
        config.frequency,
        config.duration,
        config.interval
    );

    let interval = Duration::from_secs(config.interval);

    // Play immediately on startup
    let mut last_play = match audio::play_tone(config) {
        Ok(()) => {
            log::info!("Initial tone played successfully");
            SystemTime::now()
        }
        Err(e) => {
            log::error!("Initial tone failed: {}", e);
            // Set last_play far in the past so we retry quickly
            SystemTime::UNIX_EPOCH
        }
    };

    loop {
        std::thread::sleep(POLL_INTERVAL);

        let elapsed = last_play.elapsed().unwrap_or(interval);

        if elapsed >= interval {
            match audio::play_tone(config) {
                Ok(()) => {
                    if elapsed > interval + Duration::from_secs(10) {
                        log::info!(
                            "Tone played after wake ({}s since last play)",
                            elapsed.as_secs()
                        );
                    } else {
                        log::debug!("Tone played successfully");
                    }
                    last_play = SystemTime::now();
                }
                Err(e) => {
                    log::warn!("Failed to play tone (retrying in {}s): {}", RETRY_DELAY.as_secs(), e);
                    // Sleep a short retry delay. On next poll, elapsed will still
                    // be >= interval so we'll try again immediately.
                    std::thread::sleep(RETRY_DELAY);
                }
            }
        }
    }
}
