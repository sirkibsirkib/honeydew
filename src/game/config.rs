use crate::{game::PlayerColor, prelude::*};

#[derive(Serialize, Deserialize)]
pub struct Config {
    preferred_player_color: PlayerColor,
    sc_mode: ScMode,
    client: AsClientConfig,
    server: AsServerConfig,
}

#[derive(Serialize, Deserialize)]
pub enum ScMode {
    Client,
    Server,
}

#[derive(Serialize, Deserialize)]
pub struct AsClientConfig {
    server_ip: Ipv4,
    s_port: u16,
}

#[derive(Serialize, Deserialize)]
pub struct Ipv4 {
    addr: [u8; 4],
    port: u16,
}

#[derive(Serialize, Deserialize)]
pub struct AsServerConfig {
    server_ip: Ipv4,
}
