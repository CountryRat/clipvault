use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut};

const DEFAULT_SHORTCUT: &str = "Ctrl+Shift+V";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub shortcut: String,
    #[serde(default = "default_bg_color")]
    pub bg_color: String,
    #[serde(default = "default_opacity")]
    pub opacity: f32,
    #[serde(default)]
    pub window_x: Option<f64>,
    #[serde(default)]
    pub window_y: Option<f64>,
    #[serde(default)]
    pub autostart: bool,
    #[serde(default)]
    pub asked_autostart: bool,  // true after user has been prompted
}

fn default_bg_color() -> String { "#0f0f0f".into() }
fn default_opacity() -> f32 { 1.0 }

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            shortcut: DEFAULT_SHORTCUT.to_string(),
            bg_color: default_bg_color(),
            opacity: default_opacity(),
            window_x: None,
            window_y: None,
            autostart: false,
            asked_autostart: false,
        }
    }
}

impl AppConfig {
    pub fn load(app_dir: &PathBuf) -> Self {
        let path = app_dir.join("config.json");
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, app_dir: &PathBuf) -> Result<(), String> {
        let path = app_dir.join("config.json");
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(&path, json).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn parse_shortcut(&self) -> Result<Shortcut, String> {
        parse_shortcut_str(&self.shortcut)
    }
}

/// Parse "Ctrl+Shift+V" into Shortcut
pub fn parse_shortcut_str(s: &str) -> Result<Shortcut, String> {
    let parts: Vec<&str> = s.split('+').map(|p| p.trim()).collect();
    if parts.is_empty() || parts.len() > 5 {
        return Err(format!("无效的快捷键: {}", s));
    }

    let mut modifiers = Modifiers::empty();
    let mut code: Option<Code> = None;

    for part in &parts {
        match *part {
            "Ctrl" | "Control" => modifiers |= Modifiers::CONTROL,
            "Shift" => modifiers |= Modifiers::SHIFT,
            "Alt" => modifiers |= Modifiers::ALT,
            "Super" | "Meta" | "Win" => modifiers |= Modifiers::SUPER,
            key => {
                if code.is_some() {
                    return Err(format!("快捷键不能有多个主键: {}", s));
                }
                code = Some(parse_code(key)?);
            }
        }
    }

    let code = code.ok_or_else(|| format!("快捷键缺少主键: {}", s))?;
    Ok(Shortcut::new(Some(modifiers), code))
}

/// Map a key string (e.g. "KeyV", "Space", "F1") to Code
fn parse_code(key: &str) -> Result<Code, String> {
    match key {
        // Letters
        "KeyA" | "A" | "a" => Ok(Code::KeyA),
        "KeyB" | "B" | "b" => Ok(Code::KeyB),
        "KeyC" | "C" | "c" => Ok(Code::KeyC),
        "KeyD" | "D" | "d" => Ok(Code::KeyD),
        "KeyE" | "E" | "e" => Ok(Code::KeyE),
        "KeyF" | "F" | "f" => Ok(Code::KeyF),
        "KeyG" | "G" | "g" => Ok(Code::KeyG),
        "KeyH" | "H" | "h" => Ok(Code::KeyH),
        "KeyI" | "I" | "i" => Ok(Code::KeyI),
        "KeyJ" | "J" | "j" => Ok(Code::KeyJ),
        "KeyK" | "K" | "k" => Ok(Code::KeyK),
        "KeyL" | "L" | "l" => Ok(Code::KeyL),
        "KeyM" | "M" | "m" => Ok(Code::KeyM),
        "KeyN" | "N" | "n" => Ok(Code::KeyN),
        "KeyO" | "O" | "o" => Ok(Code::KeyO),
        "KeyP" | "P" | "p" => Ok(Code::KeyP),
        "KeyQ" | "Q" | "q" => Ok(Code::KeyQ),
        "KeyR" | "R" | "r" => Ok(Code::KeyR),
        "KeyS" | "S" | "s" => Ok(Code::KeyS),
        "KeyT" | "T" | "t" => Ok(Code::KeyT),
        "KeyU" | "U" | "u" => Ok(Code::KeyU),
        "KeyV" | "V" | "v" => Ok(Code::KeyV),
        "KeyW" | "W" | "w" => Ok(Code::KeyW),
        "KeyX" | "X" | "x" => Ok(Code::KeyX),
        "KeyY" | "Y" | "y" => Ok(Code::KeyY),
        "KeyZ" | "Z" | "z" => Ok(Code::KeyZ),

        // Digits
        "Digit0" | "0" => Ok(Code::Digit0),
        "Digit1" | "1" => Ok(Code::Digit1),
        "Digit2" | "2" => Ok(Code::Digit2),
        "Digit3" | "3" => Ok(Code::Digit3),
        "Digit4" | "4" => Ok(Code::Digit4),
        "Digit5" | "5" => Ok(Code::Digit5),
        "Digit6" | "6" => Ok(Code::Digit6),
        "Digit7" | "7" => Ok(Code::Digit7),
        "Digit8" | "8" => Ok(Code::Digit8),
        "Digit9" | "9" => Ok(Code::Digit9),

        // Function keys
        "F1" => Ok(Code::F1),
        "F2" => Ok(Code::F2),
        "F3" => Ok(Code::F3),
        "F4" => Ok(Code::F4),
        "F5" => Ok(Code::F5),
        "F6" => Ok(Code::F6),
        "F7" => Ok(Code::F7),
        "F8" => Ok(Code::F8),
        "F9" => Ok(Code::F9),
        "F10" => Ok(Code::F10),
        "F11" => Ok(Code::F11),
        "F12" => Ok(Code::F12),

        // Special keys
        "Space" | " " => Ok(Code::Space),
        "Escape" | "Esc" => Ok(Code::Escape),
        "Enter" | "Return" => Ok(Code::Enter),
        "Tab" => Ok(Code::Tab),
        "Backspace" => Ok(Code::Backspace),
        "Delete" | "Del" => Ok(Code::Delete),
        "Insert" | "Ins" => Ok(Code::Insert),
        "Home" => Ok(Code::Home),
        "End" => Ok(Code::End),
        "PageUp" | "PgUp" => Ok(Code::PageUp),
        "PageDown" | "PgDn" => Ok(Code::PageDown),
        "ArrowUp" | "Up" => Ok(Code::ArrowUp),
        "ArrowDown" | "Down" => Ok(Code::ArrowDown),
        "ArrowLeft" | "Left" => Ok(Code::ArrowLeft),
        "ArrowRight" | "Right" => Ok(Code::ArrowRight),

        // Punctuation / symbols (common ones)
        "Comma" | "," => Ok(Code::Comma),
        "Period" | "." => Ok(Code::Period),
        "Slash" | "/" => Ok(Code::Slash),
        "Backslash" | "\\" => Ok(Code::Backslash),
        "Semicolon" | ";" => Ok(Code::Semicolon),
        "Quote" | "\"" => Ok(Code::Quote),
        "BracketLeft" | "[" => Ok(Code::BracketLeft),
        "BracketRight" | "]" => Ok(Code::BracketRight),
        "Minus" | "-" => Ok(Code::Minus),
        "Equal" | "=" => Ok(Code::Equal),
        "Backquote" | "`" => Ok(Code::Backquote),

        _ => Err(format!("不支持的按键: {}", key)),
    }
}
