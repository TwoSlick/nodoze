mod audio;
mod config;
mod daemon;
mod service;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "wake-speaker",
    about = "Keep your speakers awake by playing an inaudible tone periodically",
    version
)]
struct Cli {
    /// Path to config file (default: ~/.config/wake-speaker/config.toml)
    #[arg(short, long, global = true)]
    config: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the daemon (plays tone at configured interval)
    Run,

    /// Play the tone once and exit
    Once,

    /// List available audio output devices
    ListDevices,

    /// Show active configuration
    Config,

    /// Install as a system service (LaunchAgent/systemd/Task Scheduler)
    Install,

    /// Remove the system service
    Uninstall,
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_secs()
        .init();

    let cli = Cli::parse();
    let cfg = config::Config::load(cli.config.as_deref());

    match cli.command.unwrap_or(Commands::Run) {
        Commands::Run => {
            daemon::run(&cfg);
        }
        Commands::Once => {
            if let Err(e) = audio::play_tone(&cfg) {
                log::error!("{}", e);
                std::process::exit(1);
            }
        }
        Commands::ListDevices => match audio::list_devices() {
            Ok(devices) => {
                println!("Available output devices:");
                for name in devices {
                    println!("  {}", name);
                }
            }
            Err(e) => {
                log::error!("{}", e);
                std::process::exit(1);
            }
        },
        Commands::Config => {
            println!("Active configuration:");
            println!("  Frequency:     {} Hz", cfg.frequency);
            println!("  Duration:      {} s", cfg.duration);
            println!("  Interval:      {} s ({:.1} min)", cfg.interval, cfg.interval as f64 / 60.0);
            println!("  Fade duration: {} s", cfg.fade_duration);
            println!("  Volume:        {:.0}%", cfg.volume * 100.0);
            println!(
                "  Device:        {}",
                if cfg.device.is_empty() {
                    "(system default)"
                } else {
                    &cfg.device
                }
            );
            if let Some(path) = config::Config::config_path() {
                println!(
                    "  Config file:   {} {}",
                    path.display(),
                    if path.exists() { "(found)" } else { "(not found, using defaults)" }
                );
            }
        }
        Commands::Install => {
            if let Err(e) = service::install() {
                log::error!("{}", e);
                std::process::exit(1);
            }
        }
        Commands::Uninstall => {
            if let Err(e) = service::uninstall() {
                log::error!("{}", e);
                std::process::exit(1);
            }
        }
    }
}
