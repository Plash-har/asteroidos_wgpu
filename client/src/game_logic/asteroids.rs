use cgmath::{Point2, Vector2};
use rand::Rng;

use super::{World, Player};

use std::f32::consts::PI;
use fnv::FnvHashMap as HashMap;

use crate::logger::{info, unexpected};

#[derive(Debug, Clone, Copy)]
pub struct Asteroid {
    pub pos: Point2<f64>,
    pub vel: Vector2<f64>,
    pub rot_speed: f32,
    pub rot: f32,
    pub img_idx: i32,
    pub spawn_time: f32,
}

pub fn update_asteroids(world: &mut World, delta_t: f64) {
    let f32_delta_t = delta_t as f32;
    
    for x  in &mut world.asteroids {
        x.rot += x.rot_speed * f32_delta_t;

        x.pos += x.vel * delta_t;
    }
}

const AST_SPEED_MAX: f64 = 0.1; // The extreme of what the random speed of an asteroid can be
const AST_ROT_SPEED_MAX: f32 = 1.; // The extreme of what the random speed of rotation of an asteroid can be

const CHUNK_SIZE: f64 = 2.;
const CHUNK_PLAYER_DIST: i64 = 6;  /// The size of the chunks that will be checked around the playery
const ASTEROID_DESCPAWN_DIST: f64 = CHUNK_PLAYER_DIST as f64 * CHUNK_SIZE;

/// Spawns a desired amount of asteroids in a desired chunk of space
pub fn spawn_ast_in_chunk(asteroids: &mut Vec<Asteroid>, n_ast_img: i32, n: usize, chunk: (i64, i64), time: f32) {
    let mut to_add = Vec::with_capacity(n as usize);

    let mut thread_rng = rand::thread_rng();

    let chunk = cgmath::Vector2 { x: chunk.0 as f64 * CHUNK_SIZE, y: chunk.1 as f64 * CHUNK_SIZE };

    for _x in 0..n {
        let pos = cgmath::Point2 { x: rand::random::<f64>() * CHUNK_SIZE, y: rand::random::<f64>() * CHUNK_SIZE } + chunk;

        let ast = Asteroid { 
            pos, 
            vel: cgmath::Vector2 { 
                x: thread_rng.gen_range(-AST_SPEED_MAX..AST_SPEED_MAX), 
                y: thread_rng.gen_range(-AST_SPEED_MAX..AST_SPEED_MAX) 
            }, 
            rot_speed: thread_rng.gen_range(-AST_ROT_SPEED_MAX..AST_ROT_SPEED_MAX), 
            rot: thread_rng.gen_range(-PI..PI), 
            img_idx: thread_rng.gen_range(0..n_ast_img), 
            spawn_time: time,
        };

        to_add.push(ast);
    }

    asteroids.append(&mut to_add);
}

pub struct AsteroidManager {
    last_del_idx: usize,
    // Holds a rough estimate to how many asteroids there are in a chunk
    chunk_counter: HashMap<(i64, i64), usize>,
}

impl AsteroidManager {
    pub fn new() -> AsteroidManager {
        return AsteroidManager { last_del_idx: 0, chunk_counter: HashMap::default() };
    }

    pub fn add_asteroids(&mut self, asteroids: &mut Vec<Asteroid>, players: &Vec<Player>, time: f32, n_ast_img: i32) {
        for player in players {
            let pos = chunk_pos_from_pos(player.pos);

            for x in (pos.0 - CHUNK_PLAYER_DIST)..(pos.0 + CHUNK_PLAYER_DIST) {
                for y in (pos.1 - CHUNK_PLAYER_DIST)..(pos.1 + CHUNK_PLAYER_DIST) {
                    if let None = self.chunk_counter.get(&(x, y)) {  // So if the chunk hasn't been generated
                        let n_ast_expected = get_n_ast_in_chunk((x, y));
                        spawn_ast_in_chunk(asteroids, n_ast_img, n_ast_expected, (x, y), time);

                        info(4, format!("Generated chunk at {} {} with {} asteroids", x, y, n_ast_expected));

                        self.chunk_counter.insert((x, y), n_ast_expected);
                    }
                }
            }
        }
    }

    /// The max number of asteroids that can be checked by the cleaning
    /// To prevent big CPU usage at high number of asteroids & players
    const N_UPDATES_FRAMES: usize = 500;

    pub fn clean_asteroids(&mut self, asteroids: &mut Vec<Asteroid>, players: &[Player]) {
        let mut ast_to_dispose = Vec::new();

        if asteroids.len() > Self::N_UPDATES_FRAMES {
            for (idx, ast) in asteroids.iter().enumerate() {
                let mut to_dispose = true;

                for player in players {
                    if player.pos.x - ast.pos.x < ASTEROID_DESCPAWN_DIST && player.pos.y - ast.pos.y < ASTEROID_DESCPAWN_DIST {
                        to_dispose = false;
                        break;
                    }
                }

                if to_dispose {
                    ast_to_dispose.push(idx);
                }
            }
        } else {
            let mut n_updates_now = 0;

            while n_updates_now < AsteroidManager::N_UPDATES_FRAMES {
                let idx = (n_updates_now + self.last_del_idx) % asteroids.len();
    
                let ast = asteroids[idx];
    
                let mut to_dispose = true;
    
                for player in players {
                    if player.pos.x - ast.pos.x < ASTEROID_DESCPAWN_DIST && player.pos.y - ast.pos.y < ASTEROID_DESCPAWN_DIST {
                        to_dispose = false;
                        break;
                    }
                }

                if to_dispose {
                    ast_to_dispose.push(idx);
                }
    
                n_updates_now += 1;
            }

            self.last_del_idx += n_updates_now;
        }

        for idx in ast_to_dispose.iter().rev() {
            asteroids.swap_remove(*idx);
        }

        self.clean_chunks(players);
    }

    /// To prevent big memory usage and cpu usage by removing chunks that are too far away
    fn clean_chunks(&mut self, players: &[Player]) {
        let players: Vec<_> = players.iter().map(|player |chunk_pos_from_pos(player.pos)).collect();

        let mut to_remove = Vec::new();

        for (key, _val) in &self.chunk_counter {
            let mut task_to_remove = true;
            
            for pos in &players {
                if key.0 - pos.0 < CHUNK_PLAYER_DIST && key.1 - pos.1 < CHUNK_PLAYER_DIST {
                    task_to_remove = false;
                    break;
                }
            }
            
            if task_to_remove {
                to_remove.push(*key);
            }
        }

        for key in to_remove.iter().rev() {
            let val = self.chunk_counter.remove(key);
            if let Some(_val) = val {
                info(4, format!("Removed chunk {:?}", *key));
            } else {
                unexpected(4, "Removed a non existing asteroid chunk");
            }
        }
    }
}

pub fn chunk_pos_from_pos(pos: cgmath::Point2<f64>) -> (i64, i64) {
    return ((pos.x / CHUNK_SIZE) as i64, (pos.y / CHUNK_SIZE) as i64);
}

/// Returns the expected amount of asteroids in a given chunk
fn get_n_ast_in_chunk(pos: (i64, i64)) -> usize {
    let pos = (pos.0.abs() as usize, pos.1.abs() as usize);
    return pos.0 + pos.1 ^ 2;
}