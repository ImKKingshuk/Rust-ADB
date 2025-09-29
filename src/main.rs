use rust_adb::ADB;
use std::time::Duration;
use log::LevelFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::Builder::new()
        .filter_level(LevelFilter::Debug)
        .init();

    // Initialize ADB with a 30-second timeout
    let adb = ADB::new(".", Duration::from_secs(30));

    // Start ADB server
    adb.start_server()?;

    // List connected devices
    let devices = adb.refresh_device_list()?;
    println!("Connected devices:");
    for device in &devices {
        println!("Serial: {}, State: {}", device.serial, device.state);
        if let Some(model) = &device.model {
            println!("Model: {}", model);
        }
    }

    // Example: Get package list from first device (if available)
    if let Some(first_device) = devices.first() {
        println!("\nPackages on device {}:", first_device.serial);
        let packages = adb.get_package_list(&first_device.serial)?;
        for package in packages {
            println!("Package: {}", package.name);
        }

        // Example: Get device properties
        println!("\nDevice properties:");
        let props = adb.get_device_props(&first_device.serial)?;
        println!("{}", props);

        // Example: Enable wireless debugging
        if let Ok((ip, port)) = adb.enable_wireless_debugging(&first_device.serial) {
            println!("\nWireless debugging enabled at {}:{}", ip, port);
            // Connect wirelessly
            adb.connect_wireless(&ip, port)?;
        }
    }

    Ok(())
}
