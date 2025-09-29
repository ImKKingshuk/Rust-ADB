use std::process::{Command, Output};
use std::time::Duration;
use tokio::process::Command as AsyncCommand;
use tokio::time::timeout;
use log::{debug, error, info, warn};
use backoff::ExponentialBackoff;

mod device;
mod error;
mod package;
mod file_ops;
mod wireless;
mod debug;
mod input;
mod device_management;
mod screen_recording;
mod system_info;
mod automation;
// mod advanced_device;

pub use crate::debug::LogcatOptions;
pub use crate::debug::LogcatPreset;
pub use crate::debug::PerformanceProfile;
pub use crate::debug::NetworkStats;
pub use crate::input::{InputSource, TouchEvent};
pub use crate::wireless::NetworkDiagnostics;
pub use crate::device_management::{AppPermissions, ProcessInfo, AppDataSize};

pub use crate::device::Device;
pub use crate::error::{ADBError, Result};
pub use crate::package::PackageInfo;
pub use crate::system_info::{SystemInfo, BatteryInfo};
pub use crate::screen_recording::ScreenRecordOptions;
// pub use crate::advanced_device::{StorageInfo, MemoryInfo, NetworkInfo};

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
            "windows" => format!("{}\\{}", bin_path, Self::BIN_WINDOWS),
            "macos" => format!("{}/{}", bin_path, Self::BIN_DARWIN),
            _ => format!("{}/{}", bin_path, Self::BIN_LINUX),
        };
        let mut backoff = ExponentialBackoff::default();
        backoff.max_elapsed_time = Some(timeout);
        ADB { bin, timeout, backoff }
    }

    fn exec_shell(&self, command: &str) -> Result<Output> {
        debug!("Executing ADB command: {}", command);
        let output = Command::new(&self.bin).arg(command).output()?;
        if !output.status.success() {
            error!("Command failed: {}", String::from_utf8_lossy(&output.stderr));
        }
        Ok(output)
    }

    async fn exec_shell_async(&self, command: &str) -> Result<Output> {
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
                Err(ADBError::Timeout(format!("Command timed out after {:?}", self.timeout)))
            }
        }
    }

    pub fn run_adb(&self, command: &str) -> Result<String> {
        let mut attempt = 0;
        loop {
            match self.exec_shell(command) {
                Ok(output) => {
                    if output.status.success() {
                        return Ok(String::from_utf8_lossy(&output.stdout).to_string());
                    } else {
                        let err = ADBError::CommandFailed(String::from_utf8_lossy(&output.stderr).to_string());
                        if attempt >= 2 {
                            return Err(err);
                        }
                        attempt += 1;
                        std::thread::sleep(std::time::Duration::from_millis(100 * attempt as u64));
                        continue;
                    }
                }
                Err(e) => {
                    if attempt >= 2 {
                        return Err(e);
                    }
                    attempt += 1;
                    std::thread::sleep(std::time::Duration::from_millis(100 * attempt as u64));
                    continue;
                }
            }
        }
    }

    pub async fn run_adb_async(&self, command: &str) -> Result<String> {
        let mut attempt = 0;
        loop {
            match self.exec_shell_async(command).await {
                Ok(output) => {
                    if output.status.success() {
                        return Ok(String::from_utf8_lossy(&output.stdout).to_string());
                    } else {
                        let err = ADBError::CommandFailed(String::from_utf8_lossy(&output.stderr).to_string());
                        if attempt >= 2 {
                            return Err(err);
                        }
                        attempt += 1;
                        tokio::time::sleep(tokio::time::Duration::from_millis(100 * attempt as u64)).await;
                        continue;
                    }
                }
                Err(e) => {
                    if attempt >= 2 {
                        return Err(e);
                    }
                    attempt += 1;
                    tokio::time::sleep(tokio::time::Duration::from_millis(100 * attempt as u64)).await;
                    continue;
                }
            }
        }
    }

    pub fn start_server(&self) -> Result<()> {
        info!("Starting ADB server");
        self.run_adb("start-server")?;
        Ok(())
    }

    pub async fn start_server_async(&self) -> Result<()> {
        info!("Starting ADB server asynchronously");
        self.run_adb_async("start-server").await?;
        Ok(())
    }

    pub fn kill_server(&self) -> Result<()> {
        info!("Killing ADB server");
        self.run_adb("kill-server")?;
        Ok(())
    }

    pub async fn kill_server_async(&self) -> Result<()> {
        info!("Killing ADB server asynchronously");
        self.run_adb_async("kill-server").await?;
        Ok(())
    }

    /// Force kill ADB server process (Windows only)
    pub fn kill_server_force(&self) -> Result<()> {
        if std::env::consts::OS == "windows" {
            info!("Force killing ADB server on Windows");
            let output = Command::new("taskkill").args(&["/f", "/im", &self.bin]).output()?;
            if !output.status.success() {
                return Err(ADBError::CommandFailed(String::from_utf8_lossy(&output.stderr).to_string()));
            }
            Ok(())
        } else {
            warn!("Force kill not supported on non-Windows platforms, using normal kill");
            self.kill_server()
        }
    }

    /// Get ADB version
    pub fn version(&self) -> Result<String> {
        self.run_adb("version")
    }

    /// Get ADB version (async)
    pub async fn version_async(&self) -> Result<String> {
        self.run_adb_async("version").await
    }
}
