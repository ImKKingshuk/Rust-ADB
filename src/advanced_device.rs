use crate::error::ADBError;
use crate::ADB;
use std::path::Path;

/// Advanced device operations for device mirroring, backup/restore, etc.
impl ADB {
    /// Enable device mirroring (screen mirroring to a local display)
    pub fn enable_device_mirroring(&self, device: &str, display_id: Option<i32>) -> Result<(), ADBError> {
        let display = display_id.unwrap_or(0);
        self.run_adb(&format!("-s {} shell am start -n com.android.systemui/.screenrecord.ScreenRecordService --ei display {}", device, display))?;
        Ok(())
    }

    /// Disable device mirroring
    pub fn disable_device_mirroring(&self, device: &str) -> Result<(), ADBError> {
        self.run_adb(&format!("-s {} shell am force-stop com.android.systemui", device))?;
        Ok(())
    }

    /// Create a full device backup
    pub fn create_device_backup(&self, device: &str, output_path: &str, include_apks: bool, include_shared: bool) -> Result<(), ADBError> {
        let mut cmd = format!("-s {} backup", device);

        if !include_apks {
            cmd.push_str(" -noapk");
        }

        if !include_shared {
            cmd.push_str(" -noshared");
        }

        cmd.push_str(&format!(" -f {}", output_path));

        // Note: ADB backup requires user interaction, this is a basic implementation
        self.run_adb(&cmd)?;
        Ok(())
    }

    /// Restore device from backup
    pub fn restore_device_backup(&self, device: &str, backup_path: &str) -> Result<(), ADBError> {
        if !Path::new(backup_path).exists() {
            return Err(ADBError::FileTransfer(format!("Backup file not found: {}", backup_path)));
        }

        self.run_adb(&format!("-s {} restore {}", device, backup_path))?;
        Ok(())
    }

    /// Get device storage information
    pub fn get_device_storage_info(&self, device: &str) -> Result<StorageInfo, ADBError> {
        let output = self.run_adb(&format!("-s {} shell df", device))?;
        self.parse_storage_info(&output)
    }

    /// Get device memory information
    pub fn get_device_memory_info(&self, device: &str) -> Result<MemoryInfo, ADBError> {
        let output = self.run_adb(&format!("-s {} shell cat /proc/meminfo", device))?;
        self.parse_memory_info(&output)
    }

    /// Set device animation scale (for development/testing)
    pub fn set_animation_scale(&self, device: &str, scale: f32) -> Result<(), ADBError> {
        let scale_str = scale.to_string();
        self.run_adb(&format!("-s {} shell settings put global animator_duration_scale {}", device, scale_str))?;
        self.run_adb(&format!("-s {} shell settings put global transition_animation_scale {}", device, scale_str))?;
        self.run_adb(&format!("-s {} shell settings put global window_animation_scale {}", device, scale_str))?;
        Ok(())
    }

    /// Enable/disable device location mocking
    pub fn set_location_mock(&self, device: &str, enabled: bool) -> Result<(), ADBError> {
        let value = if enabled { "1" } else { "0" };
        self.run_adb(&format!("-s {} shell settings put secure mock_location {}", device, value))?;
        Ok(())
    }

    /// Set mock location coordinates
    pub fn set_mock_location(&self, device: &str, latitude: f64, longitude: f64) -> Result<(), ADBError> {
        // This requires the mock location app to be set as the location provider
        // This is a simplified implementation
        self.run_adb(&format!("-s {} shell am broadcast -a android.intent.action.SET_MOCK_LOCATION --ef latitude {} --ef longitude {}", device, latitude, longitude))?;
        Ok(())
    }

    /// Get device network information
    pub fn get_network_info(&self, device: &str) -> Result<NetworkInfo, ADBError> {
        let wifi_info = self.run_adb(&format!("-s {} shell dumpsys wifi", device))?;
        let network_info = self.run_adb(&format!("-s {} shell dumpsys connectivity", device))?;

        // Simplified parsing - would need more sophisticated parsing in practice
        Ok(NetworkInfo {
            wifi_connected: wifi_info.contains("Wi-Fi is enabled"),
            mobile_connected: network_info.contains("mobile"),
            ip_address: self.extract_ip_from_network_info(&network_info),
        })
    }

    /// Reboot device
    pub fn reboot_device(&self, device: &str) -> Result<(), ADBError> {
        self.run_adb(&format!("-s {} reboot", device))?;
        Ok(())
    }

    /// Reboot device into recovery mode
    pub fn reboot_recovery(&self, device: &str) -> Result<(), ADBError> {
        self.run_adb(&format!("-s {} reboot recovery", device))?;
        Ok(())
    }

    /// Reboot device into bootloader/fastboot
    pub fn reboot_bootloader(&self, device: &str) -> Result<(), ADBError> {
        self.run_adb(&format!("-s {} reboot bootloader", device))?;
        Ok(())
    }

    // Helper methods for parsing
    fn parse_storage_info(&self, output: &str) -> Result<StorageInfo, ADBError> {
        // Simplified parsing of df output
        let mut total = 0u64;
        let mut used = 0u64;
        let mut available = 0u64;

        for line in output.lines() {
            if line.contains("/data") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    total = parts[1].parse().unwrap_or(0);
                    used = parts[2].parse().unwrap_or(0);
                    available = parts[3].parse().unwrap_or(0);
                }
                break;
            }
        }

        Ok(StorageInfo { total, used, available })
    }

    fn parse_memory_info(&self, output: &str) -> Result<MemoryInfo, ADBError> {
        let mut total = 0u64;
        let mut available = 0u64;

        for line in output.lines() {
            if line.starts_with("MemTotal:") {
                total = self.extract_memory_value(line);
            } else if line.starts_with("MemAvailable:") {
                available = self.extract_memory_value(line);
            }
        }

        Ok(MemoryInfo { total, available })
    }

    fn extract_memory_value(&self, line: &str) -> u64 {
        line.split_whitespace()
            .nth(1)
            .and_then(|s| s.parse().ok())
            .unwrap_or(0)
    }

    fn extract_ip_from_network_info(&self, network_info: &str) -> Option<String> {
        // Very simplified IP extraction
        if let Some(line) = network_info.lines().find(|l| l.contains("ipaddr")) {
            line.split('=').nth(1).map(|s| s.trim().to_string())
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct StorageInfo {
    pub total: u64,      // in KB
    pub used: u64,       // in KB
    pub available: u64,  // in KB
}

#[derive(Debug, Clone)]
pub struct MemoryInfo {
    pub total: u64,      // in KB
    pub available: u64,  // in KB
}

#[derive(Debug, Clone)]
pub struct NetworkInfo {
    pub wifi_connected: bool,
    pub mobile_connected: bool,
    pub ip_address: Option<String>,
}
