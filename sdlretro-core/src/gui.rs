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

        Self {
            title: title.to_string(),
            items,
            selected: 0,
            scroll_offset: 0,
        }
    }

    /// Get the number of items visible on screen
    pub fn visible_count(&self, fb_height: u32) -> usize {
        let available_height = fb_height as i32 - 40; // Reserve space for header/footer
        let item_height = 12; // Big font height + padding
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

    /// Handle input and return new state
    pub fn handle_input(&mut self, input: &InputReader, fb_height: u32) -> GuiState {
        if self.state == GuiState::Playing {
            // Check for ESC to open menu
            if input.is_key_pressed(1) {
                self.toggle_menu();
            }
            return self.state.clone();
        }

        // Menu is open - handle navigation
        if let Some(ref mut menu) = self.menu {
            // Up arrow
            if input.is_key_pressed(14) {
                menu.select_up();
                self.show_info = false;
            }

            // Down arrow
            if input.is_key_pressed(17) {
                menu.select_down(fb_height);
                self.show_info = false;
            }

            // Enter - select/confirm
            if input.is_key_pressed(28) {
                if menu.is_editable() {
                    self.state = GuiState::Settings;
                } else if let Some(MenuItem::Action { .. }) = menu.items.get(menu.selected) {
                    self.state = GuiState::Playing;
                }
            }

            // Right arrow - next value
            if input.is_key_pressed(15) {
                menu.cycle_next();
            }

            // Left arrow - previous value
            if input.is_key_pressed(12) {
                menu.cycle_prev();
            }

            // Space to cycle value
            if input.is_key_pressed(57) {
                menu.cycle_next();
            }

            // ESC to close menu
            if input.is_key_pressed(1) {
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
            None => return,
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
        video.draw_text_big_overlay(header_x, header_y, self.core_name.as_bytes(), 0xFFFFFF);

        // Calculate visible items
        let visible_count = menu.visible_count(fb_height);
        let item_height = 16;
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
            video.draw_text_overlay((w - hint.len() as i32 * 5) / 2, footer_y + 14, hint.as_bytes(), 0xFFFF00);
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
