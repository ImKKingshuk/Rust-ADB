// src/main.rs

use rust_adb::ADB;
use std::time::Duration;

#[tokio::main]
async fn main() {
    let adb = ADB::new("/path/to/adb", Duration::from_secs(10));

    match adb.refresh_device_list_async().await {
        Ok(devices) => {
            println!("Connected devices:");
            for device in devices {
                println!("{}", device);
            }
        }
        Err(err) => eprintln!("Error: {}", err),
    }

    match adb.get_screenshot_png_async("device_id_or_serial").await {
        Ok(screenshot) => {
            println!("Screenshot received: {} bytes", screenshot.len());
        }
        Err(err) => eprintln!("Error getting screenshot: {}", err),
    }

    // Example usage of install_app and logcat
    match adb.install_app_async("device_id_or_serial", "/path/to/app.apk").await {
        Ok(result) => println!("App installed: {}", result),
        Err(err) => eprintln!("Error installing app: {}", err),
    }

    match adb.logcat_async("device_id_or_serial").await {
        Ok(logs) => println!("Logcat output: {}", logs),
        Err(err) => eprintln!("Error getting logcat output: {}", err),
    }
}
