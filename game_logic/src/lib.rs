pub mod asteroids;
mod player;

use std::time::Instant;

pub use asteroids::{Asteroid, AsteroidManager};
pub use player::Player;

pub struct World {
    pub players:  Vec<Player>,
    last_upd: Instant,
    start_upd: Instant,
    pub n_player_img: i32,
    pub n_asteroid_img: i32,

    pub asteroids: Vec<Asteroid>,
    asteroid_manager: AsteroidManager,
}

impl World {
    pub fn new(n_asteroid_img: i32, n_player_img: i32) -> World {

        return World {
            asteroids: vec![
                Asteroid { pos: cgmath::Point2 { x: 2., y: 0. }, rot: 0., rot_speed: 1., vel: cgmath::Vector2 { x: 0., y: 0.}, img_idx: 1, spawn_time: 0. }
            ],
            players: vec![Player::new(), Player::new()],
            last_upd: Instant::now(),
            start_upd: Instant::now(),
            n_player_img,
            n_asteroid_img,
            asteroid_manager: AsteroidManager::new(),
        }
    }

    pub fn new_img_auto() -> World {
        return World { 
            players: vec![Player::new(), Player::new()], 
            last_upd: Instant::now(), 
            start_upd: Instant::now(), 
            n_player_img: player::get_n_player_img(), 
            n_asteroid_img: asteroids::get_n_asteroid_img(), 
            asteroids: Vec::new(), 
            asteroid_manager: AsteroidManager::new(),   
        };
    }
    
    pub fn update(&mut self) {
        let delta_t = self.last_upd.elapsed().as_secs_f64();
        self.last_upd = Instant::now();
        let time = self.start_upd.elapsed().as_secs_f32();
        
        asteroids::update_asteroids(self, delta_t);
        player::update_players(self, delta_t);

        self.asteroid_manager.clean_asteroids(&mut self.asteroids, &self.players);
        self.asteroid_manager.add_asteroids(&mut self.asteroids, &self.players, time, self.n_asteroid_img);
    }
}