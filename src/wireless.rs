use crate::error::ADBError;
use crate::ADB;

impl ADB {
    pub fn connect_wireless(&self, ip: &str, port: u16) -> Result<(), ADBError> {
        let output = self.run_adb(&format!("connect {}:{}", ip, port))?;
        if !output.contains("connected") {
            return Err(ADBError::WirelessConnection(output));
        }
        Ok(())
    }

    pub async fn connect_wireless_async(&self, ip: &str, port: u16) -> Result<(), ADBError> {
        let output = self.run_adb_async(&format!("connect {}:{}", ip, port)).await?;
        if !output.contains("connected") {
            return Err(ADBError::WirelessConnection(output));
        }
        Ok(())
    }

    pub fn disconnect_wireless(&self, ip: &str, port: u16) -> Result<(), ADBError> {
        let output = self.run_adb(&format!("disconnect {}:{}", ip, port))?;
        if !output.contains("disconnected") {
            return Err(ADBError::WirelessConnection(output));
        }
        Ok(())
    }

    pub async fn disconnect_wireless_async(&self, ip: &str, port: u16) -> Result<(), ADBError> {
        let output = self.run_adb_async(&format!("disconnect {}:{}", ip, port)).await?;
        if !output.contains("disconnected") {
            return Err(ADBError::WirelessConnection(output));
        }
        Ok(())
    }

    pub fn enable_wireless_debugging(&self, device: &str) -> Result<(String, u16), ADBError> {
        let output = self.run_adb(&format!("-s {} tcpip 5555", device))?;
        if !output.contains("restarting in TCP mode") {
            return Err(ADBError::WirelessConnection(output));
        }

        // Get device IP address
        let ip_output = self.shell_command(device, "ip route | grep wlan0")?;
        let ip = ip_output
            .split_whitespace()
            .nth(8)
            .ok_or_else(|| ADBError::WirelessConnection("Failed to get device IP".to_string()))?;

        Ok((ip.to_string(), 5555))
    }

    pub async fn enable_wireless_debugging_async(&self, device: &str) -> Result<(String, u16), ADBError> {
        let output = self.run_adb_async(&format!("-s {} tcpip 5555", device)).await?;
        if !output.contains("restarting in TCP mode") {
            return Err(ADBError::WirelessConnection(output));
        }

        // Get device IP address
        let ip_output = self.shell_command_async(device, "ip route | grep wlan0").await?;
        let ip = ip_output
            .split_whitespace()
            .nth(8)
            .ok_or_else(|| ADBError::WirelessConnection("Failed to get device IP".to_string()))?;

        Ok((ip.to_string(), 5555))
    }

    pub fn disable_wireless_debugging(&self, device: &str) -> Result<(), ADBError> {
        let output = self.run_adb(&format!("-s {} usb", device))?;
        if !output.contains("restarting in USB mode") {
            return Err(ADBError::WirelessConnection(output));
        }
        Ok(())
    }

    pub async fn disable_wireless_debugging_async(&self, device: &str) -> Result<(), ADBError> {
        let output = self.run_adb_async(&format!("-s {} usb", device)).await?;
        if !output.contains("restarting in USB mode") {
            return Err(ADBError::WirelessConnection(output));
        }
        Ok(())
    }
}