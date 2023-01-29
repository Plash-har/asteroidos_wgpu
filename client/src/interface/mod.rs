use winit::event::WindowEvent;

use game_logic::World;

use logger::{unexpected, UiLogger};
use crate::rendering::MainRenderer;

use std::time::Instant;

mod key_handler;

pub use key_handler::KeyInputHashset;

const DEBUG_CAM_SPEED: f64 = 1.5;
const DEBUG_CAM_ZOOM: f64 = 1.5;

/// Here is all the logic to interface between the player and the game
pub struct UserInterface {
    player_idx: usize,
    pub mouse_pos: cgmath::Point2<f64>,
    last_upd: Instant,
    pub keys: KeyInputHashset,

    frame_times: Vec<Instant>,

    center_cam: bool,
    free_cam_pos: cgmath::Point2<f64>,
    cam_zoom: f64,
    
    selected_player: usize,
    
    gui_logger: UiLogger,
}

impl UserInterface {
    pub fn new() -> UserInterface {
        // #[cfg(debug_assertions)]
        // key_handler::save_keymap();

        return UserInterface { 
            player_idx: 0, 
            mouse_pos: cgmath::Point2 { x: 0., y: 0. },
            last_upd: Instant::now(),
            keys: KeyInputHashset::load(),

            cam_zoom: 1.,
            center_cam: true,
            free_cam_pos: cgmath::Point2 { x: 0., y: 0. },
            frame_times: Vec::new(),

            selected_player: 0,
            
            gui_logger: UiLogger::new(),
        }
    }

