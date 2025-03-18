use crate::error::ADBError;
use crate::ADB;

#[derive(Debug)]
pub enum InputSource {
    Touchscreen,
    Keyboard,
    Mouse,
    Joystick,
}

#[derive(Debug)]
pub struct TouchEvent {
    pub x: i32,
    pub y: i32,
    pub pressure: Option<i32>,
    pub size: Option<i32>,
}

impl ADB {
    pub fn send_keyevent(&self, device: &str, keycode: i32) -> Result<(), ADBError> {
        self.run_adb(&format!("-s {} shell input keyevent {}", device, keycode))?;
        Ok(())
    }

    pub fn send_text(&self, device: &str, text: &str) -> Result<(), ADBError> {
        self.run_adb(&format!("-s {} shell input text {}", device, text))?;
        Ok(())
    }

    pub fn tap(&self, device: &str, x: i32, y: i32) -> Result<(), ADBError> {
        self.run_adb(&format!("-s {} shell input tap {} {}", device, x, y))?;
        Ok(())
    }

    pub fn swipe(&self, device: &str, x1: i32, y1: i32, x2: i32, y2: i32, duration_ms: Option<u32>) -> Result<(), ADBError> {
        let cmd = if let Some(duration) = duration_ms {
            format!("-s {} shell input swipe {} {} {} {} {}", device, x1, y1, x2, y2, duration)
        } else {
            format!("-s {} shell input swipe {} {} {} {}", device, x1, y1, x2, y2)
        };
        self.run_adb(&cmd)?;
        Ok(())
    }

    pub fn press_and_hold(&self, device: &str, x: i32, y: i32, duration_ms: u32) -> Result<(), ADBError> {
        self.swipe(device, x, y, x, y, Some(duration_ms))
    }

    pub fn send_touch_event(&self, device: &str, event: TouchEvent, source: InputSource) -> Result<(), ADBError> {
        let source_str = match source {
            InputSource::Touchscreen => "touchscreen",
            InputSource::Mouse => "mouse",
            InputSource::Keyboard => "keyboard",
            InputSource::Joystick => "joystick",
        };

        let mut cmd = format!("-s {} shell sendevent {} {} {}", 
            device, source_str, event.x, event.y);

        if let Some(pressure) = event.pressure {
            cmd.push_str(&format!(" {}", pressure));
        }

        if let Some(size) = event.size {
            cmd.push_str(&format!(" {}", size));
        }

        self.run_adb(&cmd)?;
        Ok(())
    }

    pub async fn send_keyevent_async(&self, device: &str, keycode: i32) -> Result<(), ADBError> {
        self.run_adb_async(&format!("-s {} shell input keyevent {}", device, keycode)).await?;
        Ok(())
    }

    pub async fn send_text_async(&self, device: &str, text: &str) -> Result<(), ADBError> {
        self.run_adb_async(&format!("-s {} shell input text {}", device, text)).await?;
        Ok(())
    }

    pub async fn tap_async(&self, device: &str, x: i32, y: i32) -> Result<(), ADBError> {
        self.run_adb_async(&format!("-s {} shell input tap {} {}", device, x, y)).await?;
        Ok(())
    }

    pub async fn swipe_async(&self, device: &str, x1: i32, y1: i32, x2: i32, y2: i32, duration_ms: Option<u32>) -> Result<(), ADBError> {
        let cmd = if let Some(duration) = duration_ms {
            format!("-s {} shell input swipe {} {} {} {} {}", device, x1, y1, x2, y2, duration)
        } else {
            format!("-s {} shell input swipe {} {} {} {}", device, x1, y1, x2, y2)
        };
        self.run_adb_async(&cmd).await?;
        Ok(())
    }

    pub async fn press_and_hold_async(&self, device: &str, x: i32, y: i32, duration_ms: u32) -> Result<(), ADBError> {
        self.swipe_async(device, x, y, x, y, Some(duration_ms)).await
    }
}