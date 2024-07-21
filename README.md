# Rust-ADB

Rust-ADB is a Rust implementation of the Android Debug Bridge (ADB) library. It provides a convenient and platform-independent way to interact with Android devices through ADB commands.

## Features

- **Device Connection:** Retrieve the list of connected Android devices with detailed information.

- **Server Management:** Start, stop, and restart the ADB server.

- **Screen Manipulation:** Set screen size, density, capture screenshots, and more.

- **Package Information:** Get information about installed packages.

- **Input Events:** Send input events to the device.

- **Document UI:** Open the Android document UI for specified paths.

- **Log Management:** Clear logcat logs.

## Installation

Add the following dependency to your `Cargo.toml` file:

```toml
[dependencies]
rust-adb = "0.1.0"
```

## Contributing

Contributions are welcome! If you have improvements, bug fixes, or new features to propose, please open an issue or submit a pull request.
