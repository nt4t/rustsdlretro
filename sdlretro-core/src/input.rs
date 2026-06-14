use evdev::Device;
use libc::c_int;
use std::sync::{Arc, Mutex};

// SNES button mappings (libretro RETRO_DEVICE_ID_JOYPAD values)
const JOYPAD_B: c_int = 0;
const JOYPAD_A: c_int = 8;
const JOYPAD_Y: c_int = 1;
const JOYPAD_X: c_int = 9;
const JOYPAD_L: c_int = 10;
const JOYPAD_R: c_int = 11;
const JOYPAD_START: c_int = 3;
const JOYPAD_SELECT: c_int = 2;
const JOYPAD_UP: c_int = 4;
const JOYPAD_DOWN: c_int = 5;
const JOYPAD_LEFT: c_int = 6;
const JOYPAD_RIGHT: c_int = 7;

fn keycode_to_joypad(keycode: u16) -> Option<c_int> {
    match keycode {
        103 | 25 | 17 => Some(JOYPAD_UP),
        108 | 16 | 31 => Some(JOYPAD_DOWN),
        105 | 30 | 14 => Some(JOYPAD_LEFT),
        106 | 32 | 15 => Some(JOYPAD_RIGHT),
        40 => Some(JOYPAD_B),
        38 => Some(JOYPAD_A),
        46 => Some(JOYPAD_Y),
        22 => Some(JOYPAD_X),
        31 => Some(JOYPAD_L),
        32 => Some(JOYPAD_R),
        28 => Some(JOYPAD_START),
        42 | 54 => Some(JOYPAD_SELECT),
        59 => Some(12), // F1 - menu toggle
        _ => None,
    }
}

pub struct InputReader {
    state: Arc<Mutex<[i32; 16]>>,
    just_pressed: Arc<Mutex<[bool; 16]>>,
}

impl InputReader {
    pub fn new() -> Result<Self, String> {
        let mut device = match Device::open("/dev/input/event0") {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Failed to open /dev/input/event0: {}", e);
                return Err(format!("Failed to open /dev/input/event0: {}", e));
            }
        };
        eprintln!("Keyboard input device opened");

        let state: [i32; 16] = [0; 16];
        let state = Arc::new(Mutex::new(state));
        let just_pressed: [bool; 16] = [false; 16];
        let just_pressed = Arc::new(Mutex::new(just_pressed));
        let state_clone = state.clone();
        let just_pressed_clone = just_pressed.clone();

        std::thread::spawn(move || {
            loop {
                match device.fetch_events() {
                    Ok(events) => {
                        for event in events {
                            if event.value() == 1 {
                                if let Some(joypad_id) = keycode_to_joypad(event.code()) {
                                    let mut s = state_clone.lock().unwrap();
                                    s[joypad_id as usize] = 1;
                                    let mut jp = just_pressed_clone.lock().unwrap();
                                    jp[joypad_id as usize] = true;
                                    eprintln!("Input: pressed code={} joypad={}", event.code(), joypad_id);
                                }
                            } else if event.value() == 0 {
                                if let Some(joypad_id) = keycode_to_joypad(event.code()) {
                                    let mut s = state_clone.lock().unwrap();
                                    s[joypad_id as usize] = 0;
                                    let mut jp = just_pressed_clone.lock().unwrap();
                                    jp[joypad_id as usize] = false;
                                    eprintln!("Input: released code={} joypad={}", event.code(), joypad_id);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error fetching events: {}", e);
                        break;
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        });

        Ok(InputReader { state, just_pressed })
    }

    pub fn get_state(&self, _port: u32, _device: u32, _index: u32, id: u32) -> i16 {
        if id < 16 {
            let s = self.state.lock().unwrap();
            s[id as usize] as i16
        } else {
            -1
        }
    }

    /// Check if a specific Linux input keycode is currently pressed
    pub fn is_key_pressed(&self, keycode: u16) -> bool {
        match keycode {
            1 => true,  // ESC - not mapped to joypad
            12 => self.check_keycodes(&[105, 30, 14]),      // Left arrow
            14 => self.check_keycodes(&[103, 25, 17]),      // Up arrow
            15 => self.check_keycodes(&[106, 32]),          // Right arrow
            17 => self.check_keycodes(&[108, 16, 31]),      // Down arrow
            28 => self.check_keycodes(&[28]),               // Enter
            42 => self.check_keycodes(&[42]),               // Left Shift
            54 => self.check_keycodes(&[54]),               // Right Shift
            57 => self.check_keycodes(&[57]),               // Space
            59 => self.check_keycodes(&[59]),               // F1
            _ => false,
        }
    }

    fn check_keycodes(&self, codes: &[u16]) -> bool {
        for &code in codes {
            if let Some(joypad_id) = keycode_to_joypad(code) {
                let s = self.state.lock().unwrap();
                if (joypad_id as usize) < s.len() && s[joypad_id as usize] == 1 {
                    return true;
                }
            }
        }
        false
    }

    /// Check if Shift key is pressed (left or right)
    pub fn is_shift_pressed(&self) -> bool {
        self.check_keycodes(&[42, 54])
    }

    /// Check if Tab key is pressed
    pub fn is_tab_pressed(&self) -> bool {
        self.check_keycodes(&[15])
    }

    /// Check if a key was just pressed (edge detection, clears after check)
    pub fn was_key_just_pressed(&self, keycode: u16) -> bool {
        if let Some(joypad_id) = keycode_to_joypad(keycode) {
            let mut jp = self.just_pressed.lock().unwrap();
            let was_pressed = jp[joypad_id as usize];
            eprintln!("was_key_just_pressed: keycode={} joypad={} was_pressed={}", keycode, joypad_id, was_pressed);
            if was_pressed {
                jp[joypad_id as usize] = false;
            }
            return was_pressed;
        }
        eprintln!("was_key_just_pressed: keycode={} not mapped", keycode);
        false
    }
}
