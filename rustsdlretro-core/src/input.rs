use evdev::Device;
use libc::c_int;
use std::sync::{Arc, Mutex};

#[cfg(feature = "minifb")]
use minifb::Key;

// Function type for polling keyboard state
type KeyboardPollFn = Box<dyn Fn(Key) -> bool + Send + 'static>;

/// Maximum number of player ports (controllers)
const MAX_PORTS: u32 = 4;
/// Total state slots: MAX_PORTS * 16 buttons
const TOTAL_SLOTS: usize = (MAX_PORTS as usize) * 16;

/// Key mappings for Player 1 (standard layout)
fn player1_keycodes_to_joypad(keycode: u16) -> Option<c_int> {
    match keycode {
        103 | 17 => Some(JOYPAD_UP),
        108 | 31 => Some(JOYPAD_DOWN),
        105 | 30 => Some(JOYPAD_LEFT),
        106 | 32 => Some(JOYPAD_RIGHT),
        40 => Some(JOYPAD_B),
        38 => Some(JOYPAD_A),
        46 => Some(JOYPAD_Y),
        22 => Some(JOYPAD_X),
        31 => Some(JOYPAD_L),
        32 => Some(JOYPAD_R),
        28 => Some(JOYPAD_START),
        42 | 54 => Some(JOYPAD_SELECT),
        _ => None,
    }
}

/// Key mappings for Player 2 (Numpad + WASD layout)
fn player2_keycodes_to_joypad(keycode: u16) -> Option<c_int> {
    match keycode {
        // Numpad direction keys
        89 => Some(JOYPAD_UP),     // KP_Up
        83 => Some(JOYPAD_DOWN),   // KP_Down
        79 => Some(JOYPAD_LEFT),   // KP_Left
        81 => Some(JOYPAD_RIGHT),  // KP_Right
        // WASD for buttons (common P2 layout)
        24 | 35 => Some(JOYPAD_A),     // W or a
        25 | 36 => Some(JOYPAD_B),     // S or s
        17 | 38 => Some(JOYPAD_X),     // A or x (mapped to X since A conflicts)
        23 | 40 => Some(JOYPAD_Y),     // D or z
        45 => Some(JOYPAD_L),          // L - Shift for P2
        56 => Some(JOYPAD_R),          // R - Enter/Return area
        96 => Some(JOYPAD_START),      // KP_Enter
        70 | 71 => Some(JOYPAD_SELECT), // KP_+ / KP_
        _ => None,
    }
}

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

// Map minifb keys to joypad IDs for Player 1 keyboard layout
#[cfg(feature = "minifb")]
fn minifb_key_to_joypad_p1(key: Key) -> Option<c_int> {
    match key {
        Key::Up => Some(JOYPAD_UP),
        Key::Down => Some(JOYPAD_DOWN),
        Key::Left => Some(JOYPAD_LEFT),
        Key::Right => Some(JOYPAD_RIGHT),
        Key::NumPad0 | Key::K => Some(JOYPAD_B),
        Key::L | Key::J => Some(JOYPAD_A),
        Key::I => Some(JOYPAD_Y),
        Key::U => Some(JOYPAD_X),
        Key::Q => Some(JOYPAD_L),
        Key::O => Some(JOYPAD_R),
        Key::Enter | Key::NumPadEnter => Some(JOYPAD_START),
        Key::Space => Some(JOYPAD_SELECT),
        _ => None,
    }
}

// Map minifb keys to joypad IDs for Player 2 keyboard layout (WASD + numpad area)
#[cfg(feature = "minifb")]
fn minifb_key_to_joypad_p2(key: Key) -> Option<c_int> {
    match key {
        // WASD for D-pad
        Key::W => Some(JOYPAD_UP),
        Key::S => Some(JOYPAD_DOWN),
        Key::A => Some(JOYPAD_LEFT),
        Key::D => Some(JOYPAD_RIGHT),
        // Numpad/other for buttons
        Key::NumPad0 | Key::J => Some(JOYPAD_B),  // J or numpad 0
        Key::K | Key::U => Some(JOYPAD_A),        // K or U
        Key::L | Key::I => Some(JOYPAD_Y),        // L or I
        Key::NumPad4 | Key::H => Some(JOYPAD_X),  // H or numpad 4
        Key::N => Some(JOYPAD_L),                 // N for L
        Key::M => Some(JOYPAD_R),                 // M for R
        Key::Semicolon => Some(JOYPAD_START),     // ; for Start
        Key::Comma => Some(JOYPAD_SELECT),        // , for Select
        _ => None,
    }
}

