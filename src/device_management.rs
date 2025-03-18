use crate::error::ADBError;
use crate::ADB;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemInfo {
    pub android_version: String,
    pub sdk_version: String,
    pub device_arch: String,
    pub security_patch: String,
    pub build_fingerprint: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BatteryInfo {
    pub level: i32,
    pub temperature: f32,
    pub status: String,
    pub health: String,
    pub is_charging: bool,
}

impl ADB {
    pub fn start_app(&self, device: &str, package_name: &str) -> Result<(), ADBError> {
        let output = self.run_adb(&format!("-s {} shell monkey -p {} -c android.intent.category.LAUNCHER 1", device, package_name))?;
        if output.contains("error") {
            return Err(ADBError::CommandFailed(output));
        }
        Ok(())
    }

    pub async fn start_app_async(&self, device: &str, package_name: &str) -> Result<(), ADBError> {
        let output = self.run_adb_async(&format!("-s {} shell monkey -p {} -c android.intent.category.LAUNCHER 1", device, package_name)).await?;
        if output.contains("error") {
            return Err(ADBError::CommandFailed(output));
        }
        Ok(())
    }

    pub fn stop_app(&self, device: &str, package_name: &str) -> Result<(), ADBError> {
        self.run_adb(&format!("-s {} shell am force-stop {}", device, package_name))?;
        Ok(())
    }

    pub async fn stop_app_async(&self, device: &str, package_name: &str) -> Result<(), ADBError> {
        self.run_adb_async(&format!("-s {} shell am force-stop {}", device, package_name)).await?;
        Ok(())
    }

    pub fn start_screen_record(&self, device: &str, output_path: &str, time_limit: u32) -> Result<(), ADBError> {
        self.run_adb(&format!("-s {} shell screenrecord --time-limit {} {}", 
            device, time_limit, output_path))?;
        Ok(())
    }

    pub fn input_keyevent(&self, device: &str, keycode: i32) -> Result<(), ADBError> {
        self.run_adb(&format!("-s {} shell input keyevent {}", device, keycode))?;
        Ok(())
    }

    pub fn input_text(&self, device: &str, text: &str) -> Result<(), ADBError> {
        self.run_adb(&format!("-s {} shell input text {}", device, text))?;
        Ok(())
    }

    pub fn input_tap(&self, device: &str, x: i32, y: i32) -> Result<(), ADBError> {
        self.run_adb(&format!("-s {} shell input tap {} {}", device, x, y))?;
        Ok(())
    }

    pub fn input_swipe(&self, device: &str, x1: i32, y1: i32, x2: i32, y2: i32, duration_ms: u32) -> Result<(), ADBError> {
        self.run_adb(&format!("-s {} shell input swipe {} {} {} {} {}", 
            device, x1, y1, x2, y2, duration_ms))?;
        Ok(())
    }

    pub fn clear_app_data(&self, device: &str, package_name: &str) -> Result<(), ADBError> {
        self.run_adb(&format!("-s {} shell pm clear {}", device, package_name))?;
        Ok(())
    }

    pub async fn clear_app_data_async(&self, device: &str, package_name: &str) -> Result<(), ADBError> {
        self.run_adb_async(&format!("-s {} shell pm clear {}", device, package_name)).await?;
        Ok(())
    }

    pub fn get_system_info(&self, device: &str) -> Result<SystemInfo, ADBError> {
        let props = self.get_device_props(device)?;
        let mut info = SystemInfo {
            android_version: String::new(),
            sdk_version: String::new(),
            device_arch: String::new(),
            security_patch: String::new(),
            build_fingerprint: String::new(),
        };

        for line in props.lines() {
            if line.contains("[ro.build.version.release]") {
                info.android_version = line.split("]: [").nth(1).unwrap_or("").trim_matches(']').to_string();
            } else if line.contains("[ro.build.version.sdk]") {
                info.sdk_version = line.split("]: [").nth(1).unwrap_or("").trim_matches(']').to_string();
            } else if line.contains("[ro.product.cpu.abi]") {
                info.device_arch = line.split("]: [").nth(1).unwrap_or("").trim_matches(']').to_string();
            } else if line.contains("[ro.build.version.security_patch]") {
                info.security_patch = line.split("]: [").nth(1).unwrap_or("").trim_matches(']').to_string();
            } else if line.contains("[ro.build.fingerprint]") {
                info.build_fingerprint = line.split("]: [").nth(1).unwrap_or("").trim_matches(']').to_string();
            }
        }

        Ok(info)
    }

    pub fn get_battery_info(&self, device: &str) -> Result<BatteryInfo, ADBError> {
        let output = self.run_adb(&format!("-s {} shell dumpsys battery", device))?;
        let mut info = BatteryInfo {
            level: 0,
            temperature: 0.0,
            status: String::new(),
            health: String::new(),
            is_charging: false,
        };

        for line in output.lines() {
            if line.contains("level:") {
                info.level = line.split_whitespace().nth(1).unwrap_or("0").parse().unwrap_or(0);
            } else if line.contains("temperature:") {
                let temp = line.split_whitespace().nth(1).unwrap_or("0").parse::<i32>().unwrap_or(0);
                info.temperature = temp as f32 / 10.0;
            } else if line.contains("status:") {
                info.status = line.split_whitespace().nth(1).unwrap_or("").to_string();
            } else if line.contains("health:") {
                info.health = line.split_whitespace().nth(1).unwrap_or("").to_string();
            } else if line.contains("plugged:") {
                info.is_charging = line.split_whitespace().nth(1).unwrap_or("0") != "0";
            }
        }

        Ok(info)
    }

    pub fn start_screen_record(&self, device: &str, output_path: &str, time_limit: u32) -> Result<(), ADBError> {
        self.run_adb(&format!("-s {} shell screenrecord --time-limit {} {}", 
            device, time_limit, output_path))?;
        Ok(())
    }

    pub fn input_keyevent(&self, device: &str, keycode: i32) -> Result<(), ADBError> {
        self.run_adb(&format!("-s {} shell input keyevent {}", device, keycode))?;
        Ok(())
    }

    pub fn input_text(&self, device: &str, text: &str) -> Result<(), ADBError> {
        self.run_adb(&format!("-s {} shell input text {}", device, text))?;
        Ok(())
    }

    pub fn input_tap(&self, device: &str, x: i32, y: i32) -> Result<(), ADBError> {
        self.run_adb(&format!("-s {} shell input tap {} {}", device, x, y))?;
        Ok(())
    }

    pub fn input_swipe(&self, device: &str, x1: i32, y1: i32, x2: i32, y2: i32, duration_ms: u32) -> Result<(), ADBError> {
        self.run_adb(&format!("-s {} shell input swipe {} {} {} {} {}", 
            device, x1, y1, x2, y2, duration_ms))?;
        Ok(())
    }
}