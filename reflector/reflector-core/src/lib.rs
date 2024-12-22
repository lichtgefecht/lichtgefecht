use log::info;

pub struct Game {}

pub struct Team {}

pub struct Device {}

pub struct Player {}

pub struct Binding {}

pub struct Core {}
impl Core {
    pub fn on_message_received(&self) {
        info!("Received message")
    }
}
