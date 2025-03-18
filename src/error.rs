use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ADBError {
    #[error("IO error: {0}")]
    IO(#[from] io::Error),

    #[error("Device not found: {0}")]
    DeviceNotFound(String),

    #[error("Command failed: {0}")]
    CommandFailed(String),

    #[error("Timeout error: {0}")]
    Timeout(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Screen recording error: {0}")]
    ScreenRecording(String),

    #[error("System info error: {0}")]
    SystemInfo(String),

    #[error("Battery info error: {0}")]
    BatteryInfo(String),

    #[error("Package management error: {0}")]
    PackageManagement(String),

    #[error("Debug error: {0}")]
    Debug(String),

    #[error("Input event error: {0}")]
    InputEvent(String),

    #[error("Connection timeout: {0}")]
    ConnectionTimeout(String),

    #[error("Connection retry failed: {0}")]
    ConnectionRetry(String),

    #[error("Wireless connection error: {0}")]
    WirelessConnection(String),

    #[error("File transfer error: {0}")]
    FileTransfer(String),
    Package(String),

    #[error("File transfer error: {0}")]
    FileTransfer(String),

    #[error("Wireless connection error: {0}")]
    WirelessConnection(String),

    #[error("Device property error: {0}")]
    DeviceProperty(String),

    #[error("Connection retry error: {0}")]
    ConnectionRetry(String),

    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Device authorization error: {0}")]
    Authorization(String),

    #[error("Package installation error: {0}")]
    PackageInstallation(String),

    #[error("Package uninstallation error: {0}")]
    PackageUninstallation(String),

    #[error("File transfer error: {0}")]
    FileTransfer(String),

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("Device state error: {0}")]
    DeviceState(String),

    #[error("Wireless connection error: {0}")]
    WirelessConnection(String),

    #[error("Screen capture error: {0}")]
    ScreenCapture(String),

    #[error("Shell execution error: {0}")]
    ShellExecution(String),

    #[error("Logcat error: {0}")]
    Logcat(String),

    #[error("Backup error: {0}")]
    Backup(String),

    #[error("Restore error: {0}")]
    Restore(String),

    #[error("Split APK installation error: {0}")]
    SplitPackageInstallation(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),
}

pub type Result<T> = std::result::Result<T, ADBError>;