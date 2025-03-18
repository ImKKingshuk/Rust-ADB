use std::process::{Command, Output};
use std::io::{self, Write};
use std::time::Duration;
use std::path::Path;
use tokio::process::Command as AsyncCommand;
use tokio::time::timeout;
use log::{debug, error, info, warn};
use backoff::{ExponentialBackoff, Error as BackoffError};

mod device;
mod error;
mod package;
mod file_ops;
mod wireless;

pub use crate::device::Device;
pub use crate::error::{ADBError, Result};
pub use crate::package::PackageInfo;

pub struct ADB {
    bin: String,
    timeout: Duration,
    backoff: ExponentialBackoff,
}

impl ADB {
    pub const BIN_LINUX: &'static str = "adb";
    pub const BIN_DARWIN: &'static str = "adb";
    pub const BIN_WINDOWS: &'static str = "adb.exe";

    pub fn new(bin_path: &str, timeout: Duration) -> Self {
        debug!("Initializing ADB with binary path: {}", bin_path);
        let bin = match std::env::consts::OS {
            "windows" => format!("{}/{}", bin_path, Self::BIN_WINDOWS),
            "macos" => format!("{}/{}", bin_path, Self::BIN_DARWIN),
            _ => format!("{}/{}", bin_path, Self::BIN_LINUX),
        };
        let mut backoff = ExponentialBackoff::default();
        backoff.max_elapsed_time = Some(timeout);
        ADB { bin, timeout, backoff }
    }

    fn exec_shell(&self, command: &str) -> Result<Output, io::Error> {
        debug!("Executing ADB command: {}", command);
        let output = Command::new(&self.bin).arg(command).output()?;
        if !output.status.success() {
            error!("Command failed: {}", String::from_utf8_lossy(&output.stderr));
        }
        Ok(output)
    }

    async fn exec_shell_async(&self, command: &str) -> Result<Output, io::Error> {
        debug!("Executing async ADB command: {}", command);
        let child = AsyncCommand::new(&self.bin)
            .arg(command)
            .output();
        match timeout(self.timeout, child).await {
            Ok(output) => {
                let output = output?;
                if !output.status.success() {
                    error!("Async command failed: {}", String::from_utf8_lossy(&output.stderr));
                }
                Ok(output)
            }
            Err(_) => {
                error!("Command timed out after {:?}", self.timeout);
                Err(io::Error::new(io::ErrorKind::TimedOut, "Command timed out"))
            }
        }
    }

    pub fn run_adb(&self, command: &str) -> Result<String, ADBError> {
        let operation = || -> Result<String, ADBError> {
            let output = self.exec_shell(command)?;
            if output.status.success() {
                Ok(String::from_utf8_lossy(&output.stdout).to_string())
            } else {
                Err(ADBError::CommandFailed(String::from_utf8_lossy(&output.stderr).to_string()))
            }
        };

        match backoff::retry(self.backoff.clone(), operation) {
            Ok(result) => Ok(result),
            Err(err) => {
                error!("Command failed after retries: {}", err);
                Err(ADBError::CommandFailed(format!("Command failed after retries: {}", err)))
            }
        }
    }

    pub async fn run_adb_async(&self, command: &str) -> Result<String, ADBError> {
        let operation = || async {
            let output = self.exec_shell_async(command).await?;
            if output.status.success() {
                Ok(String::from_utf8_lossy(&output.stdout).to_string())
            } else {
                Err(ADBError::CommandFailed(String::from_utf8_lossy(&output.stderr).to_string()))
            }
        };

        match backoff::future::retry(self.backoff.clone(), operation).await {
            Ok(result) => Ok(result),
            Err(err) => {
                error!("Async command failed after retries: {}", err);
                Err(ADBError::CommandFailed(format!("Async command failed after retries: {}", err)))
            }
        }
    }

    pub fn start_server(&self) -> Result<(), ADBError> {
        info!("Starting ADB server");
        self.run_adb("start-server")?;
        Ok(())
    }

    pub async fn start_server_async(&self) -> Result<(), ADBError> {
        info!("Starting ADB server asynchronously");
        self.run_adb_async("start-server").await?;
        Ok(())
    }

    pub fn kill_server(&self) -> Result<(), ADBError> {
        info!("Killing ADB server");
        self.run_adb("kill-server")?;
        Ok(())
    }

    pub async fn kill_server_async(&self) -> Result<(), ADBError> {
        info!("Killing ADB server asynchronously");
        self.run_adb_async("kill-server").await?;
        Ok(())
    }
}

pub struct ADB {
    bin: String,
    timeout: Duration,
    backoff: ExponentialBackoff,
}

impl ADB {
    pub const BIN_LINUX: &'static str = "adb";
    pub const BIN_DARWIN: &'static str = "adb";
    pub const BIN_WINDOWS: &'static str = "adb.exe";

