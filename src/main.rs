mod axes;
mod bit_set;
mod game;
mod prelude;
mod rng;
mod wrap_int;

use {
    crate::{
        game::{config::Config, rendering::render_config, GameState},
        prelude::*,
    },
    gfx_2020::{gfx_hal::Backend, *},
    std::path::Path,
};
/////////////////////////////////

pub(crate) fn game_state_init_fn<B: Backend>(
    renderer: &mut Renderer<B>,
) -> ProceedWith<&'static mut GameState> {
    let maybe_arg = std::env::args().nth(1);
    let config_path = if let Some(arg) = maybe_arg.as_ref() {
        Path::new(arg)
    } else {
        Path::new("./honeydew_config.ron")
    };
    let config = Config::try_load_from(config_path).unwrap_or_else(move || {
        println!("No config found at {:?}. Generating default!", config_path.canonicalize());
        let config = Config::default();
        config.try_save_into(config_path);
        config
    });
    {
        let stdio = std::io::stdout();
        let mut stdio = stdio.lock();
        use std::io::Write;
        writeln!(stdio, "Beginning game with config ").unwrap();
        config.write_ron_into(stdio);
    }
    Ok(Box::leak(Box::new(GameState::new(renderer, &config))))
}

fn main() {
    gfx_2020::main_loop::<gfx_backend::Backend, _, _>(&render_config(), game_state_init_fn);
}
