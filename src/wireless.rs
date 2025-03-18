use crate::error::ADBError;
use crate::ADB;

impl ADB {
    pub fn connect_wireless(&self, ip: &str, port: u16) -> Result<(), ADBError> {
        const MAX_RETRIES: u32 = 5;
        const RETRY_DELAY_MS: u64 = 2000;
        const CONNECTION_TIMEOUT_MS: u64 = 10000;

        let start_time = std::time::Instant::now();
        for attempt in 1..=MAX_RETRIES {
            let output = self.run_adb(&format!("connect {}:{}", ip, port))?;
            if output.contains("connected") {
                info!("Successfully connected to device {}:{} on attempt {}", ip, port, attempt);
                return Ok(());
            }

            if start_time.elapsed().as_millis() as u64 > CONNECTION_TIMEOUT_MS {
                return Err(ADBError::ConnectionTimeout(format!("Connection timeout after {}ms", CONNECTION_TIMEOUT_MS)));
            }

            warn!("Connection attempt {} failed, retrying...", attempt);
            if attempt < MAX_RETRIES {
                std::thread::sleep(std::time::Duration::from_millis(RETRY_DELAY_MS));
            }
        }
        Err(ADBError::ConnectionRetry(format!("Failed to connect to {}:{} after {} attempts", ip, port, MAX_RETRIES)))
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
        // First, check if device is connected via USB
        let devices = self.refresh_device_list()?;
        if !devices.iter().any(|d| d.serial == device) {
            return Err(ADBError::DeviceNotFound(device.to_string()));
        }

        // Enable TCP/IP mode
        let output = self.run_adb(&format!("-s {} tcpip 5555", device))?;
        if !output.contains("restarting in TCP mode") {
            return Err(ADBError::WirelessConnection(output));
        }

        // Wait for device to restart in TCP mode
        std::thread::sleep(std::time::Duration::from_secs(2));

        // Get device IP address (try multiple network interfaces)
        let interfaces = ["wlan0", "eth0", "wifi0"];
        let mut ip = None;
        
        for interface in interfaces.iter() {
            let ip_output = self.shell_command(device, &format!("ip addr show {}", interface));
            if let Ok(output) = ip_output {
                if let Some(addr) = output.lines()
                    .find(|line| line.contains("inet ") && !line.contains("127.0.0.1"))
                    .and_then(|line| line.split_whitespace().nth(1))
                    .and_then(|addr| addr.split('/').next()) {
                    ip = Some(addr.to_string());
                    info!("Found device IP address {} on interface {}", addr, interface);
                    break;
                }
            }
        }

        let ip = ip.ok_or_else(|| ADBError::WirelessConnection("Failed to get device IP".to_string()))?;

        // Verify TCP/IP mode is active
        let status = self.shell_command(device, "getprop service.adb.tcp.port")?;
        if !status.trim().contains("5555") {
            return Err(ADBError::WirelessConnection("TCP/IP mode not active".to_string()));
        }

        Ok((ip, 5555))
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