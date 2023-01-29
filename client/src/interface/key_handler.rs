use std::collections::HashMap;
use std::collections::HashSet;

use winit::event::ElementState;
use winit::event::KeyboardInput;
use winit::event::VirtualKeyCode;
use winit::event::WindowEvent;

use super::KeyInput;

/// A blanket over a hashset to make cleaner input keycode
pub struct KeyInputHashset {
    pressed_keys: HashSet<VirtualKeyCode>,
    pub keymap: HashMap<KeyInput, VirtualKeyCode>,
    key_presses: Vec<VirtualKeyCode>,
}

impl KeyInputHashset {
    /// Loads the keymap from file
    pub fn load() -> KeyInputHashset {
        let keymap_file = match std::fs::read("keymap.json") {
            Ok(val) => val,
            Err(err) => panic!("Error while reading keymaps: {:?}", err),
        };

        let keymap_str = String::from_utf8_lossy(&keymap_file);

        let keymap: HashMap<KeyInput, VirtualKeyCode> = match serde_json::from_str(&keymap_str) {
            Ok(val) => val,
            Err(err) => panic!("Error while deserializing keymaps: {:?}", err),
        };

        return KeyInputHashset { pressed_keys: HashSet::new(), keymap, key_presses: Vec::new() };
    }

    pub fn handle_key_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::KeyboardInput { input, .. } => {
                match input {
                    KeyboardInput { state: ElementState::Pressed, virtual_keycode, .. } => {
                        if let Some(keycode) = virtual_keycode {
                            self.pressed_keys.insert(*keycode);
                            self.key_presses.push(*keycode);
                        } else {
                            eprintln!("Received Pressed Keyboard input without a virtual keycode");
                        }
                    },
                    KeyboardInput { state: ElementState::Released, virtual_keycode, .. } => {
                        if let Some(keycode) = virtual_keycode {
                            self.pressed_keys.remove(&keycode);
                        } else {
                            eprintln!("Received Released Keyboard input without a virtual keycode");
                        }
                    },
                }
            },

            _ => panic!("Interface Error Computer Stupid"),
        }
    }

    pub fn is_pressed(&self, key: &KeyInput) -> bool {
        return self.pressed_keys.contains(self.keymap.get(key).unwrap());
    }

    pub fn drain_events(&mut self) -> Vec<KeyInput> {
        let mut output = Vec::new();

        for event in self.key_presses.drain(..).into_iter() {
            for (key, code) in &self.keymap {
                if event == *code {
                    output.push(*key);
                    break;
                }
            }
        }

        return output;
    }
}

#[cfg(debug_assertions)]
#[allow(unused)]
pub fn save_keymap() {
    let keymap = HashMap::from([
        (KeyInput::Thrust, VirtualKeyCode::Up),
        (KeyInput::TurnRight, VirtualKeyCode::Right),
        (KeyInput::TurnLeft, VirtualKeyCode::Left),
        (KeyInput::Zoom, VirtualKeyCode::P),
        (KeyInput::DeZoom, VirtualKeyCode::M),
        (KeyInput::CenterCam, VirtualKeyCode::O),
        (KeyInput::CamDown, VirtualKeyCode::S),
        (KeyInput::CamLeft, VirtualKeyCode::Q),
        (KeyInput::CamRight, VirtualKeyCode::D),
        (KeyInput::CamUp, VirtualKeyCode::Z),
    ]);

    let serialized = serde_json::to_string(&keymap).unwrap();

    std::fs::write("keymap.json", serialized.as_bytes()).unwrap();

    println!("Saved KeyMap !");
}