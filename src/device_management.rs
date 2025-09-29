use crate::error::ADBError;
use crate::ADB;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppDataSize {
    pub total_size: u64, // in bytes
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppPermissions {
    pub package_name: String,
    pub permissions: Vec<String>,
    pub requested_permissions: Vec<String>,
    pub granted_permissions: Vec<String>,
    pub denied_permissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub user: String,
    pub cpu_usage: f32,
    pub memory_usage: u64,
    pub status: String,
}

impl ADB {
    /// Get detailed permissions for a specific app
    pub fn get_app_permissions(&self, device: &str, package_name: &str) -> Result<AppPermissions, ADBError> {
        let output = self.run_adb(&format!("-s {} shell dumpsys package {}", device, package_name))?;
        self.parse_app_permissions(&output, package_name)
    }

    /// Grant a specific permission to an app
    pub fn grant_permission(&self, device: &str, package_name: &str, permission: &str) -> Result<(), ADBError> {
        self.run_adb(&format!("-s {} shell pm grant {} {}", device, package_name, permission))?;
        Ok(())
    }

    /// Revoke a specific permission from an app
    pub fn revoke_permission(&self, device: &str, package_name: &str, permission: &str) -> Result<(), ADBError> {
        self.run_adb(&format!("-s {} shell pm revoke {} {}", device, package_name, permission))?;
        Ok(())
    }

    /// Reset permissions for an app
    pub fn reset_permissions(&self, device: &str, package_name: &str) -> Result<(), ADBError> {
        self.run_adb(&format!("-s {} shell pm reset-permissions {}", device, package_name))?;
        Ok(())
    }

    /// Get running processes information
    pub fn get_running_processes(&self, device: &str) -> Result<Vec<ProcessInfo>, ADBError> {
        let output = self.run_adb(&format!("-s {} shell ps", device))?;
        self.parse_processes(&output)
    }

    /// Kill a process by PID
    pub fn kill_process(&self, device: &str, pid: u32) -> Result<(), ADBError> {
        self.run_adb(&format!("-s {} shell kill {}", device, pid))?;
        Ok(())
    }

    /// Kill a process by name
    pub fn kill_process_by_name(&self, device: &str, process_name: &str) -> Result<(), ADBError> {
        let processes = self.get_running_processes(device)?;
        for process in processes {
            if process.name.contains(process_name) {
                self.kill_process(device, process.pid)?;
                return Ok(());
            }
        }
        Err(ADBError::CommandFailed(format!("Process {} not found", process_name)))
    }

    /// Get app data size information
    pub fn get_app_data_size(&self, device: &str, package_name: &str) -> Result<AppDataSize, ADBError> {
        let output = self.run_adb(&format!("-s {} shell pm path {}", device, package_name))?;
        self.calculate_app_data_size(device, &output)
    }

    /// Backup app data
    pub fn backup_app_data(&self, device: &str, package_name: &str, output_path: &str) -> Result<(), ADBError> {
        self.run_adb(&format!("-s {} backup -f {} -apk {}", device, output_path, package_name))?;
        Ok(())
    }

    /// Restore app data
    pub fn restore_app_data(&self, device: &str, backup_path: &str) -> Result<(), ADBError> {
        self.run_adb(&format!("-s {} restore {}", device, backup_path))?;
        Ok(())
    }

    /// Enable/disable app components
    pub fn set_component_state(&self, device: &str, component: &str, enabled: bool) -> Result<(), ADBError> {
        let action = if enabled { "enable" } else { "disable" };
        self.run_adb(&format!("-s {} shell pm {} {}", device, action, component))?;
        Ok(())
    }

    /// Get app component states
    pub fn get_component_states(&self, device: &str, package_name: &str) -> Result<String, ADBError> {
        self.run_adb(&format!("-s {} shell dumpsys package {} | grep -A 10 'Activity Resolver Table'", device, package_name))
    }

    /// Force stop all apps except system apps
    pub fn force_stop_all_apps(&self, device: &str) -> Result<(), ADBError> {
        let packages = self.run_adb(&format!("-s {} shell pm list packages -3", device))?; // -3 for third party apps
        for line in packages.lines() {
            if let Some(package) = line.strip_prefix("package:") {
                let _ = self.run_adb(&format!("-s {} shell am force-stop {}", device, package.trim()));
            }
        }
        Ok(())
    }

    /// Get app usage statistics
    pub fn get_app_usage_stats(&self, device: &str, days: u32) -> Result<String, ADBError> {
        let end_time = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_millis();
        let start_time = end_time - (days as u128 * 24 * 60 * 60 * 1000);
        self.run_adb(&format!("-s {} shell dumpsys usagestats --proto {} {}", device, start_time, end_time))
    }

    // Helper methods
    fn parse_app_permissions(&self, dumpsys_output: &str, package_name: &str) -> Result<AppPermissions, ADBError> {
        let mut permissions = AppPermissions {
            package_name: package_name.to_string(),
            permissions: Vec::new(),
            requested_permissions: Vec::new(),
            granted_permissions: Vec::new(),
            denied_permissions: Vec::new(),
        };

        for line in dumpsys_output.lines() {
            if line.contains("granted=true") {
                if let Some(perm) = self.extract_permission_from_line(line) {
                    permissions.granted_permissions.push(perm);
                }
            } else if line.contains("granted=false") {
                if let Some(perm) = self.extract_permission_from_line(line) {
                    permissions.denied_permissions.push(perm);
                }
            }
        }

        // All requested permissions are the union of granted and denied
        permissions.requested_permissions = permissions.granted_permissions.clone();
        permissions.requested_permissions.extend(permissions.denied_permissions.clone());

        Ok(permissions)
    }

    fn extract_permission_from_line(&self, line: &str) -> Option<String> {
        line.split("permission.").nth(1)?
            .split_whitespace().next()?
            .to_string()
            .into()
    }

    fn parse_processes(&self, ps_output: &str) -> Result<Vec<ProcessInfo>, ADBError> {
        let mut processes = Vec::new();

        for line in ps_output.lines().skip(1) { // Skip header
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 9 {
                if let Ok(pid) = parts[1].parse::<u32>() {
                    processes.push(ProcessInfo {
                        pid,
                        name: parts[8].to_string(),
                        user: parts[0].to_string(),
                        cpu_usage: 0.0, // Would need additional parsing
                        memory_usage: 0, // Would need additional parsing
                        status: parts[7].to_string(),
                    });
                }
            }
        }

        Ok(processes)
    }

    fn calculate_app_data_size(&self, device: &str, pm_path_output: &str) -> Result<AppDataSize, ADBError> {
        let mut total_size = 0u64;

        for line in pm_path_output.lines() {
            if let Some(path) = line.strip_prefix("package:") {
                // Get size of APK file
                if let Ok(size_output) = self.run_adb(&format!("-s {} shell stat -c%s {}", device, path.trim())) {
                    if let Ok(size) = size_output.trim().parse::<u64>() {
                        total_size += size;
                    }
                }
            }
        }

        Ok(AppDataSize { total_size })
    }

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

    pub fn clear_app_data(&self, device: &str, package_name: &str) -> Result<(), ADBError> {
        self.run_adb(&format!("-s {} shell pm clear {}", device, package_name))?;
        Ok(())
    }

    pub async fn clear_app_data_async(&self, device: &str, package_name: &str) -> Result<(), ADBError> {
        self.run_adb_async(&format!("-s {} shell pm clear {}", device, package_name)).await?;
        Ok(())
    }

    /// Set device animation scale for testing
    pub fn set_animation_scale(&self, device: &str, scale: f32) -> Result<(), ADBError> {
        let scale_str = scale.to_string();
        self.run_adb(&format!("-s {} shell settings put global animator_duration_scale {}", device, scale_str))?;
        self.run_adb(&format!("-s {} shell settings put global transition_animation_scale {}", device, scale_str))?;
        self.run_adb(&format!("-s {} shell settings put global window_animation_scale {}", device, scale_str))?;
        Ok(())
    }
}