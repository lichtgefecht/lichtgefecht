use std::sync::atomic::AtomicBool;

use log::info;

pub struct Game {}

pub struct Team {}

pub struct Device {}

pub struct Player {}

pub struct Binding {}

pub struct Core {
    running: AtomicBool
}
impl Core {
    pub fn new() -> Self {
        Core {
            running: AtomicBool::new(true)
        }
    }

    pub fn on_message_received(&self) {
        info!("Received message, temp {}", self.running.load(std::sync::atomic::Ordering::Relaxed))
    }
}

impl Default for Core {
    fn default() -> Self {
        Self::new()
    }
}