use std::time::Duration;
use indicatif::{ProgressBar, ProgressStyle};
use tokio::time::sleep;
use crate::error::ADBError;
use crate::ADB;

pub struct ScreenRecordOptions {
    pub bit_rate: Option<u32>,      // Mbps
    pub resolution: Option<(u32, u32)>,  // Width x Height
    pub time_limit: Option<u32>,    // Seconds
    pub rotate: Option<bool>,
    pub verbose: Option<bool>,
    pub bugreport: Option<bool>,
}

impl Default for ScreenRecordOptions {
    fn default() -> Self {
        Self {
            bit_rate: None,
            resolution: None,
            time_limit: None,
            rotate: None,
            verbose: None,
            bugreport: None,
        }
    }
}

impl ADB {
    pub async fn start_screen_record_with_options(
        &self,
        device: &str,
        output_path: &str,
        options: ScreenRecordOptions
    ) -> Result<(), ADBError> {
        let mut cmd = format!("-s {} shell screenrecord", device);

        if let Some(bit_rate) = options.bit_rate {
            cmd.push_str(&format!(" --bit-rate {}", bit_rate * 1000000));
        }

        if let Some((width, height)) = options.resolution {
            cmd.push_str(&format!(" --size {}x{}", width, height));
        }

        if let Some(time_limit) = options.time_limit {
            cmd.push_str(&format!(" --time-limit {}", time_limit));
        }

        if options.rotate.unwrap_or(false) {
            cmd.push_str(" --rotate");
        }

        if options.verbose.unwrap_or(false) {
            cmd.push_str(" --verbose");
        }

        if options.bugreport.unwrap_or(false) {
            cmd.push_str(" --bugreport");
        }

        cmd.push_str(&format!(" {}", output_path));

        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} [{elapsed_precise}] {msg}")
                .unwrap()
        );
        pb.set_message("Recording screen...");

        let output = self.run_adb_async(&cmd).await?;
        if !output.is_empty() && !output.contains("started") {
            return Err(ADBError::CommandFailed(output));
        }

        // Update progress
        let time_limit = options.time_limit.unwrap_or(180);
        for i in 0..time_limit {
            pb.set_message(format!("Recording: {}/{} seconds", i, time_limit));
            sleep(Duration::from_secs(1)).await;
        }

        pb.finish_with_message("Recording completed!");
        Ok(())
    }

    pub async fn capture_screen_video(
        &self,
        device: &str,
        output_path: &str,
        duration_secs: u32
    ) -> Result<(), ADBError> {
        let options = ScreenRecordOptions {
            time_limit: Some(duration_secs),
            ..Default::default()
        };
        self.start_screen_record_with_options(device, output_path, options).await
    }

    pub async fn capture_screen_video_hd(
        &self,
        device: &str,
        output_path: &str,
        duration_secs: u32
    ) -> Result<(), ADBError> {
        let options = ScreenRecordOptions {
            time_limit: Some(duration_secs),
            resolution: Some((1920, 1080)),
            bit_rate: Some(8),  // 8Mbps
            ..Default::default()
        };
        self.start_screen_record_with_options(device, output_path, options).await
    }
}