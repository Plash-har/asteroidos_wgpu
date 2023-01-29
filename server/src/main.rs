use std::time::Duration;

use game_logic::World;

mod interface;

const WORLD_UPD_RATE: Duration = Duration::from_millis(1000 / 60);  // 60 times per second

fn main() {
    let mut world = World::new(3, 3);

    for _x in 0..100 {
        world.update();
    }
}
