# nodoze

Keep your speakers awake by playing an inaudible tone periodically.

Many external speakers and audio devices enter sleep mode after a period of silence (typically ~10 minutes). Waking them can take several seconds of sustained audio, which means short sounds like notifications get swallowed. **nodoze** prevents this by periodically playing a near-inaudible 20Hz tone that keeps the speakers active.

## Install

```sh
# macOS / Linux
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/TwoSlick/nodoze/releases/latest/download/nodoze-installer.sh | sh

# Windows
powershell -ExecutionPolicy ByPass -c "irm https://github.com/TwoSlick/nodoze/releases/latest/download/nodoze-installer.ps1 | iex"

# Homebrew
brew install TwoSlick/tap/nodoze

# From source
cargo install nodoze
```

## Features

- **No external dependencies** — uses [cpal](https://github.com/RustAudio/cpal) for native audio (CoreAudio, WASAPI, ALSA)
- **Cross-platform** — macOS, Linux, and Windows
- **Single binary** — ~2.5MB, no runtime needed
- **Sleep/wake aware** — detects system sleep via wall-clock tracking and plays immediately on wake
- **Configurable** — frequency, duration, interval, volume, fade, and device selection
- **Interactive setup** — `nodoze setup` wizard walks through configuration
- **Service install** — one command to run at login (LaunchAgent, systemd, Task Scheduler)

## Quick Start

```sh
# Build
cargo build --release

# Run the setup wizard
./target/release/nodoze setup

# Test with a single tone
./target/release/nodoze once

# Run the daemon
./target/release/nodoze run

# Install as a system service (starts on login)
./target/release/nodoze install
```

## Commands

| Command | Description |
|---------|-------------|
| `run` | Run the daemon (default if no command given) |
| `once` | Play the tone once and exit |
| `list-devices` | List available audio output devices |
| `config` | Show active configuration |
| `setup` | Interactive configuration wizard |
| `install` | Install as a system service |
| `uninstall` | Remove the system service |

## Configuration

Config file location: `~/.config/nodoze/config.toml`

Run `nodoze setup` to generate one interactively, or create it manually:

```toml
# Frequency of the tone in Hz (20 Hz is near the bottom of human hearing)
frequency = 20.0

# Duration of each tone in seconds
duration = 15

# Interval between tones in seconds (540 = 9 minutes)
interval = 540

# Fade in/out duration in seconds (prevents clicks/pops)
fade_duration = 1.0

# Volume (0.0 to 1.0) — 0.05 = 5%
volume = 0.05

# Audio output device name (empty string = system default)
# Use `nodoze list-devices` to see available devices
device = ""
```

All values have sensible defaults. The config file is optional — without it, nodoze uses the defaults shown above.

### Why these defaults?

- **20 Hz** is at the bottom of human hearing — most people can't hear it, but speakers register it as activity
- **15 seconds** is long enough to prevent the speaker's silence detector from triggering
- **540 seconds (9 minutes)** is just under the typical 10-minute sleep threshold
- **5% volume** is enough to keep speakers awake without causing distortion when other audio is playing
- **1 second fade** prevents clicks and pops from abrupt signal start/stop

## Service Installation

`nodoze install` registers the daemon to start automatically at login:

| Platform | Mechanism | File |
|----------|-----------|------|
| macOS | LaunchAgent | `~/Library/LaunchAgents/com.nodoze.daemon.plist` |
| Linux | systemd user unit | `~/.config/systemd/user/nodoze.service` |
| Windows | Task Scheduler | Task named `NoDoze` |

On macOS and Linux, the service auto-restarts if it crashes. Remove with `nodoze uninstall`.

## Sleep/Wake Handling

The daemon uses wall-clock time (`SystemTime`) rather than monotonic sleep to track intervals. This means:

1. Machine goes to sleep — daemon process suspends
2. Machine wakes — daemon detects the real elapsed time exceeds the interval
3. Tone plays within seconds of wake
4. If the audio device isn't ready yet, retries every 5 seconds

This ensures your speakers never stay asleep after your machine wakes up.

## Building

Requires Rust 2021 edition. Install via [rustup](https://rustup.rs/) (recommended) or your platform's package manager.

```sh
git clone https://github.com/TwoSlick/nodoze.git
cd nodoze
cargo build --release
```

The binary is at `./target/release/nodoze` (or `nodoze.exe` on Windows).

## Development Setup

### macOS

No extra system dependencies — cpal uses CoreAudio which is included in macOS.

```sh
# Install Rust
brew install rust
# or: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

cargo build
cargo run -- once    # quick smoke test
```

### Linux

Requires ALSA development headers and pkg-config for cpal's Linux audio backend.

```sh
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install ALSA dev headers + pkg-config
sudo apt install libasound2-dev pkg-config    # Debian/Ubuntu
sudo dnf install alsa-lib-devel pkg-config    # Fedora
sudo pacman -S alsa-lib pkg-config            # Arch

cargo build
cargo run -- once
```

### Windows

Requires the MSVC C++ build tools for cpal's WASAPI backend.

1. Install [Rust](https://rustup.rs/) (the installer will prompt you to install Visual Studio Build Tools if needed)
2. Ensure "Desktop development with C++" workload is selected in the Visual Studio Build Tools installer

```sh
cargo build
cargo run -- once
```

### Verifying a build

On any platform, these commands exercise the main code paths without running the daemon:

```sh
cargo run -- list-devices   # verify audio device enumeration
cargo run -- once           # play a single tone (confirms audio output works)
cargo run -- config         # show resolved configuration
```

## License

MIT
