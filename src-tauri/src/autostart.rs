const APP_NAME: &str = "LocalGitIssues";

#[cfg(target_os = "windows")]
const RUN_KEY: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

pub fn is_autostart_enabled() -> bool {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        let mut cmd = std::process::Command::new("reg");
        cmd.args(["query", RUN_KEY, "/v", APP_NAME]);
        cmd.creation_flags(CREATE_NO_WINDOW);
        cmd.output().map(|o| o.status.success()).unwrap_or(false)
    }
    #[cfg(target_os = "macos")]
    {
        if let Some(home) = dirs_or_home() {
            home.join("Library/LaunchAgents/com.local.git.issues.plist").exists()
        } else {
            false
        }
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        if let Some(home) = dirs_or_home() {
            home.join(".config/autostart/local-git-issues.desktop").exists()
        } else {
            false
        }
    }
}

pub fn set_autostart_enabled(enabled: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        let mut cmd = std::process::Command::new("reg");
        cmd.creation_flags(CREATE_NO_WINDOW);
        if enabled {
            let exe = std::env::current_exe().map_err(|e| e.to_string())?;
            let exe_str = format!("\"{}\" --minimized", exe.to_string_lossy());
            cmd.args(["add", RUN_KEY, "/v", APP_NAME, "/t", "REG_SZ", "/d", &exe_str, "/f"]);
            let output = cmd.output().map_err(|e| e.to_string())?;
            if !output.status.success() {
                return Err("Failed to enable autostart".into());
            }
        } else {
            cmd.args(["delete", RUN_KEY, "/v", APP_NAME, "/f"]);
            let _ = cmd.output();
        }
        Ok(())
    }
    #[cfg(target_os = "macos")]
    {
        let home = dirs_or_home().ok_or("Could not find home directory")?;
        let dir = home.join("Library/LaunchAgents");
        let plist_path = dir.join("com.local.git.issues.plist");
        if enabled {
            std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
            let exe = std::env::current_exe().map_err(|e| e.to_string())?;
            let content = format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
                <!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
                <plist version="1.0">
                <dict>
                    <key>Label</key>
                    <string>com.local.git.issues</string>
                    <key>ProgramArguments</key>
                    <array>
                        <string>{}</string>
                        <string>--minimized</string>
                    </array>
                    <key>RunAtLoad</key>
                    <true/>
                </dict>
                </plist>"#,
                exe.to_string_lossy()
            );
            std::fs::write(plist_path, content).map_err(|e| e.to_string())?;
        } else if plist_path.exists() {
            let _ = std::fs::remove_file(plist_path);
        }
        Ok(())
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        let home = dirs_or_home().ok_or("Could not find home directory")?;
        let dir = home.join(".config/autostart");
        let desktop_path = dir.join("local-git-issues.desktop");
        if enabled {
            std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
            let exe = std::env::current_exe().map_err(|e| e.to_string())?;
            let content = format!(
                "[Desktop Entry]\nType=Application\nName=Local Git Issues\nExec=\"{}\" --minimized\nHidden=false\nNoDisplay=false\nX-GNOME-Autostart-enabled=true\n",
                exe.to_string_lossy()
            );
            std::fs::write(desktop_path, content).map_err(|e| e.to_string())?;
        } else if desktop_path.exists() {
            let _ = std::fs::remove_file(desktop_path);
        }
        Ok(())
    }
}

#[cfg(not(target_os = "windows"))]
fn dirs_or_home() -> Option<std::path::PathBuf> {
    std::env::var_os("HOME").map(std::path::PathBuf::from)
}