// Global keycodes for system functions (not port-specific)
const MENU_TOGGLE_KEYS: &[u16] = &[1, 59]; // ESC, F1

pub struct InputReader {
    /// Per-port button states (port x 16 buttons)
    state: Arc<Mutex<[i32; TOTAL_SLOTS]>>,
    /// Per-port just-pressed flags
    just_pressed: Arc<Mutex<[bool; TOTAL_SLOTS]>>,
    /// Track previous key state to detect edge transitions (minifb mode)
    #[cfg(feature = "minifb")]
    prev_state: Arc<Mutex<[i32; TOTAL_SLOTS]>>,
    /// F-key tracking for minifb mode
    #[cfg(feature = "minifb")]
    f1_just_pressed: Arc<Mutex<bool>>,
    #[cfg(feature = "minifb")]
    f2_just_pressed: Arc<Mutex<bool>>,
    #[cfg(feature = "minifb")]
    f4_just_pressed: Arc<Mutex<bool>>,
    /// Track whether F1/F2/F4 was held last poll cycle (for edge detection)
    #[cfg(feature = "minifb")]
    f1_held: Arc<Mutex<bool>>,
    #[cfg(feature = "minifb")]
    f2_held: Arc<Mutex<bool>>,
    #[cfg(feature = "minifb")]
    f4_held: Arc<Mutex<bool>>,
    #[cfg(feature = "minifb")]
    poll_fn: Option<KeyboardPollFn>,
}

impl InputReader {
    /// Create an InputReader using evdev (embedded/framebuffer mode)
    pub fn new() -> Result<Self, String> {
        let mut device = match Device::open("/dev/input/event0") {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Failed to open /dev/input/event0: {}", e);
                return Err(format!("Failed to open /dev/input/event0: {}", e));
            }
        };
        eprintln!("Keyboard input device opened");

        let state: [i32; TOTAL_SLOTS] = [0; TOTAL_SLOTS];
        let state = Arc::new(Mutex::new(state));
        let just_pressed: [bool; TOTAL_SLOTS] = [false; TOTAL_SLOTS];
        let just_pressed = Arc::new(Mutex::new(just_pressed));
        let state_clone = state.clone();
        let just_pressed_clone = just_pressed.clone();

