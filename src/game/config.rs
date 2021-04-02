use crate::winit::event::VirtualKeyCode;
use {
    crate::{game::PlayerColor, prelude::*},
    std::{fs::File, io::Write, net::SocketAddrV4, path::Path},
};

#[derive(Serialize, Deserialize, Clone)]
pub struct InputConfig {
    pub exit: VirtualKeyCode,
    pub up: VirtualKeyCode,
    pub down: VirtualKeyCode,
    pub left: VirtualKeyCode,
    pub right: VirtualKeyCode,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub server_mode: bool,
    pub if_client: IfClient,
    pub if_server: IfServer,
    pub input: InputConfig,
}

#[derive(Serialize, Deserialize)]
pub struct IfClient {
    pub server_addr: SocketAddrV4,
    pub preferred_color: PlayerColor,
}

#[derive(Serialize, Deserialize)]
pub struct IfServer {
    pub server_addr: SocketAddrV4,
    pub player_color: PlayerColor,
    pub room_seed: Option<u64>,
    pub ai_enabled: Vec<PlayerColor>,
}

impl Config {
    pub fn try_load_from(path: &Path) -> Option<Self> {
        File::open(path).ok().and_then(|f| ron::de::from_reader(f).ok())
    }
    pub fn write_ron_into(&self, w: impl Write) {
        ron::ser::to_writer_pretty(w, self, ron::ser::PrettyConfig::default()).unwrap();
    }
    pub fn try_save_into(&self, path: &Path) -> bool {
        File::create(path).map(move |f| self.write_ron_into(f)).is_ok()
    }
}
impl Default for Config {
    fn default() -> Self {
        let server_addr = SocketAddrV4::new(std::net::Ipv4Addr::LOCALHOST, 8000);
        use {PlayerColor as Pc, VirtualKeyCode as Vkc};
        Self {
            server_mode: true,
            if_client: IfClient { preferred_color: PlayerColor::Black, server_addr },
            if_server: IfServer {
                room_seed: None,
                player_color: PlayerColor::Black,
                server_addr,
                ai_enabled: vec![Pc::Black, Pc::Blue, Pc::Orange],
            },
            input: InputConfig {
                up: Vkc::W,
                down: Vkc::S,
                left: Vkc::A,
                right: Vkc::D,
                exit: Vkc::Escape,
            },
        }
    }
}
