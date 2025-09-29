use crate::error::ADBError;
use crate::ADB;
use log::{info, warn};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkDiagnostics {
    pub wifi_enabled: bool,
    pub mobile_data_enabled: bool,
    pub connected_networks: Vec<String>,
    pub ip_routes: String,
}

impl ADB {
    /// Forward a local port to a remote port on the device
    pub fn forward_port(&self, device: &str, local_port: u16, remote_port: u16) -> Result<(), ADBError> {
        self.run_adb(&format!("-s {} forward tcp:{} tcp:{}", device, local_port, remote_port))?;
        Ok(())
    }

    /// Remove a port forwarding rule
    pub fn remove_port_forward(&self, device: &str, local_port: u16) -> Result<(), ADBError> {
        self.run_adb(&format!("-s {} forward --remove tcp:{}", device, local_port))?;
        Ok(())
    }

    /// Reverse forward a remote port to a local port (device to host)
    pub fn reverse_forward_port(&self, device: &str, remote_port: u16, local_port: u16) -> Result<(), ADBError> {
        self.run_adb(&format!("-s {} reverse tcp:{} tcp:{}", device, remote_port, local_port))?;
        Ok(())
    }

    /// Remove a reverse port forwarding rule
    pub fn remove_reverse_forward(&self, device: &str, remote_port: u16) -> Result<(), ADBError> {
        self.run_adb(&format!("-s {} reverse --remove tcp:{}", device, remote_port))?;
        Ok(())
    }

    /// List all active port forwarding rules
    pub fn list_forwards(&self, device: &str) -> Result<String, ADBError> {
        self.run_adb(&format!("-s {} forward --list", device))
    }

    /// List all active reverse port forwarding rules
    pub fn list_reverse_forwards(&self, device: &str) -> Result<String, ADBError> {
        self.run_adb(&format!("-s {} reverse --list", device))
    }

    /// Setup port forwarding for common development scenarios
    pub fn setup_dev_forwarding(&self, device: &str) -> Result<(), ADBError> {
        // Forward common development ports
        self.forward_port(device, 8080, 8080)?; // HTTP dev server
        self.forward_port(device, 3000, 3000)?; // React/Node dev server
        self.forward_port(device, 5000, 5000)?; // Flask/Django dev server
        self.forward_port(device, 5037, 5037)?; // ADB server port
        Ok(())
    }

    /// Get network diagnostics information
    pub fn get_network_diagnostics(&self, device: &str) -> Result<NetworkDiagnostics, ADBError> {
        let wifi_info = self.run_adb(&format!("-s {} shell dumpsys wifi", device))?;
        let network_info = self.run_adb(&format!("-s {} shell dumpsys connectivity", device))?;
        let ip_info = self.run_adb(&format!("-s {} shell ip route", device))?;

        Ok(NetworkDiagnostics {
            wifi_enabled: wifi_info.contains("Wi-Fi is enabled"),
            mobile_data_enabled: network_info.contains("MOBILE"),
            connected_networks: self.parse_connected_networks(&network_info),
            ip_routes: ip_info,
        })
    }

    /// Test network connectivity to a specific host
    pub fn test_connectivity(&self, device: &str, host: &str, port: Option<u16>) -> Result<bool, ADBError> {
        let port_str = port.map(|p| format!(":{}", p)).unwrap_or_default();
        let output = self.run_adb(&format!("-s {} shell nc -z -w5 {}{}", device, host, port_str))?;
        Ok(output.is_empty()) // nc returns empty on success
    }

    /// Get current network interface information
    pub fn get_network_interfaces(&self, device: &str) -> Result<String, ADBError> {
        self.run_adb(&format!("-s {} shell ip addr show", device))
    }

    // Helper method for parsing connected networks
    fn parse_connected_networks(&self, network_info: &str) -> Vec<String> {
        let mut networks = Vec::new();
        for line in network_info.lines() {
            if line.contains("NetworkAgentInfo") && line.contains("CONNECTED") {
                // Extract network type (WiFi, Mobile, etc.)
                if line.contains("WIFI") {
                    networks.push("WiFi".to_string());
                } else if line.contains("MOBILE") {
                    networks.push("Mobile".to_string());
                } else if line.contains("ETHERNET") {
                    networks.push("Ethernet".to_string());
                }
            }
        }
        networks
    }
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