//! Linux implementation of the OSD (On-Screen Display) manager.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use nucleus_platform::OsdManager;
use tracing::info;

/// Linux implementation of the `OsdManager` trait.
pub struct LinuxOsdManager {
    current: Arc<Mutex<Option<(String, Instant, Duration)>>>,
    color: Arc<Mutex<String>>,
}

impl LinuxOsdManager {
    /// Creates a new `LinuxOsdManager`.
    pub fn new() -> Self {
        Self {
            current: Arc::new(Mutex::new(None)),
            color: Arc::new(Mutex::new("255,255,255".to_string())),
        }
    }
}

impl Default for LinuxOsdManager {
    fn default() -> Self {
        Self::new()
    }
}

impl OsdManager for LinuxOsdManager {
    fn show(&self, duration_ms: u32, text: &str) {
        info!("[OSD] {} (displaying for {}ms)", text, duration_ms);
        let duration = Duration::from_millis(duration_ms as u64);
        if let Ok(mut cur) = self.current.lock() {
            *cur = Some((text.to_string(), Instant::now(), duration));
        }
    }

    fn hide(&self) {
        if let Ok(mut cur) = self.current.lock() {
            *cur = None;
        }
    }

    fn current_message(&self) -> Option<String> {
        if let Ok(mut cur) = self.current.lock() {
            if let Some((msg, start, dur)) = cur.as_ref() {
                if start.elapsed() < *dur {
                    return Some(msg.clone());
                } else {
                    *cur = None;
                }
            }
        }
        None
    }

    fn set_color(&self, color: &str) {
        if let Ok(mut c) = self.color.lock() {
            *c = color.to_string();
        }
    }
}
