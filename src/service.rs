use std::path::PathBuf;

#[cfg(target_os = "macos")]
const LAUNCHD_LABEL: &str = "com.nodoze.daemon";
#[cfg(target_os = "linux")]
const SYSTEMD_SERVICE: &str = "nodoze";

/// Install nodoze as a system service
pub fn install() -> Result<(), String> {
    let exe = std::env::current_exe()
        .map_err(|e| format!("Failed to get executable path: {}", e))?;

    #[cfg(target_os = "macos")]
    return install_launchd(&exe);

    #[cfg(target_os = "linux")]
    return install_systemd(&exe);

    #[cfg(target_os = "windows")]
    return install_windows_task(&exe);

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    return Err("Service installation not supported on this platform".to_string());
}

/// Uninstall nodoze system service
pub fn uninstall() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    return uninstall_launchd();

    #[cfg(target_os = "linux")]
    return uninstall_systemd();

    #[cfg(target_os = "windows")]
    return uninstall_windows_task();

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    return Err("Service uninstallation not supported on this platform".to_string());
}

// ── macOS LaunchAgent ──────────────────────────────────────────────

#[cfg(target_os = "macos")]
fn launchd_plist_path() -> Result<PathBuf, String> {
    let home = dirs::home_dir().ok_or("Could not determine home directory")?;
    Ok(home
        .join("Library")
        .join("LaunchAgents")
        .join(format!("{}.plist", LAUNCHD_LABEL)))
}

#[cfg(target_os = "macos")]
fn install_launchd(exe: &PathBuf) -> Result<(), String> {
    let plist_path = launchd_plist_path()?;

    if let Some(parent) = plist_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create LaunchAgents directory: {}", e))?;
    }

    let exe_str = exe.to_string_lossy();

    let plist = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
  <dict>
    <key>Label</key>
    <string>{label}</string>
    <key>ProgramArguments</key>
    <array>
      <string>{exe}</string>
      <string>run</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardErrorPath</key>
    <string>/tmp/nodoze.err</string>
    <key>StandardOutPath</key>
    <string>/tmp/nodoze.out</string>
  </dict>
</plist>"#,
        label = LAUNCHD_LABEL,
        exe = exe_str,
    );

    std::fs::write(&plist_path, plist)
        .map_err(|e| format!("Failed to write plist: {}", e))?;

    let status = std::process::Command::new("launchctl")
        .args(["load", "-w"])
        .arg(&plist_path)
        .status()
        .map_err(|e| format!("Failed to run launchctl: {}", e))?;

    if status.success() {
        println!("Service installed and started: {}", plist_path.display());
        Ok(())
    } else {
        Err("launchctl load failed".to_string())
    }
}

#[cfg(target_os = "macos")]
fn uninstall_launchd() -> Result<(), String> {
    let plist_path = launchd_plist_path()?;

    if plist_path.exists() {
        let _ = std::process::Command::new("launchctl")
            .args(["unload"])
            .arg(&plist_path)
            .status();

        std::fs::remove_file(&plist_path)
            .map_err(|e| format!("Failed to remove plist: {}", e))?;

        println!("Service uninstalled: {}", plist_path.display());
    } else {
        println!("Service not installed (plist not found)");
    }

    Ok(())
}

// ── Linux systemd ──────────────────────────────────────────────────

#[cfg(target_os = "linux")]
fn systemd_unit_path() -> Result<PathBuf, String> {
    let home = dirs::home_dir().ok_or("Could not determine home directory")?;
    Ok(home
        .join(".config")
        .join("systemd")
        .join("user")
        .join(format!("{}.service", SYSTEMD_SERVICE)))
}

#[cfg(target_os = "linux")]
fn install_systemd(exe: &PathBuf) -> Result<(), String> {
    let unit_path = systemd_unit_path()?;
    let exe_str = exe.to_string_lossy();

    if let Some(parent) = unit_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create systemd directory: {}", e))?;
    }

    let unit = format!(
        r#"[Unit]
Description=NoDoze - Keep speakers alive with inaudible tones
After=sound.target

[Service]
Type=simple
ExecStart={exe} run
Restart=on-failure
RestartSec=10

[Install]
WantedBy=default.target
"#,
        exe = exe_str,
    );

    std::fs::write(&unit_path, unit)
        .map_err(|e| format!("Failed to write unit file: {}", e))?;

    let reload = std::process::Command::new("systemctl")
        .args(["--user", "daemon-reload"])
        .status()
        .map_err(|e| format!("Failed to reload systemd: {}", e))?;

    if !reload.success() {
        return Err("systemctl daemon-reload failed".to_string());
    }

    let enable = std::process::Command::new("systemctl")
        .args(["--user", "enable", "--now", SYSTEMD_SERVICE])
        .status()
        .map_err(|e| format!("Failed to enable service: {}", e))?;

    if enable.success() {
        println!("Service installed and started: {}", unit_path.display());
        Ok(())
    } else {
        Err("systemctl enable failed".to_string())
    }
}

#[cfg(target_os = "linux")]
fn uninstall_systemd() -> Result<(), String> {
    let unit_path = systemd_unit_path()?;

    let _ = std::process::Command::new("systemctl")
        .args(["--user", "disable", "--now", SYSTEMD_SERVICE])
        .status();

    if unit_path.exists() {
        std::fs::remove_file(&unit_path)
            .map_err(|e| format!("Failed to remove unit file: {}", e))?;

        let _ = std::process::Command::new("systemctl")
            .args(["--user", "daemon-reload"])
            .status();

        println!("Service uninstalled: {}", unit_path.display());
    } else {
        println!("Service not installed (unit file not found)");
    }

    Ok(())
}

// ── Windows Task Scheduler ─────────────────────────────────────────

#[cfg(target_os = "windows")]
fn install_windows_task(exe: &PathBuf) -> Result<(), String> {
    let exe_str = exe.to_string_lossy();

    let status = std::process::Command::new("schtasks")
        .args([
            "/Create",
            "/SC", "ONLOGON",
            "/TN", "NoDoze",
            "/TR", &format!("\"{}\" run", exe_str),
            "/F",
        ])
        .status()
        .map_err(|e| format!("Failed to run schtasks: {}", e))?;

    if status.success() {
        // Also start it immediately
        let _ = std::process::Command::new("schtasks")
            .args(["/Run", "/TN", "NoDoze"])
            .status();

        println!("Service installed as scheduled task: NoDoze");
        Ok(())
    } else {
        Err("schtasks /Create failed".to_string())
    }
}

#[cfg(target_os = "windows")]
fn uninstall_windows_task() -> Result<(), String> {
    // End any running instance first
    let _ = std::process::Command::new("schtasks")
        .args(["/End", "/TN", "NoDoze"])
        .status();

    let status = std::process::Command::new("schtasks")
        .args(["/Delete", "/TN", "NoDoze", "/F"])
        .status()
        .map_err(|e| format!("Failed to run schtasks: {}", e))?;

    if status.success() {
        println!("Service uninstalled: NoDoze");
        Ok(())
    } else {
        Err("schtasks /Delete failed (task may not exist)".to_string())
    }
}
