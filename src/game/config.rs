use crate::{game::PlayerColor, prelude::*};
use std::net::SocketAddrV4;
#[derive(Serialize, Deserialize)]
pub struct Config {
    pub preferred_player_color: PlayerColor,
    pub server_mode: bool,
    pub server_addr_if_server: SocketAddrV4,
    pub server_addr_if_client: SocketAddrV4,
}
