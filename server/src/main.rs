use game_logic::World;

fn main() {
    let mut world = World::new(3, 3);

    for _x in 0..100 {
        world.update();
    }
}
