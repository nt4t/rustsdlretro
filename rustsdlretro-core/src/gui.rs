// GUI menu framework for browsing and modifying core options
// Renders overlay on framebuffer using embedded bitmap fonts

use crate::core_options::CoreOptionDefinition;
use crate::input::InputReader;
use crate::video::VideoBackend;

/// Current state of the GUI
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuiState {
    /// Normal game playback
    Playing,
    /// Menu is open
    MenuOpen,
    /// Editing a specific option value
    Settings,
}

/// Action triggered by save/load state keys
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveLoadAction {
    /// F2 pressed — save current state
    Save,
    /// F4 pressed — load last saved state
    Load,
}

/// Type of menu item
#[derive(Debug, Clone)]
pub enum MenuItem {
    /// Static text label (e.g., header or separator)
    Text { label: String, is_header: bool },
    /// A core option with selectable values
    OptionItem {
        key: String,
        label: String,
        values: Vec<String>,
        current_index: usize,
        info: Option<String>,
    },
    /// Visual separator
    Separator,
    /// Action button (e.g., "Save & Exit")
    Action { label: String },
}

/// Menu configuration
#[derive(Debug)]
pub struct Menu {
    pub title: String,
    pub items: Vec<MenuItem>,
    pub selected: usize,
    pub scroll_offset: usize,
}

impl Menu {
    /// Create a new menu from old-style variables
    pub fn from_old_variables(title: &str, vars: &[crate::core_options::OldVariable]) -> Self {
        let mut items = Vec::new();

        items.push(MenuItem::Text {
            label: title.to_string(),
            is_header: true,
        });

        items.push(MenuItem::Separator);

        for var in vars {
            items.push(MenuItem::OptionItem {
                key: var.key.clone(),
                label: var.title.clone(),
                values: var.values.clone(),
                current_index: var.default_index,
                info: None,
            });
        }

        items.push(MenuItem::Separator);
        items.push(MenuItem::Action {
            label: "Back".to_string(),
        });

        // selected=0 points to header (not drawn in loop), selected=1 is separator, selected=2 is first option
        let initial_selection = if items.len() > 2 { 2 } else { 0 };
        Self {
            title: title.to_string(),
            items,
            selected: initial_selection,
            scroll_offset: 0,
        }
    }

    /// Create a new menu from core options
    pub fn from_core_options(title: &str, options: &[CoreOptionDefinition]) -> Self {
        let mut items = Vec::new();

        // Add header
        items.push(MenuItem::Text {
            label: title.to_string(),
            is_header: true,
        });

        items.push(MenuItem::Separator);

        // Add each option as a menu item
        for opt in options {
            items.push(MenuItem::OptionItem {
                key: opt.key.clone(),
                label: opt.desc.clone(),
                values: opt.values.iter().map(|v| v.value.clone()).collect(),
                current_index: opt.values.iter().position(|v| {
                    opt.default_value.as_deref() == Some(&v.value)
                }).unwrap_or(0),
                info: opt.info.clone(),
            });
        }

        // Add footer action
        items.push(MenuItem::Separator);
        items.push(MenuItem::Action {
            label: "Back".to_string(),
        });

        // selected=0 points to header (not drawn in loop), selected=1 is separator, selected=2 is first option
        let initial_selection = if items.len() > 2 { 2 } else { 0 };
        Self {
            title: title.to_string(),
            items,
            selected: initial_selection,
            scroll_offset: 0,
        }
    }

    /// Get the number of items visible on screen
    pub fn visible_count(&self, menu_height: i32) -> usize {
        let available_height = menu_height - 35; // Reserve space for header/footer
        let item_height = 12; // Small font height + padding
        (available_height / item_height) as usize
    }

    /// Move selection up (stops at first option item, index 2)
    pub fn select_up(&mut self) {
        if self.selected > 2 {
            self.selected -= 1;
            if self.selected < self.scroll_offset {
                self.scroll_offset = self.selected;
            }
        }
    }

