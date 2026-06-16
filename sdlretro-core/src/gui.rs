// GUI menu framework for browsing and modifying core options
// Renders overlay on framebuffer using embedded bitmap fonts

use crate::core_options::CoreOptionDefinition;
use crate::input::InputReader;
use crate::video::FbdevVideo;

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
    pub fn visible_count(&self, fb_height: u32) -> usize {
        let available_height = fb_height as i32 - 40; // Reserve space for header/footer
        let item_height = 12; // Small font height + padding
        (available_height / item_height) as usize
    }

    /// Move selection up
    pub fn select_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            if self.selected < self.scroll_offset {
                self.scroll_offset = self.selected;
            }
        }
    }

    /// Move selection down
    pub fn select_down(&mut self, fb_height: u32) {
        let visible = self.visible_count(fb_height);
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
        }
    }

    /// Set the core name (from retro_get_system_info)
    pub fn set_core_name(&mut self, name: &str) {
        self.core_name = name.to_string();
    }

    /// Set the ROM name
    pub fn set_rom_name(&mut self, name: &str) {
        self.rom_name = name.to_string();
    }

    /// Toggle menu open/close
    pub fn toggle_menu(&mut self) {
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
            // Check for ESC to open menu
            let esc_pressed = input.was_key_just_pressed(1);
            if esc_pressed {
                self.toggle_menu();
                self.try_init_menu_from_global();
            }
            return self.state.clone();
        }

        // Menu is open - handle navigation
        if let Some(ref mut menu) = self.menu {
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
                menu.select_down(fb_height);
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

            // ESC to close menu
            if input.was_key_just_pressed(1) {
                self.state = GuiState::Playing;
            }
        }

        self.state.clone()
    }

    /// Render the GUI overlay on the framebuffer
 pub fn render(&self, video: &mut FbdevVideo, fb_width: u32, fb_height: u32) {
        if self.state == GuiState::Playing {
            return;
        }

        let menu = match &self.menu {
            Some(m) => m,
            None => {
                let w = fb_width as i32;
                let h = fb_height as i32;
                video.draw_rect_overlay(10, 40, w - 10, 120, 0x000000);
                let overlay_y = 60;
                video.draw_text_overlay(20, overlay_y, b"Core options not available", 0xFFFF00);
                 video.draw_text_overlay(20, overlay_y + 12, b"Press ESC to close", 0x888888);
                eprintln!("GUI: rendered fallback overlay");
                return;
            }
        };

        let w = fb_width as i32;
        let h = fb_height as i32;

        // Draw semi-transparent background
        let bg_x1 = (w - 400) / 2;
        let bg_y1 = 60;
        let bg_x2 = bg_x1 + 400;
        let bg_y2 = h - 40;

        video.draw_rect_overlay(bg_x1, bg_y1, bg_x2, bg_y2, 0x000000);

        // Draw border
        let border_color = 0x888888;
        for x in bg_x1..bg_x2 {
            video.draw_pixel_overlay(x, bg_y1, border_color);
            video.draw_pixel_overlay(x, bg_y2 - 1, border_color);
        }
        for y in bg_y1..bg_y2 {
            video.draw_pixel_overlay(bg_x1, y, border_color);
            video.draw_pixel_overlay(bg_x2 - 1, y, border_color);
        }

        // Draw header
        let header_y = bg_y1 + 10;
        let header_x = bg_x1 + 10;
        video.draw_text_overlay(header_x, header_y, self.core_name.as_bytes(), 0xFFFFFF);

        // Calculate visible items
        let visible_count = menu.visible_count(fb_height);
        let item_height = 12;
        let start_y = bg_y1 + 30;

        // Draw scroll indicator if needed
        if menu.scroll_offset > 0 {
            video.draw_text_overlay(bg_x1 + bg_x2 - bg_x1 - 20, start_y, b"^", 0x888888);
        }

        // Draw menu items
        for i in menu.scroll_offset..menu.items.len() {
            if (i as i32) - (menu.scroll_offset as i32) >= visible_count as i32 {
                break;
            }

            let item_y = start_y + ((i as i32) - (menu.scroll_offset as i32)) * item_height;
            let item_x = bg_x1 + 15;

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
                        let value_x = bg_x2 - 15 - value_text.len() as i32 * 6;
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

                    // Draw arrow indicators
                    if is_selected && values.len() > 1 {
                        video.draw_text_overlay(item_x - 10, item_y, b"<", 0xFFFF00);
                        video.draw_text_overlay(item_x - 20, item_y, b">", 0xFFFF00);
                    }
                }
                MenuItem::Separator => {
                    let sep_color = 0x444444;
                    for x in bg_x1 + 10..bg_x2 - 10 {
                        video.draw_pixel_overlay(x, item_y, sep_color);
                    }
                }
                MenuItem::Action { label } => {
                    let color = if is_selected { 0xFFFF00 } else { 0x888888 };
                    video.draw_text_overlay(item_x, item_y, label.as_bytes(), color);
                }
            }
        }

        // Draw scroll down indicator
        if menu.scroll_offset + visible_count < menu.items.len() {
            let scroll_y = bg_y2 - 20;
            video.draw_text_overlay(bg_x1 + bg_x2 - bg_x1 - 20, scroll_y, b"v", 0x888888);
        }

        // Draw footer
        let footer_y = bg_y2 + 10;
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
    let mut word_start = 0;
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