    pub fn new(bin_path: &str, timeout: Duration) -> Self {
        debug!("Initializing ADB with binary path: {}", bin_path);
        let bin = match std::env::consts::OS {
            "windows" => format!("{}/{}", bin_path, Self::BIN_WINDOWS),
            "macos" => format!("{}/{}", bin_path, Self::BIN_DARWIN),
            _ => format!("{}/{}", bin_path, Self::BIN_LINUX),
        };
        let mut backoff = ExponentialBackoff::default();
        backoff.max_elapsed_time = Some(timeout);
        ADB { bin, timeout, backoff }
    }

    fn exec_shell(&self, command: &str) -> Result<Output, io::Error> {
        debug!("Executing ADB command: {}", command);
        let output = Command::new(&self.bin).arg(command).output()?;
        if !output.status.success() {
            error!("Command failed: {}", String::from_utf8_lossy(&output.stderr));
        }
        Ok(output)
    }

    async fn exec_shell_async(&self, command: &str) -> Result<Output, io::Error> {
        debug!("Executing async ADB command: {}", command);
        let child = AsyncCommand::new(&self.bin)
            .arg(command)
            .output();
        match timeout(self.timeout, child).await {
            Ok(output) => {
                let output = output?;
                if !output.status.success() {
                    error!("Async command failed: {}", String::from_utf8_lossy(&output.stderr));
                }
                Ok(output)
            }
            Err(_) => {
                error!("Command timed out after {:?}", self.timeout);
                Err(io::Error::new(io::ErrorKind::TimedOut, "Command timed out"))
            }
        }
    }

    pub fn run_adb(&self, command: &str) -> Result<String, ADBError> {
        let operation = || -> Result<String, ADBError> {
            let output = self.exec_shell(command)?;
            if output.status.success() {
                Ok(String::from_utf8_lossy(&output.stdout).to_string())
            } else {
                Err(ADBError::CommandFailed(String::from_utf8_lossy(&output.stderr).to_string()))
            }
        };

        match backoff::retry(self.backoff.clone(), operation) {
            Ok(result) => Ok(result),
            Err(err) => {
                error!("Command failed after retries: {}", err);
                Err(ADBError::CommandFailed(format!("Command failed after retries: {}", err)))
            }
        }
    }

    pub async fn run_adb_async(&self, command: &str) -> Result<String, ADBError> {
        let operation = || async {
            let output = self.exec_shell_async(command).await?;
            if output.status.success() {
                Ok(String::from_utf8_lossy(&output.stdout).to_string())
            } else {
                Err(ADBError::CommandFailed(String::from_utf8_lossy(&output.stderr).to_string()))
            }
        };

        match backoff::future::retry(self.backoff.clone(), operation).await {
            Ok(result) => Ok(result),
            Err(err) => {
                error!("Async command failed after retries: {}", err);
                Err(ADBError::CommandFailed(format!("Async command failed after retries: {}", err)))
            }
        }
    }

    pub fn start_server(&self) -> Result<(), ADBError> {
        info!("Starting ADB server");
        self.run_adb("start-server")?;
        Ok(())
    }

    pub async fn start_server_async(&self) -> Result<(), ADBError> {
        info!("Starting ADB server asynchronously");
        self.run_adb_async("start-server").await?;
        Ok(())
    }

    pub fn kill_server(&self) -> Result<(), ADBError> {
        info!("Killing ADB server");
        self.run_adb("kill-server")?;
        Ok(())
    }

    pub async fn kill_server_async(&self) -> Result<(), ADBError> {
        info!("Killing ADB server asynchronously");
        self.run_adb_async("kill-server").await?;
        Ok(())
    }
}

impl ADB {
    pub const BIN_LINUX: &'static str = "adb";
    pub const BIN_DARWIN: &'static str = "adb-darwin";
    pub const BIN_WINDOWS: &'static str = "adb.exe";

    pub fn new(bin_path: &str, timeout: Duration) -> Self {
        debug!("Initializing ADB with binary path: {}", bin_path);
        let bin = match std::env::consts::OS {
            "windows" => format!("{}\\{}", bin_path, Self::BIN_WINDOWS),
            "macos" => format!("{}/{}", bin_path, Self::BIN_DARWIN),
            _ => format!("{}/{}", bin_path, Self::BIN_LINUX),
        };
        let mut backoff = ExponentialBackoff::default();
        backoff.max_elapsed_time = Some(timeout);
        ADB { bin, timeout, backoff }
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

    pub fn refresh_device_list(&self) -> Result<Vec<Device>> {
        let result = self.run_adb("devices -l")?;
        let mut devices = Vec::new();
        for line in result.lines().skip(1) {
            if line.trim().is_empty() { continue; }
            if let Some(device) = self.parse_device_line(line) {
                devices.push(device);
            }
        }
        Ok(devices)
    }

    fn parse_device_line(&self, line: &str) -> Option<Device> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 { return None; }

        let serial = parts[0].to_string();
        let state = parts[1].to_string();
        let mut device = Device {
            serial,
            state,
            product: None,
            model: None,
            device: None,
            transport_id: None,
        };

        for part in parts.iter().skip(2) {
            let kv: Vec<&str> = part.split(':').collect();
            if kv.len() != 2 { continue; }
            match kv[0] {
                "product" => device.product = Some(kv[1].to_string()),
                "model" => device.model = Some(kv[1].to_string()),
                "device" => device.device = Some(kv[1].to_string()),
                "transport_id" => device.transport_id = Some(kv[1].to_string()),
                _ => {}
            }
        }
        Some(device)
    }

