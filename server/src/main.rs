use game_logic::World;

fn main() {
    let mut world = World::new();

    for _x in 0..100 {
        world.update();
    }
}
