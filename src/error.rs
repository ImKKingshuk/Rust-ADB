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
}

pub type Result<T> = std::result::Result<T, ADBError>;