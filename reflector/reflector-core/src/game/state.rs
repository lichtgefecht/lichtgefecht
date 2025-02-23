use std::collections::HashMap;

pub struct Game {}

pub struct Team {}

#[derive(Debug)]
#[allow(dead_code)]
pub struct Device {
    pub(crate) hid: String,
    pub(crate) device_type: i32,
}

pub struct Player {}

pub struct Binding {}

#[derive(Default, Debug)]
pub struct State {
    pub devices: HashMap<String, Device>,
}
