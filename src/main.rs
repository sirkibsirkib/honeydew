////////////////////////////
mod rendering;
mod rng;
mod room;
use crate::room::RoomData;
use gfx_2020::{DrawInfo, TexId};
use gfx_backend_vulkan::Backend as VulkanBackend;
/////////////////////////////////

struct GameState {
    room_data: RoomData,
    tex_id: TexId,
    draw_infos: [DrawInfo; 1],
}

fn main() {
    gfx_2020::main_loop::<VulkanBackend, _, _>(&rendering::render_config(), |x| {
        Ok(rendering::heap_leak(rendering::game_state_init_fn(x)))
    })
}