        std::thread::spawn(move || {
            loop {
                match device.fetch_events() {
                    Ok(events) => {
                        for event in events {
                            let keycode = event.code();
                            let pressed = event.value() == 1;
                            
                            // Track ESC/F1 edge transitions for menu toggle
                            if keycode == 1 || keycode == 59 { // ESC or F1
                                let pressed = event.value() == 1;
                                // Use a simple flag - just_pressed equivalent for system keys
                                // We'll use the state array's high indices (62, 63) as flags
                                if pressed {
                                    let mut s = state_clone.lock().unwrap();
                                    let idx = if keycode == 1 { TOTAL_SLOTS - 2 } else { TOTAL_SLOTS - 1 };
                                    if s[idx] == 0 {
                                        // Rising edge detected - set a flag
                                        let mut jp = just_pressed_clone.lock().unwrap();
                                        jp[idx] = true;
                                    }
                                    s[idx] = 1;
                                } else {
                                    let mut s = state_clone.lock().unwrap();
                                    let idx = if keycode == 1 { TOTAL_SLOTS - 2 } else { TOTAL_SLOTS - 1 };
                                    s[idx] = 0;
                                }
                                continue;
                            }
                            
                            // Player 1: standard arrow + Z/X/C/V layout
                            if let Some(joypad_id) = player1_keycodes_to_joypad(keycode) {
                                let idx = joypad_id as usize;
                                let mut s = state_clone.lock().unwrap();
                                s[idx] = if pressed { 1 } else { 0 };
                                let mut jp = just_pressed_clone.lock().unwrap();
                                jp[idx] = pressed;
                            }
                            // Player 2: numpad + WASD layout (only for specific keycodes)
                            else if let Some(joypad_id) = player2_keycodes_to_joypad(keycode) {
                                let base = 16; // Port 1 starts at index 16
                                let idx = base + joypad_id as usize;
                                let mut s = state_clone.lock().unwrap();
                                s[idx] = if pressed { 1 } else { 0 };
                                let mut jp = just_pressed_clone.lock().unwrap();
                                jp[idx] = pressed;
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

        Ok(InputReader {
            state,
            just_pressed,
            #[cfg(feature = "minifb")]
            prev_state: Arc::new(Mutex::new([0; TOTAL_SLOTS])),
            #[cfg(feature = "minifb")]
            f1_just_pressed: Arc::new(Mutex::new(false)),
            #[cfg(feature = "minifb")]
            f2_just_pressed: Arc::new(Mutex::new(false)),
            #[cfg(feature = "minifb")]
            f4_just_pressed: Arc::new(Mutex::new(false)),
            #[cfg(feature = "minifb")]
            f1_held: Arc::new(Mutex::new(false)),
            #[cfg(feature = "minifb")]
            f2_held: Arc::new(Mutex::new(false)),
            #[cfg(feature = "minifb")]
            f4_held: Arc::new(Mutex::new(false)),
            #[cfg(feature = "minifb")]
            poll_fn: None,
        })
    }
    
    /// Create a keyboard-based InputReader using minifb (desktop mode)
    #[cfg(feature = "minifb")]
    pub fn new_keyboard() -> Self {
        eprintln!("Using keyboard input (minifb mode)");
        let state: [i32; TOTAL_SLOTS] = [0; TOTAL_SLOTS];
        let state = Arc::new(Mutex::new(state));
        let just_pressed: [bool; TOTAL_SLOTS] = [false; TOTAL_SLOTS];
        let just_pressed = Arc::new(Mutex::new(just_pressed));
        let prev_state: [i32; TOTAL_SLOTS] = [0; TOTAL_SLOTS];
        let prev_state = Arc::new(Mutex::new(prev_state));

        InputReader {
            state,
            just_pressed,
            prev_state,
            f1_just_pressed: Arc::new(Mutex::new(false)),
            f2_just_pressed: Arc::new(Mutex::new(false)),
            f4_just_pressed: Arc::new(Mutex::new(false)),
            f1_held: Arc::new(Mutex::new(false)),
            f2_held: Arc::new(Mutex::new(false)),
            f4_held: Arc::new(Mutex::new(false)),
            poll_fn: None, // Will be set by frontend
        }
    }
    
    /// Set the keyboard poll function (for minifb mode)
    #[cfg(feature = "minifb")]
    pub fn set_poll_fn(&mut self, poll_fn: impl Fn(Key) -> bool + Send + 'static) {
        self.poll_fn = Some(Box::new(poll_fn));
    }
    
    /// Poll the keyboard state using a minifb video reference (for minifb mode)
    #[cfg(feature = "minifb")]
    pub fn poll_with_video(&self, video: &crate::video_minifb::MinifbVideo) {
        let mut s = self.state.lock().unwrap();
        let mut jp = self.just_pressed.lock().unwrap();
        let mut prev = self.prev_state.lock().unwrap();

        // Reset all per-port states first
        for port in 0..MAX_PORTS {
            let base = (port as usize) * 16;
            for i in 0..16 {
                s[base + i] = 0;
            }
        }

        // Player 1 keys: arrow keys + numpad/letters
        let p1_keys = vec![
            Key::Up, Key::Down, Key::Left, Key::Right,
            Key::NumPad0, Key::K, Key::L, Key::J, Key::I, Key::U,
            Key::Q, Key::O, Key::Enter, Key::NumPadEnter, Key::Space,
        ];

        for key in p1_keys {
            if video.is_key_down(key) {
                if let Some(joypad_id) = minifb_key_to_joypad_p1(key) {
                    s[joypad_id as usize] = 1;
                    // Only set just_pressed on rising edge
                    if prev[joypad_id as usize] == 0 {
                        jp[joypad_id as usize] = true;
                    }
                }
            }
        }

        // Player 2 keys: WASD + numpad area
        let p2_keys = vec![
            Key::W, Key::S, Key::A, Key::D,
            Key::NumPad0, Key::J, Key::K, Key::U, Key::L, Key::I,
            Key::N, Key::M, Key::Semicolon, Key::Comma, Key::H,
        ];

        for key in p2_keys {
            if video.is_key_down(key) {
                if let Some(joypad_id) = minifb_key_to_joypad_p2(key) {
                    let base = 16; // Port 1 starts at index 16
                    s[base + joypad_id as usize] = 1;
                    // Only set just_pressed on rising edge
                    if prev[base + joypad_id as usize] == 0 {
                        jp[base + joypad_id as usize] = true;
                    }
                }
            }
        }

        // Save current state for next frame's edge detection
        let total_slots = (MAX_PORTS as usize) * 16;
        for i in 0..total_slots {
            prev[i] = s[i];
        }

        // Track F1/F2/F4 edge transitions (not mapped to joypad buttons)
        let f1_down = video.is_key_down(Key::F1);
        {
            let mut prev_held = self.f1_held.lock().unwrap();
            if f1_down && !*prev_held {
                *self.f1_just_pressed.lock().unwrap() = true;
                eprintln!("[INPUT] F1 just pressed (edge detected)");
            }
            *prev_held = f1_down;
        }
        let f2_down = video.is_key_down(Key::F2);
        {
            let mut prev_held = self.f2_held.lock().unwrap();
            if f2_down && !*prev_held {
                *self.f2_just_pressed.lock().unwrap() = true;
                eprintln!("[INPUT] F2 just pressed (edge detected)");
            }
            *prev_held = f2_down;
        }
        let f4_down = video.is_key_down(Key::F4);
        {
            let mut prev_held = self.f4_held.lock().unwrap();
            if f4_down && !*prev_held {
                *self.f4_just_pressed.lock().unwrap() = true;
                eprintln!("[INPUT] F4 just pressed (edge detected)");
            }
            *prev_held = f4_down;
        }
    }
    
    /// Poll the keyboard state (for minifb mode, uses poll_fn if set)
    #[cfg(feature = "minifb")]
    pub fn poll(&self) {
        if let Some(ref poll_fn) = self.poll_fn {
            let mut s = self.state.lock().unwrap();
            let mut jp = self.just_pressed.lock().unwrap();
            
            // Reset all per-port states first
            for port in 0..MAX_PORTS {
                let base = (port as usize) * 16;
                for i in 0..16 {
                    s[base + i] = 0;
                }
            }
            
            // Player 1 keys
            let p1_keys = vec![
                Key::Up, Key::Down, Key::Left, Key::Right,
                Key::NumPad0, Key::K, Key::L, Key::J, Key::I, Key::U,
                Key::Q, Key::O, Key::Enter, Key::NumPadEnter, Key::Space,
            ];
            
            for key in p1_keys {
                if poll_fn(key) {
                    if let Some(joypad_id) = minifb_key_to_joypad_p1(key) {
                        s[joypad_id as usize] = 1;
                        jp[joypad_id as usize] = true;
                    }
                }
            }
            
            // Player 2 keys
            let p2_keys = vec![
                Key::W, Key::S, Key::A, Key::D,
                Key::NumPad0, Key::J, Key::K, Key::U, Key::L, Key::I,
                Key::N, Key::M, Key::Semicolon, Key::Comma, Key::H,
            ];
            
            for key in p2_keys {
                if poll_fn(key) {
                    if let Some(joypad_id) = minifb_key_to_joypad_p2(key) {
                        let base = 16;
                        s[base + joypad_id as usize] = 1;
                        jp[base + joypad_id as usize] = true;
                    }
                }
            }
        }
    }

    pub fn get_state(&self, port: u32, _device: u32, _index: u32, id: u32) -> i16 {
        // Only support up to MAX_PORTS players
        if port >= MAX_PORTS || id >= 16 {
            return -1;
        }
        let base = (port as usize) * 16;
        let idx = base + id as usize;
        let s = self.state.lock().unwrap();
        s[idx] as i16
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
            if let Some(joypad_id) = player1_keycodes_to_joypad(code) {
                let s = self.state.lock().unwrap();
                // Check port 0 (Player 1)
                if s[joypad_id as usize] == 1 {
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
        if let Some(joypad_id) = player1_keycodes_to_joypad(keycode) {
            let mut jp = self.just_pressed.lock().unwrap();
            let was_pressed = jp[joypad_id as usize];
            if was_pressed {
                jp[joypad_id as usize] = false;
            }
            return was_pressed;
        }
        false
    }

    /// Check if an F-key (F1, F2, F4) was just pressed.
    /// Works in minifb mode via dedicated tracking; always returns false for evdev mode.
    /// Clears the flag after reading so it only fires once per press cycle.
    #[cfg(feature = "minifb")]
    pub fn was_f_key_just_pressed(&self, f_num: u8) -> bool {
        let mut flag = match f_num {
            1 => self.f1_just_pressed.lock().unwrap(),
            2 => self.f2_just_pressed.lock().unwrap(),
            4 => self.f4_just_pressed.lock().unwrap(),
            _ => return false,
        };
        let was_pressed = *flag;
        if was_pressed {
            *flag = false;
        }
        was_pressed
    }
}
