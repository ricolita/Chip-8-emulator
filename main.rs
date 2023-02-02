mod chip_8;

use macroquad::prelude::*;
use chip_8::{Chip, QUAD};
 
const SCREEN_LAG: i32 = QUAD as i32 * 64;
const SCREEN_ALT: i32 = QUAD as i32 * 32;

fn window_conf() -> Conf {
    Conf {
        window_title: "chip-8".to_owned(),
        window_width: SCREEN_LAG,
        window_height: SCREEN_ALT,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {

    let mut chip = Chip::new();
    chip.load_game();

    loop {
        clear_background(BLACK);

        chip.emular();
        chip.render();
        if chip.dt > 0 {
            chip.dt -= 1;
        }
        if chip.st > 0 {
            chip.st -= 1;
        }
        next_frame().await;
    }

}
