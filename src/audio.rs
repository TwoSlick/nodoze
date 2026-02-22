use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, Device, SampleFormat, StreamConfig};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use crate::config::Config;

/// Get the human-readable name of a device
fn device_name(device: &Device) -> Option<String> {
    device.description().ok().map(|d| d.name().to_string())
}

/// Find an output device by name, or return the default
pub fn get_device(name: &str) -> Result<Device, String> {
    let host = cpal::default_host();

    if name.is_empty() {
        return host
            .default_output_device()
            .ok_or_else(|| "No default output device found".to_string());
    }

    let devices = host
        .output_devices()
        .map_err(|e| format!("Failed to enumerate devices: {}", e))?;

    for device in devices {
        if let Some(dev_name) = device_name(&device) {
            if dev_name.to_lowercase().contains(&name.to_lowercase()) {
                return Ok(device);
            }
        }
    }

    Err(format!("No output device matching '{}' found", name))
}

/// List all available output devices
pub fn list_devices() -> Result<Vec<String>, String> {
    let host = cpal::default_host();
    let devices = host
        .output_devices()
        .map_err(|e| format!("Failed to enumerate devices: {}", e))?;

    let mut names = Vec::new();
    let default_name = host
        .default_output_device()
        .and_then(|d| device_name(&d))
        .unwrap_or_default();

    for device in devices {
        if let Some(name) = device_name(&device) {
            let is_default = name == default_name;
            names.push(if is_default {
                format!("{} (default)", name)
            } else {
                name
            });
        }
    }

    Ok(names)
}

/// Play a sine wave tone with fade in/out
pub fn play_tone(config: &Config) -> Result<(), String> {
    let device = get_device(&config.device)?;
    let dev_name = device_name(&device).unwrap_or_else(|| "unknown".into());
    log::info!(
        "Playing {}Hz tone for {}s at {:.0}% volume on '{}'",
        config.frequency,
        config.duration,
        config.volume * 100.0,
        dev_name
    );

    let supported_config = device
        .default_output_config()
        .map_err(|e| format!("Failed to get default output config: {}", e))?;

    let sample_rate = supported_config.sample_rate() as f64;
    let channels = supported_config.channels() as usize;

    let frequency = config.frequency;
    let volume = config.volume.clamp(0.0, 1.0) as f32;
    let total_samples = (config.duration as f64 * sample_rate) as u64;
    let fade_samples = (config.fade_duration * sample_rate) as u64;

    let sample_clock = Arc::new(std::sync::atomic::AtomicU64::new(0));
    let finished = Arc::new(AtomicBool::new(false));
    let finished_clone = finished.clone();
    let sample_clock_clone = sample_clock.clone();

    let mut stream_config: StreamConfig = supported_config.clone().into();
    stream_config.buffer_size = BufferSize::Fixed(4096);

    let err_fn = |err| log::warn!("Audio stream: {}", err);

    let stream = match supported_config.sample_format() {
        SampleFormat::F32 => device.build_output_stream(
            &stream_config,
            move |data: &mut [f32], _| {
                write_samples(
                    data,
                    channels,
                    &sample_clock_clone,
                    sample_rate,
                    frequency,
                    volume,
                    total_samples,
                    fade_samples,
                    &finished_clone,
                );
            },
            err_fn,
            None,
        ),
        SampleFormat::I16 => device.build_output_stream(
            &stream_config,
            move |data: &mut [i16], _| {
                let mut float_buf = vec![0.0f32; data.len()];
                write_samples(
                    &mut float_buf,
                    channels,
                    &sample_clock_clone,
                    sample_rate,
                    frequency,
                    volume,
                    total_samples,
                    fade_samples,
                    &finished_clone,
                );
                for (out, &sample) in data.iter_mut().zip(float_buf.iter()) {
                    *out = (sample * i16::MAX as f32) as i16;
                }
            },
            err_fn,
            None,
        ),
        SampleFormat::U16 => device.build_output_stream(
            &stream_config,
            move |data: &mut [u16], _| {
                let mut float_buf = vec![0.0f32; data.len()];
                write_samples(
                    &mut float_buf,
                    channels,
                    &sample_clock_clone,
                    sample_rate,
                    frequency,
                    volume,
                    total_samples,
                    fade_samples,
                    &finished_clone,
                );
                for (out, &sample) in data.iter_mut().zip(float_buf.iter()) {
                    *out = ((sample * 0.5 + 0.5) * u16::MAX as f32) as u16;
                }
            },
            err_fn,
            None,
        ),
        _ => return Err("Unsupported sample format".to_string()),
    }
    .map_err(|e| format!("Failed to build output stream: {}", e))?;

    stream
        .play()
        .map_err(|e| format!("Failed to play stream: {}", e))?;

    // Wait for playback to complete
    while !finished.load(Ordering::Relaxed) {
        std::thread::sleep(Duration::from_millis(100));
    }

    // Small delay to let the stream drain
    std::thread::sleep(Duration::from_millis(50));
    drop(stream);

    log::info!("Tone playback complete");
    Ok(())
}

fn write_samples(
    data: &mut [f32],
    channels: usize,
    sample_clock: &std::sync::atomic::AtomicU64,
    sample_rate: f64,
    frequency: f64,
    volume: f32,
    total_samples: u64,
    fade_samples: u64,
    finished: &AtomicBool,
) {
    for frame in data.chunks_mut(channels) {
        let n = sample_clock.fetch_add(1, Ordering::Relaxed);

        if n >= total_samples {
            finished.store(true, Ordering::Relaxed);
            for sample in frame.iter_mut() {
                *sample = 0.0;
            }
            continue;
        }

        // Generate sine wave
        let t = n as f64 / sample_rate;
        let value = (2.0 * std::f64::consts::PI * frequency * t).sin();

        // Apply fade envelope
        let envelope = if n < fade_samples {
            // Fade in
            n as f64 / fade_samples as f64
        } else if n > total_samples - fade_samples {
            // Fade out
            (total_samples - n) as f64 / fade_samples as f64
        } else {
            1.0
        };

        let sample = (value * envelope) as f32 * volume;

        for s in frame.iter_mut() {
            *s = sample;
        }
    }
}
