use std::path::Path;
use crate::error::ADBError;
use crate::ADB;

impl ADB {
    pub fn push_file(&self, device: &str, local_path: &str, remote_path: &str) -> Result<(), ADBError> {
        if !Path::new(local_path).exists() {
            return Err(ADBError::FileTransfer(format!("Local file not found: {}", local_path)));
        }
        let output = self.run_adb(&format!("-s {} push {} {}", device, local_path, remote_path))?;
        if !output.contains("pushed") && !output.contains("transferred") {
            return Err(ADBError::FileTransfer(output));
        }
        Ok(())
    }

    pub async fn push_file_async(&self, device: &str, local_path: &str, remote_path: &str) -> Result<(), ADBError> {
        if !Path::new(local_path).exists() {
            return Err(ADBError::FileTransfer(format!("Local file not found: {}", local_path)));
        }
        let output = self.run_adb_async(&format!("-s {} push {} {}", device, local_path, remote_path)).await?;
        if !output.contains("pushed") && !output.contains("transferred") {
            return Err(ADBError::FileTransfer(output));
        }
        Ok(())
    }

    pub fn pull_file(&self, device: &str, remote_path: &str, local_path: &str) -> Result<(), ADBError> {
        let output = self.run_adb(&format!("-s {} pull {} {}", device, remote_path, local_path))?;
        if !output.contains("pulled") && !output.contains("transferred") {
            return Err(ADBError::FileTransfer(output));
        }
        Ok(())
    }

    pub async fn pull_file_async(&self, device: &str, remote_path: &str, local_path: &str) -> Result<(), ADBError> {
        let output = self.run_adb_async(&format!("-s {} pull {} {}", device, remote_path, local_path)).await?;
        if !output.contains("pulled") && !output.contains("transferred") {
            return Err(ADBError::FileTransfer(output));
        }
        Ok(())
    }

    pub fn shell_command(&self, device: &str, command: &str) -> Result<String, ADBError> {
        self.run_adb(&format!("-s {} shell {}", device, command))
    }

    pub async fn shell_command_async(&self, device: &str, command: &str) -> Result<String, ADBError> {
        self.run_adb_async(&format!("-s {} shell {}", device, command)).await
    }

    pub fn get_screenshot_png(&self, device: &str) -> Result<Vec<u8>, ADBError> {
        let output = self.run_adb(&format!("-s {} exec-out screencap -p", device))?;
        Ok(output.into_bytes())
    }

    pub async fn get_screenshot_png_async(&self, device: &str) -> Result<Vec<u8>, ADBError> {
        let output = self.run_adb_async(&format!("-s {} exec-out screencap -p", device)).await?;
        Ok(output.into_bytes())
    }
}