use serde::{Deserialize, Serialize};
use crate::error::ADBError;
use crate::ADB;

#[derive(Debug, Serialize, Deserialize)]
pub struct Device {
    pub serial: String,
    pub state: String,
    pub product: Option<String>,
    pub model: Option<String>,
    pub device: Option<String>,
    pub transport_id: Option<String>,
}

impl ADB {
    pub fn refresh_device_list(&self) -> Result<Vec<Device>, ADBError> {
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

    pub async fn refresh_device_list_async(&self) -> Result<Vec<Device>, ADBError> {
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

    pub fn get_device_props(&self, device: &str) -> Result<String, ADBError> {
        self.run_adb(&format!("-s {} shell getprop", device))
    }

    pub async fn get_device_props_async(&self, device: &str) -> Result<String, ADBError> {
        self.run_adb_async(&format!("-s {} shell getprop", device)).await
    }

    pub fn set_device_prop(&self, device: &str, prop: &str, value: &str) -> Result<(), ADBError> {
        self.run_adb(&format!("-s {} shell setprop {} {}", device, prop, value))?;
        Ok(())
    }

    pub async fn set_device_prop_async(&self, device: &str, prop: &str, value: &str) -> Result<(), ADBError> {
        self.run_adb_async(&format!("-s {} shell setprop {} {}", device, prop, value)).await?;
        Ok(())
    }
}