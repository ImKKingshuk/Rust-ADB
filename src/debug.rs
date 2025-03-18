use crate::error::ADBError;
use crate::ADB;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug)]
pub struct LogcatOptions {
    pub buffer: Option<String>,
    pub format: Option<String>,
    pub filters: Vec<String>,
    pub clear: bool,
    pub dump: bool,
}

impl Default for LogcatOptions {
    fn default() -> Self {
        Self {
            buffer: None,
            format: Some("time".to_string()),
            filters: Vec::new(),
            clear: false,
            dump: false,
        }
    }
}

impl ADB {
    pub fn start_debug(&self, device: &str, process: &str) -> Result<(), ADBError> {
        let output = self.run_adb(&format!("-s {} shell ps | grep {}", device, process))?;
        let pid = output
            .split_whitespace()
            .nth(1)
            .ok_or_else(|| ADBError::Debug(format!("Process {} not found", process)))?;

        self.run_adb(&format!("-s {} forward tcp:8700 jdwp:{}", device, pid))?;
        Ok(())
    }

    pub async fn start_debug_async(&self, device: &str, process: &str) -> Result<(), ADBError> {
        let output = self.run_adb_async(&format!("-s {} shell ps | grep {}", device, process)).await?;
        let pid = output
            .split_whitespace()
            .nth(1)
            .ok_or_else(|| ADBError::Debug(format!("Process {} not found", process)))?;

        self.run_adb_async(&format!("-s {} forward tcp:8700 jdwp:{}", device, pid)).await?;
        Ok(())
    }

    pub fn stop_debug(&self, device: &str) -> Result<(), ADBError> {
        self.run_adb(&format!("-s {} forward --remove tcp:8700", device))?;
        Ok(())
    }

    pub async fn stop_debug_async(&self, device: &str) -> Result<(), ADBError> {
        self.run_adb_async(&format!("-s {} forward --remove tcp:8700", device)).await?;
        Ok(())
    }

    pub fn watch_logcat(&self, device: &str, options: LogcatOptions) -> Result<(), ADBError> {
        let mut cmd = format!("-s {} logcat", device);

        if let Some(buffer) = options.buffer {
            cmd.push_str(&format!(" -b {}", buffer));
        }

        if let Some(format) = options.format {
            cmd.push_str(&format!(" -v {}", format));
        }

        if options.clear {
            self.run_adb(&format!("-s {} logcat -c", device))?;
        }

        if options.dump {
            cmd.push_str(" -d");
        }

        for filter in options.filters {
            cmd.push_str(&format!(" {}", filter));
        }

        self.run_adb(&cmd)?;
        Ok(())
    }

    pub async fn watch_logcat_async(&self, device: &str, options: LogcatOptions) -> Result<(), ADBError> {
        let mut cmd = format!("-s {} logcat", device);

        if let Some(buffer) = options.buffer {
            cmd.push_str(&format!(" -b {}", buffer));
        }

        if let Some(format) = options.format {
            cmd.push_str(&format!(" -v {}", format));
        }

        if options.clear {
            self.run_adb_async(&format!("-s {} logcat -c", device)).await?;
        }

        if options.dump {
            cmd.push_str(" -d");
        }

        for filter in options.filters {
            cmd.push_str(&format!(" {}", filter));
        }

        self.run_adb_async(&cmd).await?;
        Ok(())
    }

    pub fn wait_for_device(&self, device: &str, timeout: Duration) -> Result<(), ADBError> {
        let start_time = std::time::Instant::now();
        while start_time.elapsed() < timeout {
            let devices = self.refresh_device_list()?;
            if devices.iter().any(|d| d.serial == device) {
                return Ok(());
            }
            std::thread::sleep(Duration::from_secs(1));
        }
        Err(ADBError::Timeout(format!("Device {} not found after {:?}", device, timeout)))
    }

    pub async fn wait_for_device_async(&self, device: &str, timeout: Duration) -> Result<(), ADBError> {
        let start_time = std::time::Instant::now();
        while start_time.elapsed() < timeout {
            let devices = self.refresh_device_list_async().await?;
            if devices.iter().any(|d| d.serial == device) {
                return Ok(());
            }
            sleep(Duration::from_secs(1)).await;
        }
        Err(ADBError::Timeout(format!("Device {} not found after {:?}", device, timeout)))
    }
}