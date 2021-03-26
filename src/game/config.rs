use crate::{game::PlayerColor, prelude::*};
use core::time::Duration;
use std::net::SocketAddrV4;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub server_mode: bool,
    pub if_client: IfClient,
    pub if_server: IfServer,
}

#[derive(Serialize, Deserialize)]
pub struct IfClient {
    pub server_addr: SocketAddrV4,
    pub preferred_color: PlayerColor,
    pub connect_timeout: Duration,
}

#[derive(Serialize, Deserialize)]
pub struct IfServer {
    pub server_addr: SocketAddrV4,
    pub preferred_color: PlayerColor,
}
