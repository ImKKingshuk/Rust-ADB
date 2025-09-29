use serde::{Deserialize, Serialize};
use crate::error::ADBError;
use crate::ADB;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfo {
    pub name: String,
    pub version_code: String,
    pub version_name: String,
    pub is_system: bool,
    pub install_time: Option<String>,
    pub update_time: Option<String>,
    pub size: Option<u64>,
}

impl ADB {
    pub fn get_package_list(&self, device: &str) -> Result<Vec<PackageInfo>, ADBError> {
        let output = self.run_adb(&format!("-s {} shell pm list packages -f", device))?;
        let mut packages = Vec::new();
        for line in output.lines() {
            if let Some(package) = self.parse_package_line(line) {
                packages.push(package);
            }
        }
        Ok(packages)
    }

    pub async fn get_package_list_async(&self, device: &str) -> Result<Vec<PackageInfo>, ADBError> {
        let output = self.run_adb_async(&format!("-s {} shell pm list packages -f", device)).await?;
        let mut packages = Vec::new();
        for line in output.lines() {
            if let Some(package) = self.parse_package_line(line) {
                packages.push(package);
            }
        }
        Ok(packages)
    }

    fn parse_package_line(&self, line: &str) -> Option<PackageInfo> {
        // Parse line like: package:/data/app/com.example.app/base.apk=com.example.app
        let parts: Vec<&str> = line.split('=').collect();
        if parts.len() != 2 { return None; }

        let name = parts[1].to_string();
        Some(PackageInfo {
            name,
            version_code: String::new(),
            version_name: String::new(),
            is_system: false,
            install_time: None,
            update_time: None,
            size: None,
        })
    }

    /// Get detailed information about a specific package
    pub fn get_package_info(&self, device: &str, package_name: &str) -> Result<PackageInfo, ADBError> {
        let output = self.run_adb(&format!("-s {} shell dumpsys package {}", device, package_name))?;
        self.parse_detailed_package_info(&output, package_name)
    }

    /// Get detailed information about a specific package (async)
    pub async fn get_package_info_async(&self, device: &str, package_name: &str) -> Result<PackageInfo, ADBError> {
        let output = self.run_adb_async(&format!("-s {} shell dumpsys package {}", device, package_name)).await?;
        self.parse_detailed_package_info(&output, package_name)
    }

    fn parse_detailed_package_info(&self, dumpsys_output: &str, package_name: &str) -> Result<PackageInfo, ADBError> {
        let mut package = PackageInfo {
            name: package_name.to_string(),
            version_code: String::new(),
            version_name: String::new(),
            is_system: false,
            install_time: None,
            update_time: None,
            size: None,
        };

        for line in dumpsys_output.lines() {
            let line = line.trim();
            if line.starts_with("versionCode=") {
                if let Some(value) = line.split('=').nth(1) {
                    package.version_code = value.trim().to_string();
                }
            } else if line.starts_with("versionName=") {
                if let Some(value) = line.split('=').nth(1) {
                    package.version_name = value.trim().to_string();
                }
            } else if line.starts_with("firstInstallTime=") {
                if let Some(value) = line.split('=').nth(1) {
                    package.install_time = Some(value.trim().to_string());
                }
            } else if line.starts_with("lastUpdateTime=") {
                if let Some(value) = line.split('=').nth(1) {
                    package.update_time = Some(value.trim().to_string());
                }
            } else if line.contains("codePath=") && line.contains("/system/") {
                package.is_system = true;
            }
        }

        // Try to get package size - simplified implementation
        // In practice, you'd want to use stat commands on the APK paths
        package.size = Some(0); // Placeholder - would need proper implementation

        Ok(package)
    }

    pub fn install_app(&self, device: &str, apk_path: &str) -> Result<String, ADBError> {
        self.run_adb(&format!("-s {} install -r {}", device, apk_path))
    }

    pub async fn install_app_async(&self, device: &str, apk_path: &str) -> Result<String, ADBError> {
        self.run_adb_async(&format!("-s {} install -r {}", device, apk_path)).await
    }

    pub fn uninstall_app(&self, device: &str, package_name: &str) -> Result<String, ADBError> {
        self.run_adb(&format!("-s {} uninstall {}", device, package_name))
    }

    pub async fn uninstall_app_async(&self, device: &str, package_name: &str) -> Result<String, ADBError> {
        self.run_adb_async(&format!("-s {} uninstall {}", device, package_name)).await
    }
}