use std::fs;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub transport: TransportConfig,
}

#[derive(Debug, Deserialize)]
pub struct TransportConfig {
    pub hid: String,
    pub bind_addr: SocketAddrConfig,
    pub advertise_addr: Option<SocketAddrConfig>,
}
#[derive(Debug, Deserialize)]
pub struct SocketAddrConfig {
    pub ip: String,
    pub port: u16,
}

pub fn read_config() -> Config {
    let data = fs::read_to_string("Reflector.toml").expect("Unable to read file");
    let config = toml::from_str(&data).expect("Can't parse config");
    return config;
}