    /// Move selection down
    pub fn select_down(&mut self, menu_height: i32) {
        let visible = self.visible_count(menu_height);
        if self.selected < self.items.len() - 1 {
            self.selected += 1;
            if self.selected >= self.scroll_offset + visible {
                self.scroll_offset = self.selected - visible + 1;
            }
        }
    }

    /// Cycle to next value for selected option
    pub fn cycle_next(&mut self) {
        if let Some(MenuItem::OptionItem { current_index, values, .. }) = self.items.get_mut(self.selected) {
            if !values.is_empty() {
                *current_index = (*current_index + 1) % values.len();
            }
        }
    }

    /// Cycle to previous value for selected option
    pub fn cycle_prev(&mut self) {
        if let Some(MenuItem::OptionItem { current_index, values, .. }) = self.items.get_mut(self.selected) {
            if !values.is_empty() {
                *current_index = if *current_index == 0 {
                    values.len() - 1
                } else {
                    *current_index - 1
                };
            }
        }
    }

    /// Check if selected item is an option that can be edited
    pub fn is_editable(&self) -> bool {
        matches!(self.items.get(self.selected), Some(MenuItem::OptionItem { values, .. }) if !values.is_empty())
    }

    /// Get the current selected option's key and value
    pub fn get_selected_value(&self) -> Option<(String, String)> {
        if let Some(MenuItem::OptionItem { key, current_index, values, .. }) = self.items.get(self.selected) {
            if let Some(value) = values.get(*current_index) {
                return Some((key.clone(), value.clone()));
            }
        }
        None
    }

    /// Get the current value of the selected option
    pub fn get_current_value(&self) -> Option<&str> {
        match self.items.get(self.selected) {
            Some(MenuItem::OptionItem { values, current_index, .. }) => {
                values.get(*current_index).map(|v| v.as_str())
            }
            _ => None,
        }
    }

    /// Get the key of the selected option
    pub fn get_selected_key(&self) -> Option<&str> {
        match self.items.get(self.selected) {
            Some(MenuItem::OptionItem { key, .. }) => Some(key.as_str()),
            _ => None,
        }
    }
}

/// GUI overlay for core options
pub struct Gui {
    state: GuiState,
    menu: Option<Menu>,
    core_name: String,
    rom_name: String,
    /// Whether to show the option description/info text
    show_info: bool,
    /// Last navigation direction (for debounce)
    last_nav_dir: i8,
    /// Frame count when last navigation happened
    last_nav_frame: u64,
    /// Whether up/nav key was released since last action
    nav_key_released: bool,
    /// Frame count for value cycling debounce
    last_value_frame: u64,
    /// Current frame counter
    frame_count: u64,
    /// User-selected values (key -> value string)
    selected_values: std::collections::HashMap<String, String>,
    /// Whether framebuffer needs to be cleared
    clear_needed: bool,
    /// Flash message display (e.g., "State Saved")
    flash_message: Option<(String, u64)>, // (text, frame_when_shown)
    /// Track just-pressed F2/F4 for save/load state
    f2_pressed: bool,
    f4_pressed: bool,
}

impl Gui {
   /// Create a new GUI instance
    pub fn new() -> Self {
        Self {
            state: GuiState::Playing,
            menu: None,
            core_name: String::from("RetroCore"),
            rom_name: String::from("No ROM"),
            show_info: false,
            last_nav_dir: -1,
            last_nav_frame: 0,
            nav_key_released: true,
             last_value_frame: 0,
            frame_count: 0,
            selected_values: std::collections::HashMap::new(),
            clear_needed: false,
            flash_message: None,
            f2_pressed: false,
            f4_pressed: false,
        }
    }

    /// Set the core name (from retro_get_system_info)
    pub fn set_core_name(&mut self, name: &str) {
        self.core_name = name.to_string();
    }

    /// Get the core name (for save directory lookup)
    pub fn get_core_name(&self) -> &str {
        &self.core_name
    }

    /// Set the ROM name
    pub fn set_rom_name(&mut self, name: &str) {
        self.rom_name = name.to_string();
    }

