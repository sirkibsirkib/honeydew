////////////////////////////
mod rendering;
mod rng;
mod room;

use crate::room::Room;
use gfx_2020::{DrawInfo, TexId};
use gfx_backend_vulkan::Backend as VulkanBackend;
/////////////////////////////////

struct GameState {
    room: Room,
    tex_id: TexId,
    draw_infos: [DrawInfo; 1],
}

fn main() {
    gfx_2020::main_loop::<VulkanBackend, _, _>(
        &rendering::render_config(),
        rendering::game_state_init_fn,
    )
}
