/// Application state and logic
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Insert,
    Command,
}

#[derive(Debug, Clone)]
pub struct Tab {
    pub id: usize,
    pub title: String,
    pub url: String,
}

impl Tab {
    pub fn new(id: usize, url: String) -> Self {
        Self {
            id,
            title: url.clone(),
            url,
        }
    }
}

pub struct App {
    pub tabs: Vec<Tab>,
    pub active_tab: usize,
    pub input_mode: InputMode,
    pub url_input: String,
    pub command_input: String,
    pub command_palette_open: bool,
    pub status_message: String,
    pub is_ascii_mode: bool,
    next_tab_id: usize,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        let initial_tab = Tab::new(0, "about:blank".to_string());

        Self {
            tabs: vec![initial_tab],
            active_tab: 0,
            input_mode: InputMode::Normal,
            url_input: String::new(),
            command_input: String::new(),
            command_palette_open: false,
            status_message: "Press Ctrl+L to enter URL, Ctrl+T for new tab, q to quit".to_string(),
            is_ascii_mode: false,
            next_tab_id: 1,
        }
    }

    pub fn new_tab(&mut self) {
        let tab = Tab::new(self.next_tab_id, "about:blank".to_string());
        self.next_tab_id += 1;
        self.tabs.push(tab);
        self.active_tab = self.tabs.len() - 1;
        self.status_message = format!("New tab created ({})", self.active_tab + 1);
    }

    pub fn close_current_tab(&mut self) {
        if self.tabs.len() > 1 {
            self.tabs.remove(self.active_tab);
            if self.active_tab >= self.tabs.len() {
                self.active_tab = self.tabs.len() - 1;
            }
            self.status_message = format!("Tab closed, {} tabs remaining", self.tabs.len());
        } else {
            self.status_message = "Cannot close last tab".to_string();
        }
    }

    pub fn next_tab(&mut self) {
        if !self.tabs.is_empty() {
            self.active_tab = (self.active_tab + 1) % self.tabs.len();
        }
    }

    pub fn previous_tab(&mut self) {
        if !self.tabs.is_empty() {
            self.active_tab = if self.active_tab == 0 {
                self.tabs.len() - 1
            } else {
                self.active_tab - 1
            };
        }
    }

    pub fn enter_insert_mode(&mut self) {
        self.input_mode = InputMode::Insert;
        if let Some(tab) = self.tabs.get(self.active_tab) {
            self.url_input = tab.url.clone();
        }
        self.status_message = "Enter URL (press Enter to navigate, Esc to cancel)".to_string();
    }

    pub fn exit_insert_mode(&mut self) {
        self.input_mode = InputMode::Normal;
        self.url_input.clear();
        self.status_message = "Normal mode".to_string();
    }

    pub fn submit_url(&mut self) {
        if !self.url_input.is_empty() {
            let url = if self.url_input.starts_with("http://") || self.url_input.starts_with("https://") {
                self.url_input.clone()
            } else {
                format!("https://{}", self.url_input)
            };

            if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                tab.url = url.clone();
                tab.title = url.clone();
            }

            self.status_message = format!("Navigating to: {}", url);

            // TODO: Actually navigate using libmongoose
            // libmongoose::navigate(&url)?;
        }

        self.exit_insert_mode();
    }

    pub fn toggle_command_palette(&mut self) {
        self.command_palette_open = !self.command_palette_open;
        if self.command_palette_open {
            self.input_mode = InputMode::Command;
            self.status_message = "Command palette (type command, Enter to execute, Esc to close)".to_string();
        } else {
            self.input_mode = InputMode::Normal;
            self.command_input.clear();
            self.status_message = "Normal mode".to_string();
        }
    }

    pub fn toggle_ascii_mode(&mut self) {
        self.is_ascii_mode = !self.is_ascii_mode;
        self.status_message = if self.is_ascii_mode {
            "ASCII art mode enabled (high-fidelity rendering)".to_string()
        } else {
            "Block mode enabled (60fps performance)".to_string()
        };

        // TODO: Send toggle to libmongoose
        // libmongoose::set_ascii_mode(self.is_ascii_mode)?;
    }

    pub fn execute_command(&mut self) {
        let command = self.command_input.trim().to_lowercase();

        match command.as_str() {
            "quit" | "q" | "exit" => {
                std::process::exit(0);
            }
            "clear cache" => {
                self.status_message = "Cache cleared (not yet implemented)".to_string();
            }
            "toggle ascii" => {
                self.toggle_ascii_mode();
            }
            "new tab" => {
                self.new_tab();
            }
            "close tab" => {
                self.close_current_tab();
            }
            _ => {
                self.status_message = format!("Unknown command: {}", command);
            }
        }

        self.toggle_command_palette();
    }

    pub fn on_tick(&mut self) {
        // This is called on every tick (60fps)
        // Future: update rendering state, fetch browser updates, etc.
    }

    pub fn get_current_tab(&self) -> Option<&Tab> {
        self.tabs.get(self.active_tab)
    }
}
