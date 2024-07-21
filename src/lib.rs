// src/lib.rs

use std::process::{Command, Output};
use std::io;
use std::time::Duration;
use tokio::process::Command as AsyncCommand;
use tokio::time::timeout;

pub struct ADB {
    bin: String,
    timeout: Duration,
}

impl ADB {
    pub const BIN_LINUX: &'static str = "adb";
    pub const BIN_DARWIN: &'static str = "adb-darwin";
    pub const BIN_WINDOWS: &'static str = "adb.exe";

    pub fn new(bin_path: &str, timeout: Duration) -> Self {
        let bin = match std::env::consts::OS {
            "windows" => format!("{}\\{}", bin_path, Self::BIN_WINDOWS),
            "macos" => format!("{}/{}", bin_path, Self::BIN_DARWIN),
            _ => format!("{}/{}", bin_path, Self::BIN_LINUX),
        };
        ADB { bin, timeout }
    }

    fn exec_shell(&self, command: &str) -> Result<Output, io::Error> {
        Command::new(&self.bin).arg(command).output()
    }

    async fn exec_shell_async(&self, command: &str) -> Result<Output, io::Error> {
        let child = AsyncCommand::new(&self.bin)
            .arg(command)
            .output();
        timeout(self.timeout, child).await?
    }

    pub fn run_adb(&self, command: &str) -> Result<String, io::Error> {
        let output = self.exec_shell(command)?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Command failed: {}", String::from_utf8_lossy(&output.stderr)),
            ))
        }
    }

    pub async fn run_adb_async(&self, command: &str) -> Result<String, io::Error> {
        let output = self.exec_shell_async(command).await?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Command failed: {}", String::from_utf8_lossy(&output.stderr)),
            ))
        }
    }

    pub fn refresh_device_list(&self) -> Result<Vec<String>, io::Error> {
        let result = self.run_adb("devices -l")?;
        let devices: Vec<String> = result
            .lines()
            .skip(1)
            .map(|line| line.trim().to_string())
            .collect();
        Ok(devices)
    }

    pub async fn refresh_device_list_async(&self) -> Result<Vec<String>, io::Error> {
        let result = self.run_adb_async("devices -l").await?;
        let devices: Vec<String> = result
            .lines()
            .skip(1)
            .map(|line| line.trim().to_string())
            .collect();
        Ok(devices)
    }

    pub fn start_server(&self) -> Result<(), io::Error> {
        self.run_adb("start-server")?;
        Ok(())
    }

    pub async fn start_server_async(&self) -> Result<(), io::Error> {
        self.run_adb_async("start-server").await?;
        Ok(())
    }

    pub fn kill_server(&self, force: bool) -> Result<(), io::Error> {
        if force {
            if std::env::consts::OS != "windows" {
                eprintln!("Force termination is not implemented on non-Windows systems, fallback to normal.");
            } else {
                self.exec_shell(&format!("taskkill /f /im {}", self.bin))?;
            }
        } else {
            self.run_adb("kill-server")?;
        }
        Ok(())
    }

    pub async fn kill_server_async(&self, force: bool) -> Result<(), io::Error> {
        if force {
            if std::env::consts::OS != "windows" {
                eprintln!("Force termination is not implemented on non-Windows systems, fallback to normal.");
            } else {
                self.exec_shell_async(&format!("taskkill /f /im {}", self.bin)).await?;
            }
        } else {
            self.run_adb_async("kill-server").await?;
        }
        Ok(())
    }

    pub fn get_screenshot_png(&self, device: &str) -> Result<Vec<u8>, io::Error> {
        let output = self.run_adb(&format!("{} exec-out screencap -p", device))?;
        Ok(output.into_bytes())
    }

    pub async fn get_screenshot_png_async(&self, device: &str) -> Result<Vec<u8>, io::Error> {
        let output = self.run_adb_async(&format!("{} exec-out screencap -p", device)).await?;
        Ok(output.into_bytes())
    }

    pub fn install_app(&self, device: &str, apk_path: &str) -> Result<String, io::Error> {
        self.run_adb(&format!("{} install {}", device, apk_path))
    }

    pub async fn install_app_async(&self, device: &str, apk_path: &str) -> Result<String, io::Error> {
        self.run_adb_async(&format!("{} install {}", device, apk_path)).await
    }

    pub fn uninstall_app(&self, device: &str, package_name: &str) -> Result<String, io::Error> {
        self.run_adb(&format!("{} uninstall {}", device, package_name))
    }

    pub async fn uninstall_app_async(&self, device: &str, package_name: &str) -> Result<String, io::Error> {
        self.run_adb_async(&format!("{} uninstall {}", device, package_name)).await
    }

    pub fn logcat(&self, device: &str) -> Result<String, io::Error> {
        self.run_adb(&format!("{} logcat", device))
    }

    pub async fn logcat_async(&self, device: &str) -> Result<String, io::Error> {
        self.run_adb_async(&format!("{} logcat", device)).await
    }
}

