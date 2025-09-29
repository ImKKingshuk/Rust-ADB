# Rust-ADB

Rust-ADB is a comprehensive, high-performance Rust library for Android Debug Bridge (ADB) operations. It provides a platform-independent, async-first API for automating Android device interactions, testing, and development workflows.

## Features

### Core ADB Operations
- **Server Management**: Start, stop, and kill ADB server with force options
- **Device Discovery**: List connected devices with detailed metadata (serial, state, model, transport ID)
- **Device Properties**: Get and set system properties with advanced property management
- **Shell Commands**: Execute arbitrary shell commands with timeout and retry logic

### Package Management
- **Package Listing**: Retrieve detailed package information with version data
- **App Installation/Uninstallation**: Install APKs with various options, uninstall packages
- **Package Information**: Get detailed package metadata including permissions and install times
- **App Data Management**: Clear app data, calculate app sizes, backup/restore functionality

### File Operations
- **File Transfer**: Push and pull files between host and device
- **Screenshot Capture**: Take PNG screenshots with async support
- **File System Operations**: List directories, check file existence, get file sizes

### Input Simulation
- **Touch Events**: Tap, swipe, press-and-hold gestures
- **Key Events**: Send keycodes and text input
- **Custom Input**: Send raw input events with configurable sources

### Debugging & Monitoring
- **Logcat**: Advanced logcat monitoring with presets (errors, performance, network)
- **Performance Profiling**: CPU, memory, battery, and network statistics
- **Memory Analysis**: Capture heap dumps and memory usage tracking
- **Process Monitoring**: List running processes with resource usage
- **System Tracing**: Capture system traces with configurable duration
- **ANR Analysis**: Get and clear Application Not Responding traces

### Network & Connectivity
- **Wireless Debugging**: Enable/disable wireless ADB with automatic IP detection
- **Port Forwarding**: Forward local ports to device ports (and reverse)
- **Network Diagnostics**: WiFi, mobile data, and connectivity status
- **Connectivity Testing**: Test network reachability to specific hosts
- **Development Setup**: Automated port forwarding for common dev servers

### Screen & Media
- **Screen Recording**: Record device screen with customizable bitrate, resolution, and duration
- **HD Recording**: Preset configurations for high-definition capture
- **Progress Tracking**: Visual progress bars for long-running operations

### Device Management
- **App Lifecycle**: Start, stop, and force-stop applications
- **Process Control**: Kill processes by PID or name
- **Permission Management**: Grant, revoke, and reset app permissions
- **Component Control**: Enable/disable app components
- **Device Control**: Reboot device (normal, recovery, bootloader modes)
- **Animation Control**: Adjust animation scales for testing
- **Mock Location**: Enable/disable location mocking with coordinate setting

### System Information
- **Hardware Info**: CPU architecture, kernel version, bootloader info
- **Software Info**: Android version, SDK level, security patches
- **Battery Status**: Level, temperature, charging status, health metrics
- **Storage Info**: Available space, total capacity, usage statistics
- **Memory Info**: RAM usage, available memory, memory pressure

### Automation & Batch Operations
- **Batch Command Execution**: Run multiple commands with result aggregation
- **Workflow Management**: Define and execute complex multi-step workflows
- **Script Automation**: JSON-based automation scripts for testing scenarios
- **Parallel Execution**: Run commands concurrently for efficiency
- **Conditional Logic**: Execute commands based on previous step results
- **Progress Reporting**: Detailed execution results with timing information

### Advanced Features
- **Device Mirroring**: Screen mirroring capabilities (requires additional setup)
- **Backup/Restore**: Full device and app-specific backup operations
- **Usage Statistics**: App usage tracking and analytics
- **Development Helpers**: Automated setup for development environments
- **Cross-Platform**: Works on Windows, macOS, and Linux

## Installation

Add the following dependency to your `Cargo.toml` file:

```toml
[dependencies]
rust-adb = "0.1.0"
```

## Quick Start

