use serde::{Deserialize, Serialize};
use crate::error::ADBError;
use crate::ADB;

#[derive(Debug, Serialize, Deserialize)]
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
    pub fn install_app(&self, device: &str, apk_path: &str) -> Result<(), ADBError> {
        let output = self.run_adb(&format!("-s {} install -r {}", device, apk_path))?;
        if !output.contains("Success") {
            return Err(ADBError::PackageInstallation(output));
        }
        Ok(())
    }

    pub async fn install_app_async(&self, device: &str, apk_path: &str) -> Result<(), ADBError> {
        let output = self.run_adb_async(&format!("-s {} install -r {}", device, apk_path)).await?;
        if !output.contains("Success") {
            return Err(ADBError::PackageInstallation(output));
        }
        Ok(())
    }

    pub fn uninstall_app(&self, device: &str, package_name: &str) -> Result<(), ADBError> {
        let output = self.run_adb(&format!("-s {} uninstall {}", device, package_name))?;
        if !output.contains("Success") {
            return Err(ADBError::PackageUninstallation(output));
        }
        Ok(())
    }

    pub async fn uninstall_app_async(&self, device: &str, package_name: &str) -> Result<(), ADBError> {
        let output = self.run_adb_async(&format!("-s {} uninstall {}", device, package_name)).await?;
        if !output.contains("Success") {
            return Err(ADBError::PackageUninstallation(output));
        }
        Ok(())
    }

    pub fn get_package_list(&self, device: &str) -> Result<Vec<PackageInfo>, ADBError> {
        let output = self.run_adb(&format!("-s {} shell pm list packages -f -i", device))?;
        let mut packages = Vec::new();
        for line in output.lines() {
            if let Some(package) = self.parse_package_line(line) {
                packages.push(package);
            }
        }
        Ok(packages)
    }

    pub async fn get_package_list_async(&self, device: &str) -> Result<Vec<PackageInfo>, ADBError> {
        let output = self.run_adb_async(&format!("-s {} shell pm list packages -f -i", device)).await?;
        let mut packages = Vec::new();
        for line in output.lines() {
            if let Some(package) = self.parse_package_line(line) {
                packages.push(package);
            }
        }
        Ok(packages)
    }

    fn parse_package_line(&self, line: &str) -> Option<PackageInfo> {
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

    pub fn get_package_info(&self, device: &str, package_name: &str) -> Result<PackageInfo, ADBError> {
        let output = self.run_adb(&format!("-s {} shell dumpsys package {}", device, package_name))?;
        self.parse_package_info(&output)
            .ok_or_else(|| ADBError::Parse(format!("Failed to parse package info for {}", package_name)))
    }

    pub async fn get_package_info_async(&self, device: &str, package_name: &str) -> Result<PackageInfo, ADBError> {
        let output = self.run_adb_async(&format!("-s {} shell dumpsys package {}", device, package_name)).await?;
        self.parse_package_info(&output)
            .ok_or_else(|| ADBError::Parse(format!("Failed to parse package info for {}", package_name)))
    }

    fn parse_package_info(&self, output: &str) -> Option<PackageInfo> {
        let mut info = PackageInfo {
            name: String::new(),
            version_code: String::new(),
            version_name: String::new(),
            is_system: false,
            install_time: None,
            update_time: None,
            size: None,
        };

        for line in output.lines() {
            let line = line.trim();
            if line.contains("pkg=") || line.starts_with("Package [") {
                info.name = if line.contains("pkg=") {
                    line.split("=").nth(1)?
                        .trim_matches('"')
                        .to_string()
                } else {
                    line.split('[').nth(1)?.split(']').next()?.to_string()
                };
            } else if line.contains("versionCode=") {
                info.version_code = line.split("=").nth(1)?
                    .trim_matches('"')
                    .to_string();
            } else if line.contains("versionName=") {
                info.version_name = line.split("=").nth(1)?
                    .trim_matches('"')
                    .to_string();
            } else if line.contains("system app") || line.contains("System") {
                info.is_system = true;
            } else if line.contains("firstInstallTime=") {
                info.install_time = Some(line.split("=").nth(1)?
                    .trim_matches('"')
                    .to_string());
            } else if line.contains("lastUpdateTime=") {
                info.update_time = Some(line.split("=").nth(1)?
                    .trim_matches('"')
                    .to_string());
            }
        }

        if info.name.is_empty() { None } else { Some(info) }
    }
    }
}