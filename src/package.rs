use std::process::Command;
use std::str;

#[derive(Debug)]
pub struct Package {
    pub name: String,
    pub version_code: String,
    pub version_name: String,
    pub install_location: String,
    pub first_install_time: String,
    pub last_update_time: String,
    pub permissions: Vec<String>,
}

#[derive(Debug)]
pub enum PackageError {
    CommandFailed(String),
    ParseError(String),
    InstallError(String),
    UninstallError(String),
}

impl Package {
    pub fn new(package_name: &str) -> Result<Self, PackageError> {
        let output = Command::new("adb")
            .args(["shell", "dumpsys", "package", package_name])
            .output()
            .map_err(|e| PackageError::CommandFailed(e.to_string()))?;

        if !output.status.success() {
            return Err(PackageError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        Self::parse_package_info(&output_str)
    }

    fn parse_package_info(info: &str) -> Result<Self, PackageError> {
        let mut package = Package {
            name: String::new(),
            version_code: String::new(),
            version_name: String::new(),
            install_location: String::new(),
            first_install_time: String::new(),
            last_update_time: String::new(),
            permissions: Vec::new(),
        };

        for line in info.lines() {
            let line = line.trim();
            if line.starts_with("Package [") {
                package.name = line
                    .split('[').nth(1)
                    .and_then(|s| s.split(']').next())
                    .ok_or_else(|| PackageError::ParseError("Failed to parse package name".to_string()))?
                    .to_string();
            } else if line.starts_with("versionCode=") {
                package.version_code = line
                    .split('=').nth(1)
                    .ok_or_else(|| PackageError::ParseError("Failed to parse version code".to_string()))?
                    .to_string();
            } else if line.starts_with("versionName=") {
                package.version_name = line
                    .split('=').nth(1)
                    .ok_or_else(|| PackageError::ParseError("Failed to parse version name".to_string()))?
                    .to_string();
            } else if line.starts_with("installLocation=") {
                package.install_location = line
                    .split('=').nth(1)
                    .ok_or_else(|| PackageError::ParseError("Failed to parse install location".to_string()))?
                    .to_string();
            } else if line.starts_with("firstInstallTime=") {
                package.first_install_time = line
                    .split('=').nth(1)
                    .ok_or_else(|| PackageError::ParseError("Failed to parse first install time".to_string()))?
                    .to_string();
            } else if line.starts_with("lastUpdateTime=") {
                package.last_update_time = line
                    .split('=').nth(1)
                    .ok_or_else(|| PackageError::ParseError("Failed to parse last update time".to_string()))?
                    .to_string();
            } else if line.starts_with("granted=true") && line.contains("permission.") {
                if let Some(permission) = line.split("permission.").nth(1) {
                    package.permissions.push(permission.trim().to_string());
                }
            }
        }

        Ok(package)
    }

    pub fn install(apk_path: &str) -> Result<(), PackageError> {
        let output = Command::new("adb")
            .args(["install", "-r", apk_path])
            .output()
            .map_err(|e| PackageError::InstallError(e.to_string()))?;

        if !output.status.success() {
            return Err(PackageError::InstallError(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        Ok(())
    }

    pub fn uninstall(package_name: &str) -> Result<(), PackageError> {
        let output = Command::new("adb")
            .args(["uninstall", package_name])
            .output()
            .map_err(|e| PackageError::UninstallError(e.to_string()))?;

        if !output.status.success() {
            return Err(PackageError::UninstallError(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        Ok(())
    }

    pub fn list_packages() -> Result<Vec<String>, PackageError> {
        let output = Command::new("adb")
            .args(["shell", "pm", "list", "packages"])
            .output()
            .map_err(|e| PackageError::CommandFailed(e.to_string()))?;

        if !output.status.success() {
            return Err(PackageError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        let packages = output_str
            .lines()
            .filter_map(|line| line.strip_prefix("package:"))
            .map(|s| s.trim().to_string())
            .collect();

        Ok(packages)
    }
}