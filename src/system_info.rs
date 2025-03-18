use serde::{Deserialize, Serialize};
use crate::error::ADBError;
use crate::ADB;

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemInfo {
    pub android_version: String,
    pub sdk_version: String,
    pub device_arch: String,
    pub security_patch: String,
    pub build_fingerprint: String,
    pub kernel_version: String,
    pub bootloader: String,
    pub baseband: String,
    pub system_partition: String,
    pub encryption_state: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BatteryInfo {
    pub level: i32,
    pub temperature: f32,
    pub voltage: f32,
    pub current: Option<i32>,
    pub status: String,
    pub health: String,
    pub is_charging: bool,
    pub power_source: String,
    pub technology: String,
    pub capacity: Option<i32>,
}

impl ADB {
    pub fn get_system_info(&self, device: &str) -> Result<SystemInfo, ADBError> {
        let props = self.get_device_props(device)?;
        let mut info = SystemInfo {
            android_version: String::new(),
            sdk_version: String::new(),
            device_arch: String::new(),
            security_patch: String::new(),
            build_fingerprint: String::new(),
            kernel_version: String::new(),
            bootloader: String::new(),
            baseband: String::new(),
            system_partition: String::new(),
            encryption_state: String::new(),
        };

        for line in props.lines() {
            if line.contains("[ro.build.version.release]") {
                info.android_version = Self::extract_prop_value(line);
            } else if line.contains("[ro.build.version.sdk]") {
                info.sdk_version = Self::extract_prop_value(line);
            } else if line.contains("[ro.product.cpu.abi]") {
                info.device_arch = Self::extract_prop_value(line);
            } else if line.contains("[ro.build.version.security_patch]") {
                info.security_patch = Self::extract_prop_value(line);
            } else if line.contains("[ro.build.fingerprint]") {
                info.build_fingerprint = Self::extract_prop_value(line);
            } else if line.contains("[ro.build.version.kernel]") {
                info.kernel_version = Self::extract_prop_value(line);
            } else if line.contains("[ro.bootloader]") {
                info.bootloader = Self::extract_prop_value(line);
            } else if line.contains("[ro.build.expect.baseband]") {
                info.baseband = Self::extract_prop_value(line);
            } else if line.contains("[ro.build.system_root_image]") {
                info.system_partition = Self::extract_prop_value(line);
            } else if line.contains("[ro.crypto.state]") {
                info.encryption_state = Self::extract_prop_value(line);
            }
        }

        Ok(info)
    }

    pub async fn get_system_info_async(&self, device: &str) -> Result<SystemInfo, ADBError> {
        let props = self.get_device_props_async(device).await?;
        let mut info = SystemInfo {
            android_version: String::new(),
            sdk_version: String::new(),
            device_arch: String::new(),
            security_patch: String::new(),
            build_fingerprint: String::new(),
            kernel_version: String::new(),
            bootloader: String::new(),
            baseband: String::new(),
            system_partition: String::new(),
            encryption_state: String::new(),
        };

        for line in props.lines() {
            if line.contains("[ro.build.version.release]") {
                info.android_version = Self::extract_prop_value(line);
            } else if line.contains("[ro.build.version.sdk]") {
                info.sdk_version = Self::extract_prop_value(line);
            } else if line.contains("[ro.product.cpu.abi]") {
                info.device_arch = Self::extract_prop_value(line);
            } else if line.contains("[ro.build.version.security_patch]") {
                info.security_patch = Self::extract_prop_value(line);
            } else if line.contains("[ro.build.fingerprint]") {
                info.build_fingerprint = Self::extract_prop_value(line);
            } else if line.contains("[ro.build.version.kernel]") {
                info.kernel_version = Self::extract_prop_value(line);
            } else if line.contains("[ro.bootloader]") {
                info.bootloader = Self::extract_prop_value(line);
            } else if line.contains("[ro.build.expect.baseband]") {
                info.baseband = Self::extract_prop_value(line);
            } else if line.contains("[ro.build.system_root_image]") {
                info.system_partition = Self::extract_prop_value(line);
            } else if line.contains("[ro.crypto.state]") {
                info.encryption_state = Self::extract_prop_value(line);
            }
        }

        Ok(info)
    }

    pub fn get_battery_info(&self, device: &str) -> Result<BatteryInfo, ADBError> {
        let output = self.run_adb(&format!("-s {} shell dumpsys battery", device))?;
        let mut info = BatteryInfo {
            level: 0,
            temperature: 0.0,
            voltage: 0.0,
            current: None,
            status: String::new(),
            health: String::new(),
            is_charging: false,
            power_source: String::new(),
            technology: String::new(),
            capacity: None,
        };

        for line in output.lines() {
            let line = line.trim();
            if line.starts_with("level:") {
                info.level = Self::parse_int_value(line);
            } else if line.starts_with("temperature:") {
                info.temperature = Self::parse_int_value(line) as f32 / 10.0;
            } else if line.starts_with("voltage:") {
                info.voltage = Self::parse_int_value(line) as f32 / 1000.0;
            } else if line.starts_with("current now:") {
                info.current = Some(Self::parse_int_value(line));
            } else if line.starts_with("status:") {
                info.status = Self::extract_value(line);
            } else if line.starts_with("health:") {
                info.health = Self::extract_value(line);
            } else if line.starts_with("plugged:") {
                info.is_charging = Self::parse_int_value(line) != 0;
                info.power_source = match Self::parse_int_value(line) {
                    1 => "AC".to_string(),
                    2 => "USB".to_string(),
                    4 => "Wireless".to_string(),
                    _ => "Unknown".to_string(),
                };
            } else if line.starts_with("technology:") {
                info.technology = Self::extract_value(line);
            } else if line.starts_with("capacity:") {
                info.capacity = Some(Self::parse_int_value(line));
            }
        }

        Ok(info)
    }

    pub async fn get_battery_info_async(&self, device: &str) -> Result<BatteryInfo, ADBError> {
        let output = self.run_adb_async(&format!("-s {} shell dumpsys battery", device)).await?;
        let mut info = BatteryInfo {
            level: 0,
            temperature: 0.0,
            voltage: 0.0,
            current: None,
            status: String::new(),
            health: String::new(),
            is_charging: false,
            power_source: String::new(),
            technology: String::new(),
            capacity: None,
        };

        for line in output.lines() {
            let line = line.trim();
            if line.starts_with("level:") {
                info.level = Self::parse_int_value(line);
            } else if line.starts_with("temperature:") {
                info.temperature = Self::parse_int_value(line) as f32 / 10.0;
            } else if line.starts_with("voltage:") {
                info.voltage = Self::parse_int_value(line) as f32 / 1000.0;
            } else if line.starts_with("current now:") {
                info.current = Some(Self::parse_int_value(line));
            } else if line.starts_with("status:") {
                info.status = Self::extract_value(line);
            } else if line.starts_with("health:") {
                info.health = Self::extract_value(line);
            } else if line.starts_with("plugged:") {
                info.is_charging = Self::parse_int_value(line) != 0;
                info.power_source = match Self::parse_int_value(line) {
                    1 => "AC".to_string(),
                    2 => "USB".to_string(),
                    4 => "Wireless".to_string(),
                    _ => "Unknown".to_string(),
                };
            } else if line.starts_with("technology:") {
                info.technology = Self::extract_value(line);
            } else if line.starts_with("capacity:") {
                info.capacity = Some(Self::parse_int_value(line));
            }
        }

        Ok(info)
    }

    fn extract_prop_value(line: &str) -> String {
        line.split("]: [").nth(1)
            .map(|s| s.trim_end_matches(']'))
            .unwrap_or("").to_string()
    }

    fn extract_value(line: &str) -> String {
        line.split(':').nth(1)
            .map(|s| s.trim())
            .unwrap_or("").to_string()
    }

    fn parse_int_value(line: &str) -> i32 {
        line.split(':').nth(1)
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(0)
    }
}