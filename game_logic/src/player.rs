use super::World;

#[derive(Debug, Clone, Copy)]
pub struct Player {
    pub pos: cgmath::Point2<f64>,
    pub vel: cgmath::Vector2<f64>,
    pub press_forward: bool,
    pub press_right: bool,
    pub press_left: bool,
    pub rot: f32,
    pub accent_color_0: [f32; 4],
    pub accent_color_1: [f32; 4],
    pub accent_color_2: [f32; 4],
    pub accent_color_3: [f32; 4],
    pub accent_flame_color: [f32; 4],
    pub flame_frame: f32,
    pub player_img: i32,
    last_frame_upd: std::time::Instant,
}

impl Player {
    const ROT_SPEED: f32 = 4.5;
    
    /// Creates a player at 0, 0 with no vel and rot = 0
    pub fn new() -> Player {
        return Player { 
            pos: cgmath::Point2 { x: 0., y: 0. }, 
            vel: cgmath::Vector2 { x: 0., y: 0. }, 
            rot: 0.,
            press_forward: false,
            press_left: false,
            press_right: false,
            accent_color_0: [1., 0.06, 0.06, 1.],
            accent_color_1: [0.3, 0.85, 1., 1.],
            accent_color_2: [1., 1., 1., 1.],
            accent_color_3: [1., 1., 1., 1.],
            accent_flame_color: [1., 0.5, 0.5, 1.],
            flame_frame: 0.,
            player_img: 0, 
            last_frame_upd: std::time::Instant::now(),
        };
    }
}

pub fn update_players(world: &mut World, delta_t: f64) {
    for player in &mut world.players {
       
        if player.press_right {
            player.rot -= Player::ROT_SPEED * delta_t as f32;
        }
        if player.press_left {
            player.rot += Player::ROT_SPEED * delta_t as f32;
        }

        if player.press_forward {
            let x = player.rot.cos() as f64;
            let y = player.rot.sin() as f64;
            
            player.vel += cgmath::Vector2 { x, y } * delta_t;

            if player.last_frame_upd.elapsed().as_secs_f32() > 0.2 {
                player.last_frame_upd = std::time::Instant::now();
    
                player.flame_frame += 0.25;
                player.flame_frame %= 1.;
            }
        } else {
            player.flame_frame = -1.5;
        }
        
        player.pos += player.vel * delta_t;
    }
}

pub fn get_n_player_img() -> i32 {
    let n_player_img: Vec<_> = match std::fs::read_dir("assets/players") {
        Ok(val) => val,
        Err(err) => panic!("Error while creating world, unable to access player textures: {}", err),
    }.collect();

    return n_player_img.len() as i32;
}