    /// Check for save/load state key presses (F2/F4).
    /// Returns SaveLoadAction if triggered, None otherwise.
    /// Edge-triggered: fires only on first frame after key press.
    pub fn check_save_load_keys(&mut self, input: &InputReader) -> Option<SaveLoadAction> {
        #[cfg(feature = "minifb")]
        {
            if input.was_f_key_just_pressed(2) {
                return Some(SaveLoadAction::Save);
            }
            if input.was_f_key_just_pressed(4) {
                return Some(SaveLoadAction::Load);
            }
        }
        #[cfg(not(feature = "minifb"))]
        {
            // F2 = save (evdev KEY_F2 = 60)
            if input.was_key_just_pressed(60) {
                return Some(SaveLoadAction::Save);
            }
            // F4 = load (evdev KEY_F4 = 62)
            if input.was_key_just_pressed(62) {
                return Some(SaveLoadAction::Load);
            }
        }
        None
    }

    /// Show a brief flash message centered at top of screen.
    /// Visible for ~120 frames (2 seconds at 60fps).
    pub fn show_flash_message(&mut self, msg: &str) {
        self.flash_message = Some((msg.to_string(), self.frame_count));
    }

    /// Check if a flash message is still visible.
    fn flash_is_visible(&self) -> bool {
        match &self.flash_message {
            Some((_, shown_at)) => self.frame_count.wrapping_sub(*shown_at) < 120,
            None => false,
        }
    }

    /// Toggle menu open/close
    pub fn toggle_menu(&mut self) {
        self.clear_needed = true;
        self.state = match self.state {
            GuiState::Playing => GuiState::MenuOpen,
            GuiState::MenuOpen | GuiState::Settings => GuiState::Playing,
        };
    }

    /// Set menu state directly
    pub fn set_state(&mut self, state: GuiState) {
        self.state = state;
    }

    /// Get current state
    pub fn state(&self) -> &GuiState {
        &self.state
    }

    /// Initialize menu with core options
    pub fn init_menu(&mut self, core_name: &str, options: &[CoreOptionDefinition]) {
        self.menu = Some(Menu::from_core_options(core_name, options));
        if self.state == GuiState::Playing {
            self.state = GuiState::MenuOpen;
        }
    }

    /// Try to initialize menu from global core options (returns true if initialized)
    pub fn try_init_menu_from_global(&mut self) -> bool {
        if self.menu.is_some() {
            eprintln!("GUI: menu already initialized");
            return true;
        }
        match crate::get_core_options_raw() {
            Some(core_opts) => {
                eprintln!("GUI: got core_opts, v1={:?}, v2={:?}, old_vars={}", core_opts.v1.is_some(), core_opts.v2.is_some(), core_opts.old_vars.len());
                if let Some(ref defs) = core_opts.v1 {
                    eprintln!("GUI: using v1 with {} definitions", defs.definitions.len());
                    let core_name = core_opts.v2.as_ref()
                        .and_then(|v2| v2.categories.first())
                        .map(|c| c.desc.as_str())
                        .unwrap_or("Core");
                    self.init_menu(core_name, &defs.definitions);
                    return true;
                }
                if let Some(ref defs) = core_opts.v2 {
                    eprintln!("GUI: using v2 with {} definitions", defs.definitions.len());
                    let core_name = defs.categories.first()
                        .map(|c| c.desc.as_str())
                        .unwrap_or("Core");
                    self.init_menu(core_name, &defs.definitions);
                    return true;
                }
                if !core_opts.old_vars.is_empty() {
                    eprintln!("GUI: using old vars with {} options", core_opts.old_vars.len());
                    self.init_menu_from_old_vars(core_opts);
                    return true;
                }
            }
            None => {}
        }
        false
    }

    /// Initialize menu from old-style variables
    fn init_menu_from_old_vars(&mut self, core_opts: &crate::core_options::CoreOptions) {
        let vars = core_opts.old_vars.clone();
        let keys = core_opts.old_variable_keys();
        if !vars.is_empty() {
            let menu = Menu::from_old_variables("Core Options", &vars);
            self.menu = Some(menu);
            if self.state == GuiState::Playing {
                self.state = GuiState::MenuOpen;
            }
            eprintln!("GUI: initialized {} menu items from old vars", keys.len());
        }
    }