    pub fn update_inputs(&mut self, key_event: &WindowEvent) {
        match key_event {
            WindowEvent::KeyboardInput { .. } => self.keys.handle_key_event(key_event),

            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_pos = cgmath::Point2 { x: position.x, y: position.y };
            },

            _ => {
                unexpected(3, "Was asked to handle a non input related event");
            }
        }
    }

    pub fn update(&mut self, world: &mut World, renderer: &mut MainRenderer, gui_context: egui::Context) {
        let delta_t = self.last_upd.elapsed().as_secs_f64();
        self.last_upd = Instant::now();

        self.handle_player_inputs(renderer, world, delta_t);

        self.clear_time_ups();

        self.draw_gui(world, gui_context);
    }

    fn handle_player_inputs(&mut self, renderer: &mut MainRenderer, world: &mut World, delta_t: f64) {
        if let Some(player) = world.players.get_mut(self.player_idx) {
            for press in self.keys.drain_events() {
                //#[cfg(debug_assertions)]  // All the values and the systems are available but we can't switch mode in release mode
                if press == KeyInput::CenterCam {
                    self.center_cam = !self.center_cam;
                }
            }

            if self.center_cam {
                player.press_forward = self.keys.is_pressed(&KeyInput::Thrust);
                player.press_left = self.keys.is_pressed(&KeyInput::TurnLeft);
                player.press_right = self.keys.is_pressed(&KeyInput::TurnRight);
                renderer.set_cam_pos(player.pos);
            } else {
                renderer.set_cam_pos(self.free_cam_pos);
                if self.keys.is_pressed(&KeyInput::CamUp) {
                    self.free_cam_pos.y += DEBUG_CAM_SPEED * delta_t / self.cam_zoom;
                }
                if self.keys.is_pressed(&KeyInput::CamDown) {
                    self.free_cam_pos.y -= DEBUG_CAM_SPEED * delta_t / self.cam_zoom;
                }

                if self.keys.is_pressed(&KeyInput::CamLeft) {
                    self.free_cam_pos.x -= DEBUG_CAM_SPEED * delta_t / self.cam_zoom;
                }
                if self.keys.is_pressed(&KeyInput::CamRight) {
                    self.free_cam_pos.x += DEBUG_CAM_SPEED * delta_t / self.cam_zoom;
                }

                if self.keys.is_pressed(&KeyInput::Zoom) {
                    self.cam_zoom *= DEBUG_CAM_ZOOM * delta_t - delta_t + 1.;
                    renderer.set_zoom(self.cam_zoom);
                }
                if self.keys.is_pressed(&KeyInput::DeZoom) {
                    self.cam_zoom /= DEBUG_CAM_ZOOM * delta_t - delta_t + 1.;
                    renderer.set_zoom(self.cam_zoom);
                }
            }


        }
    }

    fn clear_time_ups(&mut self) {
        let mut to_remove = Vec::new();

        for (idx, t) in self.frame_times.iter().enumerate() {
            if t.elapsed().as_secs_f64() > 1. {
                to_remove.push(idx);
            }
        }

        for idx in to_remove.iter().rev() {
            self.frame_times.remove(*idx);
        }

        self.frame_times.push(Instant::now());
    }

    fn draw_gui(&mut self, world: &mut World, ctx: egui::Context) {
        self.gui_logger.render(&ctx);

        egui::Window::new("Window Info").resizable(true).show(&ctx, |ui| {
            let fps = self.frame_times.len();

            ui.add(egui::Label::new(format!("FPS: {}", fps)));

            if self.center_cam {
                ui.add(egui::Label::new("Centered Camera"));
            } else {
                ui.add(egui::Label::new("Free Camera"));
            }

            ui.add(egui::Label::new(format!("ZOOM: {:.2}", self.cam_zoom)));

            ui.add(egui::Label::new(format!("{} Asteroids", world.asteroids.len())));
        });

        egui::Window::new("Player Info").resizable(true).show(&ctx, |ui| {
            ui.add(egui::Label::new(format!("{} Players", world.players.len())));

            ui.horizontal(|ui| {
                if ui.add(egui::Button::new("<<")).clicked() && self.selected_player > 0 {
                    self.selected_player -= 1;
                };

                ui.add(egui::Label::new(format!("Player {}", self.selected_player)));

                if ui.add(egui::Button::new(">>")).clicked() {
                    self.selected_player += 1;
                    if self.selected_player >= world.players.len() {
                        self.selected_player = world.players.len() - 1;
                    }
                };
            });

            if let Some(player) = world.players.get_mut(self.selected_player) {
                ui.add(egui::Label::new(format!("Pos x: {:.2} y: {:.2}", player.pos.x, player.pos.y)));
                ui.label(format!("Chunk pos {:?}", game_logic::asteroids::chunk_pos_from_pos(player.pos)));
                ui.add(egui::Label::new(format!("Vel x: {:.2} y: {:.2}", player.vel.x, player.vel.y)));
                ui.add(egui::Label::new(format!("Speed {:.2}", (player.vel.x * player.vel.x + player.vel.y * player.vel.y).sqrt())));

                ui.collapsing("Accent Color 0", |ui| {
                    ui.add(egui::Slider::new(&mut player.accent_color_0[0], 0.0..=1.));
                    ui.add(egui::Slider::new(&mut player.accent_color_0[1], 0.0..=1.));
                    ui.add(egui::Slider::new(&mut player.accent_color_0[2], 0.0..=1.));
                    ui.add(egui::Slider::new(&mut player.accent_color_0[3], 0.0..=1.));
                });

                ui.collapsing("Accent Color 1", |ui| {
                    ui.add(egui::Slider::new(&mut player.accent_color_1[0], 0.0..=1.));
                    ui.add(egui::Slider::new(&mut player.accent_color_1[1], 0.0..=1.));
                    ui.add(egui::Slider::new(&mut player.accent_color_1[2], 0.0..=1.));
                    ui.add(egui::Slider::new(&mut player.accent_color_1[3], 0.0..=1.));
                });

                ui.collapsing("Accent Color 2", |ui| {
                    ui.add(egui::Slider::new(&mut player.accent_color_2[0], 0.0..=1.));
                    ui.add(egui::Slider::new(&mut player.accent_color_2[1], 0.0..=1.));
                    ui.add(egui::Slider::new(&mut player.accent_color_2[2], 0.0..=1.));
                    ui.add(egui::Slider::new(&mut player.accent_color_2[3], 0.0..=1.));
                });

                ui.collapsing("Accent Color 3", |ui| {
                    ui.add(egui::Slider::new(&mut player.accent_color_3[0], 0.0..=1.));
                    ui.add(egui::Slider::new(&mut player.accent_color_3[1], 0.0..=1.));
                    ui.add(egui::Slider::new(&mut player.accent_color_3[2], 0.0..=1.));
                    ui.add(egui::Slider::new(&mut player.accent_color_3[3], 0.0..=1.));
                });

                ui.collapsing("Accent Flame Color", |ui| {
                    ui.add(egui::Slider::new(&mut player.accent_flame_color[0], 0.0..=1.));
                    ui.add(egui::Slider::new(&mut player.accent_flame_color[1], 0.0..=1.));
                    ui.add(egui::Slider::new(&mut player.accent_flame_color[2], 0.0..=1.));
                    ui.add(egui::Slider::new(&mut player.accent_flame_color[3], 0.0..=1.));
                });

                ui.horizontal(|ui| {
                    if ui.add(egui::Button::new("-")).clicked() && player.player_img > 0 {
                        player.player_img -= 1;
                    }

                    ui.add(egui::Label::new(&format!("Img {}", player.player_img)));

                    if ui.add(egui::Button::new("+")).clicked() && player.player_img < world.n_player_img -1 {
                        player.player_img += 1;
                    }
                });
            }
        });
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, serde::Deserialize, serde::Serialize)]
pub enum KeyInput {
    Thrust, // Player thrust
    TurnRight,
    TurnLeft,
    Zoom,
    DeZoom,
    CenterCam,
    CamUp,
    CamLeft,
    CamRight,
    CamDown,
}