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

#[derive(Debug)]
pub struct LogcatPreset {
    pub name: String,
    pub description: String,
    pub options: LogcatOptions,
}

impl LogcatPreset {
    pub fn error_only() -> Self {
        Self {
            name: "errors".to_string(),
            description: "Show only error messages".to_string(),
            options: LogcatOptions {
                filters: vec!["*:E".to_string()],
                format: Some("time".to_string()),
                ..Default::default()
            },
        }
    }

    pub fn app_specific(package: &str) -> Self {
        Self {
            name: format!("app_{}", package),
            description: format!("Show logs for package: {}", package),
            options: LogcatOptions {
                filters: vec![package.to_string()],
                format: Some("time".to_string()),
                ..Default::default()
            },
        }
    }

    pub fn system_performance() -> Self {
        Self {
            name: "performance".to_string(),
            description: "Show system performance related logs".to_string(),
            options: LogcatOptions {
                buffer: Some("system".to_string()),
                filters: vec![
                    "ActivityManager:I".to_string(),
                    "WindowManager:I".to_string(),
                    "SystemUI:I".to_string(),
                ],
                format: Some("time".to_string()),
                ..Default::default()
            },
        }
    }

    pub fn network_debug() -> Self {
        Self {
            name: "network".to_string(),
            description: "Show network related logs".to_string(),
            options: LogcatOptions {
                filters: vec![
                    "ConnectivityService:D".to_string(),
                    "NetworkManagement:D".to_string(),
                    "WifiStateMachine:D".to_string(),
                ],
                format: Some("time".to_string()),
                ..Default::default()
            },
        }
    }
}

#[derive(Debug)]
pub struct PerformanceProfile {
    pub cpu_usage: f32,
    pub memory_usage: u64,
    pub battery_level: i32,
    pub network_stats: NetworkStats,
}

#[derive(Debug)]
pub struct NetworkStats {
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub rx_packets: u64,
    pub tx_packets: u64,
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

    /// Use a predefined logcat preset
    pub fn watch_logcat_preset(&self, device: &str, preset: LogcatPreset) -> Result<(), ADBError> {
        self.watch_logcat(device, preset.options)
    }

    /// Get available logcat presets
    pub fn get_logcat_presets() -> Vec<LogcatPreset> {
        vec![
            LogcatPreset::error_only(),
            LogcatPreset::system_performance(),
            LogcatPreset::network_debug(),
        ]
    }

    /// Capture memory dump (heap dump)
    pub fn capture_memory_dump(&self, device: &str, package_name: &str, output_path: &str) -> Result<(), ADBError> {
        // Get process ID first
        let pid_output = self.run_adb(&format!("-s {} shell pidof {}", device, package_name))?;
        let pid = pid_output.trim().parse::<u32>().map_err(|_| {
            ADBError::Debug(format!("Could not find process ID for package: {}", package_name))
        })?;

        // Capture heap dump
        self.run_adb(&format!("-s {} shell am dumpheap {} {}", device, pid, output_path))?;
        Ok(())
    }

    /// Get performance profile
    pub fn get_performance_profile(&self, device: &str) -> Result<PerformanceProfile, ADBError> {
        let cpu_output = self.run_adb(&format!("-s {} shell dumpsys cpuinfo", device))?;
        let mem_output = self.run_adb(&format!("-s {} shell dumpsys meminfo", device))?;
        let battery_output = self.run_adb(&format!("-s {} shell dumpsys battery", device))?;
        let net_output = self.run_adb(&format!("-s {} shell cat /proc/net/dev", device))?;

        let cpu_usage = self.parse_cpu_usage(&cpu_output);
        let memory_usage = self.parse_memory_usage(&mem_output);
        let battery_level = self.parse_battery_level(&battery_output);
        let network_stats = self.parse_network_stats(&net_output);

        Ok(PerformanceProfile {
            cpu_usage,
            memory_usage,
            battery_level,
            network_stats,
        })
    }

    /// Start method profiling
    pub fn start_method_profiling(&self, device: &str, package_name: &str) -> Result<(), ADBError> {
        self.run_adb(&format!("-s {} shell am profile start {} /sdcard/profile.trace", device, package_name))?;
        Ok(())
    }

    /// Stop method profiling
    pub fn stop_method_profiling(&self, device: &str, package_name: &str) -> Result<(), ADBError> {
        self.run_adb(&format!("-s {} shell am profile stop {}", device, package_name))?;
        Ok(())
    }

    /// Capture system trace
    pub fn capture_system_trace(&self, device: &str, duration_secs: u32, output_path: &str) -> Result<(), ADBError> {
        // Enable tracing
        self.run_adb(&format!("-s {} shell setprop debug.atrace.tags.enabled 1", device))?;
        self.run_adb(&format!("-s {} shell atrace --async_start -b 16000 -t {} -c", device, duration_secs))?;

        // Wait for trace to complete
        std::thread::sleep(std::time::Duration::from_secs(duration_secs as u64 + 1));

        // Pull trace file
        self.pull_file(device, "/sdcard/atrace.trace", output_path)?;

        Ok(())
    }

    /// Get ANR traces
    pub fn get_anr_traces(&self, device: &str) -> Result<String, ADBError> {
        self.run_adb(&format!("-s {} shell cat /data/anr/traces.txt", device))
    }

    /// Clear ANR traces
    pub fn clear_anr_traces(&self, device: &str) -> Result<(), ADBError> {
        self.run_adb(&format!("-s {} shell rm -f /data/anr/traces.txt", device))?;
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

    // Helper parsing methods
    fn parse_cpu_usage(&self, output: &str) -> f32 {
        // Simplified CPU usage parsing
        if let Some(line) = output.lines().find(|l| l.contains("TOTAL")) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 9 {
                parts[8].trim_end_matches('%').parse().unwrap_or(0.0)
            } else {
                0.0
            }
        } else {
            0.0
        }
    }

    fn parse_memory_usage(&self, output: &str) -> u64 {
        // Simplified memory parsing - look for total used memory
        if let Some(line) = output.lines().find(|l| l.contains("Used RAM:")) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 5 {
                // Extract number before "K" or "M"
                if let Some(mem_str) = parts[4].split('K').next() {
                    mem_str.parse().unwrap_or(0)
                } else {
                    0
                }
            } else {
                0
            }
        } else {
            0
        }
    }

    fn parse_battery_level(&self, output: &str) -> i32 {
        for line in output.lines() {
            if line.contains("level:") {
                if let Some(level_str) = line.split(':').nth(1) {
                    return level_str.trim().parse().unwrap_or(0);
                }
            }
        }
        0
    }

    fn parse_network_stats(&self, output: &str) -> NetworkStats {
        let mut stats = NetworkStats {
            rx_bytes: 0,
            tx_bytes: 0,
            rx_packets: 0,
            tx_packets: 0,
        };

        // Find eth0 or wlan0 interface stats
        for line in output.lines() {
            if line.contains("eth0:") || line.contains("wlan0:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 10 {
                    stats.rx_bytes = parts[1].parse().unwrap_or(0);
                    stats.rx_packets = parts[2].parse().unwrap_or(0);
                    stats.tx_bytes = parts[9].parse().unwrap_or(0);
                    stats.tx_packets = parts[10].parse().unwrap_or(0);
                }
                break;
            }
        }

        stats
    }
}