    /// Handle input and return new state
    pub fn handle_input(&mut self, input: &InputReader, fb_height: u32) -> GuiState {
        self.frame_count += 1;
        
        if self.state == GuiState::Playing {
            // F1 opens menu in minifb (ESC reserved for window close).
            // In fbdev mode, ESC still works as the toggle key.
            let esc_pressed = input.was_key_just_pressed(1);
            #[cfg(feature = "minifb")]
            let f1_pressed = input.was_f_key_just_pressed(1); // F1 scancode
            #[cfg(not(feature = "minifb"))]
            let f1_pressed = false;
            if esc_pressed || f1_pressed {
                self.toggle_menu();
                self.try_init_menu_from_global();
            }
            return self.state.clone();
        }

        // Menu is open - handle navigation
        if let Some(ref mut menu) = self.menu {
            let fb_h = fb_height as i32;
            let menu_height = (fb_h - 60).max(120);
            let up_pressed = input.is_key_pressed(14);
            let down_pressed = input.is_key_pressed(17);
            let left_pressed = input.is_key_pressed(12);
            let right_pressed = input.is_key_pressed(15);

            // Track key release for debounce
            if !up_pressed && !down_pressed {
                self.nav_key_released = true;
            }

            // Up arrow (debounced)
            if up_pressed && self.nav_key_released {
                menu.select_up();
                self.show_info = false;
                self.nav_key_released = false;
            }

            // Down arrow (debounced)
            if down_pressed && self.nav_key_released {
                menu.select_down(menu_height);
                self.show_info = false;
                self.nav_key_released = false;
            }

            // Enter - select/confirm
            if input.is_key_pressed(28) {
                if menu.is_editable() {
                    self.state = GuiState::Settings;
                } else if let Some(MenuItem::Action { .. }) = menu.items.get(menu.selected) {
                    self.state = GuiState::Playing;
                }
            }

            // Right arrow - next value (debounced, 15 frame delay)
            if right_pressed && self.frame_count.wrapping_sub(self.last_value_frame) >= 15 {
                menu.cycle_next();
                if let Some((key, value)) = menu.get_selected_value() {
                    if let Some(ref mut core_opts) = crate::get_core_options_raw_mut() {
                        core_opts.set_v2_value(&key, &value);
                    }
                    unsafe { crate::VARIABLE_UPDATE_PENDING = true; }
                }
                self.last_value_frame = self.frame_count;
            }

            // Left arrow - previous value (debounced, 15 frame delay)
            if left_pressed && self.frame_count.wrapping_sub(self.last_value_frame) >= 15 {
                menu.cycle_prev();
                if let Some((key, value)) = menu.get_selected_value() {
                    if let Some(ref mut core_opts) = crate::get_core_options_raw_mut() {
                        core_opts.set_v2_value(&key, &value);
                    }
                    unsafe { crate::VARIABLE_UPDATE_PENDING = true; }
                }
                self.last_value_frame = self.frame_count;
            }

            // Space to cycle value (debounced, 15 frame delay)
            if input.is_key_pressed(57) && self.frame_count.wrapping_sub(self.last_value_frame) >= 15 {
                menu.cycle_next();
                if let Some((key, value)) = menu.get_selected_value() {
                    if let Some(ref mut core_opts) = crate::get_core_options_raw_mut() {
                        core_opts.set_v2_value(&key, &value);
                    }
                    unsafe { crate::VARIABLE_UPDATE_PENDING = true; }
                }
                self.last_value_frame = self.frame_count;
            }

            // F1 or ESC closes menu (ESC may also close window in minifb)
            #[cfg(feature = "minifb")]
            let f1_pressed = input.was_f_key_just_pressed(1); // F1 scancode
            #[cfg(not(feature = "minifb"))]
            let f1_pressed = false;
            if input.was_key_just_pressed(1) || f1_pressed {
                self.clear_needed = true;
                self.state = GuiState::Playing;
            }
        }

        self.state.clone()
    }