    pub async fn refresh_device_list_async(&self) -> Result<Vec<Device>> {
        let result = self.run_adb_async("devices -l").await?;
        let mut devices = Vec::new();
        for line in result.lines().skip(1) {
            if line.trim().is_empty() { continue; }
            if let Some(device) = self.parse_device_line(line) {
                devices.push(device);
            }
        }
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

    pub fn push_file(&self, device: &str, local_path: &str, remote_path: &str) -> Result<()> {
        self.run_adb(&format!("-s {} push {} {}", device, local_path, remote_path))?;
        Ok(())
    }

    pub async fn push_file_async(&self, device: &str, local_path: &str, remote_path: &str) -> Result<()> {
        self.run_adb_async(&format!("-s {} push {} {}", device, local_path, remote_path)).await?;
        Ok(())
    }

    pub fn pull_file(&self, device: &str, remote_path: &str, local_path: &str) -> Result<()> {
        self.run_adb(&format!("-s {} pull {} {}", device, remote_path, local_path))?;
        Ok(())
    }

    pub async fn pull_file_async(&self, device: &str, remote_path: &str, local_path: &str) -> Result<()> {
        self.run_adb_async(&format!("-s {} pull {} {}", device, remote_path, local_path)).await?;
        Ok(())
    }

    pub fn shell_command(&self, device: &str, command: &str) -> Result<String> {
        self.run_adb(&format!("-s {} shell {}", device, command))
    }

    pub async fn shell_command_async(&self, device: &str, command: &str) -> Result<String> {
        self.run_adb_async(&format!("-s {} shell {}", device, command)).await
    }

    pub fn connect_wireless(&self, ip: &str, port: u16) -> Result<()> {
        self.run_adb(&format!("connect {}:{}", ip, port))?;
        Ok(())
    }

    pub async fn connect_wireless_async(&self, ip: &str, port: u16) -> Result<()> {
        self.run_adb_async(&format!("connect {}:{}", ip, port)).await?;
        Ok(())
    }

    pub fn disconnect_wireless(&self, ip: &str, port: u16) -> Result<()> {
        self.run_adb(&format!("disconnect {}:{}", ip, port))?;
        Ok(())
    }

    pub async fn disconnect_wireless_async(&self, ip: &str, port: u16) -> Result<()> {
        self.run_adb_async(&format!("disconnect {}:{}", ip, port)).await?;
        Ok(())
    }

    pub fn get_package_list(&self, device: &str) -> Result<Vec<PackageInfo>> {
        let output = self.run_adb(&format!("-s {} shell pm list packages -f -i", device))?;
        let mut packages = Vec::new();
        for line in output.lines() {
            if let Some(package) = self.parse_package_line(line) {
                packages.push(package);
            }
        }
        Ok(packages)
    }

    pub async fn get_package_list_async(&self, device: &str) -> Result<Vec<PackageInfo>> {
        let output = self.run_adb_async(&format!("-s {} shell pm list packages -f -i", device)).await?;
        let mut packages = Vec::new();
        for line in output.lines() {
            if let Some(package) = self.parse_package_line(line) {
                packages.push(package);
            }
        }
        Ok(packages)
    }

    fn parse_package_line(&self, line: &str) -> Option<PackageInfo> {
        let parts: Vec<&str> = line.split('=').collect();
        if parts.len() != 2 { return None; }

        let name = parts[1].to_string();
        Some(PackageInfo {
            name,
            version_code: String::new(),
            version_name: String::new(),
            is_system: false,
        })
    }

    pub fn get_device_props(&self, device: &str) -> Result<String> {
        self.run_adb(&format!("-s {} shell getprop", device))
    }

    pub async fn get_device_props_async(&self, device: &str) -> Result<String> {
        self.run_adb_async(&format!("-s {} shell getprop", device)).await
    }

    pub fn set_device_prop(&self, device: &str, prop: &str, value: &str) -> Result<()> {
        self.run_adb(&format!("-s {} shell setprop {} {}", device, prop, value))?;
        Ok(())
    }

    pub async fn set_device_prop_async(&self, device: &str, prop: &str, value: &str) -> Result<()> {
        self.run_adb_async(&format!("-s {} shell setprop {} {}", device, prop, value)).await?;
        Ok(())
    }
}