```rust
use rust_adb::ADB;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize ADB
    let adb = ADB::new(".", Duration::from_secs(30));

    // Start ADB server
    adb.start_server()?;

    // List connected devices
    let devices = adb.refresh_device_list()?;
    println!("Found {} device(s)", devices.len());

    if let Some(device) = devices.first() {
        // Get device info
        let props = adb.get_device_props(&device.serial)?;
        println!("Device properties: {}", props);

        // Get system info
        let sys_info = adb.get_system_info(&device.serial)?;
        println!("Android version: {}", sys_info.android_version);

        // List installed packages
        let packages = adb.get_package_list(&device.serial)?;
        println!("Installed packages: {}", packages.len());

        // Take screenshot
        let screenshot = adb.get_screenshot_png(&device.serial)?;
        println!("Screenshot captured: {} bytes", screenshot.len());
    }

    Ok(())
}
```

## Advanced Usage Examples

### Logcat Monitoring with Presets
```rust
use rust_adb::{ADB, LogcatPreset};

// Use predefined logcat preset for error monitoring
let error_preset = LogcatPreset::error_only();
adb.watch_logcat_preset(device_id, error_preset)?;

// Create custom logcat configuration
let options = LogcatOptions {
    filters: vec!["MyApp:D".to_string(), "*:E".to_string()],
    format: Some("time".to_string()),
    clear: true,
    ..Default::default()
};
adb.watch_logcat(device_id, options)?;
```

### Performance Profiling
```rust
// Get current performance metrics
let profile = adb.get_performance_profile(device_id)?;
println!("CPU: {:.1}%, Memory: {}KB", profile.cpu_usage, profile.memory_usage);

// Start method profiling for an app
adb.start_method_profiling(device_id, "com.example.app")?;
// ... run your test scenario ...
adb.stop_method_profiling(device_id, "com.example.app")?;
```

### Network Diagnostics
```rust
// Get network information
let net_info = adb.get_network_diagnostics(device_id)?;
println!("WiFi: {}, IP: {:?}", net_info.wifi_enabled,
         net_info.ip_routes.lines().next().unwrap_or("None"));

// Test connectivity
let reachable = adb.test_connectivity(device_id, "8.8.8.8", Some(53))?;
println!("Google DNS reachable: {}", reachable);
```

### Automation Scripts
```rust
use rust_adb::AutomationScript;

// Define automation tasks
let script = AutomationScript {
    name: "Test Setup".to_string(),
    tasks: vec![
        AutomationTask {
            name: "Device Setup".to_string(),
            task_type: TaskType::DeviceSetup,
            device: device_id.to_string(),
            ..Default::default()
        },
        AutomationTask {
            name: "Install App".to_string(),
            task_type: TaskType::AppInstallation,
            device: device_id.to_string(),
            app_path: Some("myapp.apk".to_string()),
            ..Default::default()
        },
        AutomationTask {
            name: "Performance Test".to_string(),
            task_type: TaskType::PerformanceTest,
            device: device_id.to_string(),
            duration_secs: Some(60),
            ..Default::default()
        },
    ],
};

// Execute automation script
let result = adb.run_automation_script(script)?;
println!("Automation completed: {}", result.success);
```

### Batch Operations
```rust
// Execute multiple commands
let commands = vec![
    "shell getprop ro.build.version.release",
    "shell pm list packages -f | wc -l",
    "shell dumpsys battery | grep level",
];

let batch_result = adb.execute_batch_commands(device_id, &commands)?;
println!("Batch completed: {}/{} successful",
         batch_result.successful, batch_result.total_commands);
```

## API Reference

### Core Types
- `ADB`: Main struct for all ADB operations
- `Device`: Connected device information
- `ADBError`: Comprehensive error type with specific error variants

### Key Modules
- **Device Management**: `AppPermissions`, `ProcessInfo`, `AppDataSize`
- **Debugging**: `LogcatOptions`, `LogcatPreset`, `PerformanceProfile`
- **Networking**: `NetworkDiagnostics`, port forwarding utilities
- **Automation**: `AutomationScript`, `Workflow`, batch operation types

### Error Handling
All operations return `Result<T, ADBError>` with detailed error information:
- `IO`: File system and I/O errors
- `CommandFailed`: ADB command execution failures
- `Timeout`: Operation timeout errors
- `DeviceNotFound`: Invalid device references
- `Parse`: Data parsing errors
- And many more specific error types

## Platform Support

- **Windows**: Full support with `adb.exe`
- **macOS**: Full support with `adb`
- **Linux**: Full support with `adb`

## Contributing

Contributions are welcome! Areas for improvement:
- Additional platform-specific features
- More comprehensive error handling
- Performance optimizations
- Additional automation workflows
- CLI interface enhancements