 /// Render the GUI overlay on the video backend
    pub fn render(&mut self, video: &mut dyn VideoBackend, fb_width: u32, fb_height: u32) {
        if self.clear_needed {
            video.clear_overlay(fb_width, fb_height);
            self.clear_needed = false;
        }

        let w = fb_width as i32;
        let h = fb_height as i32;
        // Menu dimensions for 320x240, scaled for larger resolutions
        let menu_width = (w - 20).max(200);
        let menu_height = (h - 60).max(120);
        let bg_x1 = (w - menu_width) / 2;
        let bg_y1 = (h - menu_height) / 2 - 20;

        // Draw flash message if visible (centered at top)
        if self.flash_is_visible() {
            let msg = match &self.flash_message {
                Some((text, _)) => text.as_str(),
                None => "",
            };
            if !msg.is_empty() {
                // Fade effect: alpha based on remaining time
                let elapsed = self.frame_count.wrapping_sub(
                    self.flash_message.as_ref().map(|(_, s)| *s).unwrap_or(0)
                );
                let fade_start: u64 = 100; // Start fading at frame 100 of display
                let alpha: u8 = if elapsed > fade_start {
                    ((fade_start as i32 - (elapsed as i32 - fade_start as i32)) * 255 / fade_start as i32).max(0) as u8
                } else {
                    255
                };
                // Blend yellow text with background for fade effect
                let color = ((alpha as u32) * 0xFFFF00 >> 8) | (((255 - alpha) as u32) * 0x333333 >> 8);
                
                let msg_width = msg.len() as i32 * 6;
                let x = (w - msg_width) / 2;
                let y = bg_y1 + 45; // Below header
                video.draw_rect_overlay(x - 8, y - 10, x + msg_width + 8, y + 14, 0x000000);
                video.draw_text_overlay(x, y, msg.as_bytes(), color | 0xFF000000);
            }
        }

        if self.state == GuiState::Playing {
            return;
        }

        let menu = match &self.menu {
            Some(m) => m,
            None => {
                let bg_x2 = bg_x1 + menu_width;
                let bg_y2 = bg_y1 + menu_height;
                video.draw_rect_overlay(bg_x1, bg_y1, bg_x2, bg_y2, 0x000000);
                let overlay_y = bg_y1 + 20;
                video.draw_text_overlay(bg_x1 + 10, overlay_y, b"Core options not available", 0xFFFF00);
                video.draw_text_overlay(bg_x1 + 10, overlay_y + 12, b"Press ESC to close", 0x888888);
                return;
            }
        };

        let bg_x2 = bg_x1 + menu_width;
        let bg_y2 = bg_y1 + menu_height;

        video.draw_rect_overlay(bg_x1, bg_y1, bg_x2, bg_y2, 0x000000);

        // Draw border (optimized with bulk line writes)
        let border_color = 0x888888;
        video.draw_hline_overlay(bg_x1, bg_x2, bg_y1, border_color);
        video.draw_hline_overlay(bg_x1, bg_x2, bg_y2 - 1, border_color);
        video.draw_vline_overlay(bg_x1, bg_y1, bg_y2, border_color);
        video.draw_vline_overlay(bg_x2 - 1, bg_y1, bg_y2, border_color);

        // Draw header
        let header_y = bg_y1 + 10;
        let header_x = bg_x1 + 10;
        video.draw_text_overlay(header_x, header_y, self.core_name.as_bytes(), 0xFFFFFF);

        // Calculate visible items
        let visible_count = menu.visible_count(menu_height);
        let item_height = 12;
        let start_y = bg_y1 + 25;

        // Draw scroll indicator if needed
        if menu.scroll_offset > 0 {
            video.draw_text_overlay(bg_x2 - 15, start_y, b"^", 0x888888);
        }

        // Draw menu items
        for i in menu.scroll_offset..menu.items.len() {
            if (i as i32) - (menu.scroll_offset as i32) >= visible_count as i32 {
                break;
            }

            let item_y = start_y + ((i as i32) - (menu.scroll_offset as i32)) * item_height;
            let item_x = bg_x1 + 10;

            let is_selected = i == menu.selected;

            match &menu.items[i] {
                MenuItem::Text { label, is_header } => {
                    if *is_header {
                        // Already drawn core name above
                    } else {
                        video.draw_text_overlay(item_x, item_y, label.as_bytes(), 0xCCCCCC);
                    }
                }
                MenuItem::OptionItem { label, values, current_index, info, .. } => {
                    // Draw label
                    let label_color = if is_selected { 0xFFFF00 } else { 0xCCCCCC };
                    video.draw_text_overlay(item_x, item_y, label.as_bytes(), label_color);

                    // Draw current value
                    if let Some(current_value) = values.get(*current_index) {
                        let value_text = format!("[{}]", current_value);
                        let value_color = if is_selected { 0xFFFF00 } else { 0x888888 };
                        let value_x = bg_x2 - 10 - value_text.len() as i32 * 6;
                        video.draw_text_overlay(value_x as i32, item_y, value_text.as_bytes(), value_color);
                    }

                    // Draw info text if selected and show_info is true
                    if is_selected && self.show_info {
                        if let Some(ref info_text) = info {
                            let max_width = bg_x2 - bg_x1 - 30;
                            let wrapped = wrap_text(info_text, max_width / 6);
                            let mut info_y = item_y + 14;
                            for line in wrapped {
                                video.draw_text_overlay(item_x, info_y, line.as_bytes(), 0x666666);
                                info_y += 12;
                                if info_y >= bg_y2 - 20 {
                                    break;
                                }
                            }
                        }
                    }

                    // Draw arrow indicator
                    if is_selected && values.len() > 1 {
                        video.draw_text_overlay(item_x - 6, item_y, b">", 0xFFFF00);
                    }
                }
                MenuItem::Separator => {
                    let sep_color = 0x444444;
                    video.draw_hline_overlay(bg_x1 + 10, bg_x2 - 10, item_y, sep_color);
                }
                MenuItem::Action { label } => {
                    let color = if is_selected { 0xFFFF00 } else { 0x888888 };
                    video.draw_text_overlay(item_x, item_y, label.as_bytes(), color);
                }
            }
        }

        // Draw scroll down indicator
        if menu.scroll_offset + visible_count < menu.items.len() {
            let scroll_y = bg_y2 - 15;
            video.draw_text_overlay(bg_x2 - 15, scroll_y, b"v", 0x888888);
        }

        // Draw footer
        let footer_y = bg_y2 + 8;
        let footer_text = format!("{} | Press ESC to close", self.rom_name);
        video.draw_text_overlay((w - footer_text.len() as i32 * 5) / 2, footer_y, footer_text.as_bytes(), 0x666666);

        // Draw settings hint if in settings mode
        if self.state == GuiState::Settings {
            let hint = "Use < > or SPACE to change value";
            video.draw_text_overlay((w - hint.len() as i32 * 5) / 2, footer_y + 12, hint.as_bytes(), 0xFFFF00);
        }
    }
}

/// Simple text wrapping utility
fn wrap_text(text: &str, max_chars: i32) -> Vec<String> {
    if text.len() as i32 <= max_chars {
        return vec![text.to_string()];
    }

    let mut lines = Vec::new();
    let mut current = String::new();
    let mut word_start: usize;
    let chars: Vec<char> = text.chars().collect();

    let mut i = 0;
    while i < chars.len() {
        if current.len() as i32 >= max_chars {
            lines.push(current.clone());
            current.clear();
            // Find next word boundary
            word_start = i;
            while word_start < chars.len() && chars[word_start].is_whitespace() {
                word_start += 1;
            }
            i = word_start;
            continue;
        }
        current.push(chars[i]);
        i += 1;
    }

    if !current.is_empty() {
        lines.push(current);
    }

    lines
}
