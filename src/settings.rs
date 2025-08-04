use serde::{Deserialize, Serialize};
use std::{fs::{File, create_dir_all, read_to_string}, io::Write, path::Path};
use crossterm::style::Color;
use toml;

#[derive(Serialize, Deserialize, Debug)]
pub struct Settings {
    #[serde(default)]
    pub login_behaviour: LoginBehaviour,
    #[serde(default)]
    pub issue_file_settings: IssueFileSettings,
    #[serde(default)]
    pub user_prompt: UserPrompt,
    #[serde(default)]
    pub start_prompt: StartPrompt,
    #[serde(default)]
    pub password_prompt: PasswordPrompt,
    #[serde(default)]
    pub arrows: ArrowStruct,
    #[serde(default)]
    pub colors: ColorsStruct,
    #[serde(default)]
    pub top_bar: TopBar,
    #[serde(default)]
    pub bottom_bar: BottomBar,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            login_behaviour: LoginBehaviour::default(),
            issue_file_settings: IssueFileSettings::default(),
            user_prompt: UserPrompt::default(),
            start_prompt: StartPrompt::default(),
            password_prompt: PasswordPrompt::default(),
            arrows: ArrowStruct::default(),
            colors: ColorsStruct::default(),
            top_bar: TopBar::default(),
            bottom_bar: BottomBar::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LoginBehaviour {
    #[serde(default = "default_min_uid")]
    pub min_uid: usize,
    #[serde(default = "default_user_file")]
    pub user_file: String,
    #[serde(default = "default_shells_file")]
    pub shells_file : String,
    #[serde(default = "default_x11_session_folder")]
    pub x11_session_folder: String,
    #[serde(default = "default_wayland_session_folder")]
    pub wayland_session_folder: String,
    #[serde(default = "default_session_file")]
    pub session_file: String,
    #[serde(default = "default_default_selection_file")]
    pub default_selection_file: String,
    #[serde(default = "default_last_to_default_selection")]
    pub write_last_to_default_selection: bool,
    #[serde(default = "default_include_root_user")]
    pub include_root_user: bool,
    #[serde(default = "default_activate_num_lock")]
    pub activate_num_lock: bool,
}

impl Default for LoginBehaviour {
    fn default() -> Self {
        LoginBehaviour {
            min_uid: default_min_uid(),
            user_file: default_user_file(),
            shells_file: default_shells_file(),
            x11_session_folder: default_x11_session_folder(),
            wayland_session_folder: default_wayland_session_folder(),
            session_file: default_session_file(),
            default_selection_file: default_default_selection_file(),
            write_last_to_default_selection: default_last_to_default_selection(),
            include_root_user: default_include_root_user(),
            activate_num_lock: default_activate_num_lock(),
        }
    }
}

fn default_min_uid() -> usize {
    1000
}
fn default_user_file() -> String {
    "/etc/passwd".to_string()
}
fn default_shells_file() -> String {
    "/etc/shells".to_string()
}
fn default_x11_session_folder() -> String { "/usr/share/xsessions".to_string() }
fn default_wayland_session_folder() -> String { "/usr/share/wayland-sessions".to_string() }
fn default_session_file() -> String { "/etc/loginrs/sessions.toml".to_string() }
fn default_default_selection_file() -> String { "/etc/loginrs/default".to_string() }
fn default_last_to_default_selection() -> bool {
    true
}
fn default_include_root_user() -> bool {
    true
}
fn default_activate_num_lock() -> bool {
    true
}


#[derive(Serialize, Deserialize, Debug)]
pub struct IssueFileSettings {
    #[serde(default = "default_issue_file")]
    pub issue_file: String,
    #[serde(default = "default_issue_row_gap")]
    pub issue_row_gap: usize,
    #[serde(default = "default_issue_col_gap")]
    pub issue_col_gap: usize,
}

impl Default for IssueFileSettings {
    fn default() -> Self {
        IssueFileSettings {
            issue_file: default_issue_file(),
            issue_row_gap: default_issue_row_gap(),
            issue_col_gap: default_issue_col_gap(),
        }
    }
}

fn default_issue_file() -> String {
    "/etc/loginrs/issue".to_string()
}
fn default_issue_row_gap() -> usize {
    2
}
fn default_issue_col_gap() -> usize {
    10
}


#[derive(Serialize, Deserialize, Debug)]
pub struct UserPrompt {
    #[serde(default = "default_user_option_prompt")]
    pub user_option_prompt: String,
    #[serde(default = "default_user_option_row_gap")]
    pub user_option_row_gap: usize,
    #[serde(default = "default_user_option_col_gap")]
    pub user_option_col_gap: usize,
}

impl Default for UserPrompt {
    fn default() -> Self {
        UserPrompt {
            user_option_prompt: default_user_option_prompt(),
            user_option_row_gap: default_user_option_row_gap(),
            user_option_col_gap: default_user_option_col_gap(),
        }
    }
}

fn default_user_option_prompt() -> String { "select user:".to_string() }
fn default_user_option_row_gap() -> usize { 12 }
fn default_user_option_col_gap() -> usize { 10 }


#[derive(Serialize, Deserialize, Debug)]
pub struct StartPrompt {
    #[serde(default = "default_start_option_prompt")]
    pub start_option_prompt: String,
    #[serde(default = "default_start_option_row_gap")]
    pub start_option_row_gap: usize,
    #[serde(default = "default_start_option_col_gap")]
    pub start_option_col_gap: usize,
}

impl Default for StartPrompt {
    fn default() -> Self {
        StartPrompt {
            start_option_prompt: default_start_option_prompt(),
            start_option_row_gap: default_start_option_row_gap(),
            start_option_col_gap: default_start_option_col_gap(),
        }
    }
}

fn default_start_option_prompt() -> String { "select environment:".to_string() }
fn default_start_option_row_gap() -> usize { 2 }
fn default_start_option_col_gap() -> usize { 10 }


#[derive(Serialize, Deserialize, Debug)]
pub struct PasswordPrompt {
    #[serde(default = "default_password_prompt")]
    pub password_prompt: String,
    #[serde(default = "default_password_row_gap")]
    pub password_row_gap: usize,
    #[serde(default = "default_password_col_gap")]
    pub password_col_gap: usize,
    #[serde(default = "default_password_stars")]
    pub password_stars: bool,
}

impl Default for PasswordPrompt {
    fn default() -> Self {
        PasswordPrompt {
            password_prompt: default_password_prompt(),
            password_row_gap: default_password_row_gap(),
            password_col_gap: default_password_col_gap(),
            password_stars: default_password_stars(),
        }
    }
}

fn default_password_prompt() -> String { "type password:".to_string() }
fn default_password_row_gap() -> usize { 2 }
fn default_password_col_gap() -> usize { 10 }
fn default_password_stars() -> bool { true }


#[derive(Serialize, Deserialize, Debug)]
pub struct ArrowStruct {
    #[serde(default = "default_arrow_left")]
    pub arrow_left: char,
    #[serde(default = "default_arrow_right")]
    pub arrow_right: char,
    #[serde(default = "default_arrow_up")]
    pub arrow_up: char,
    #[serde(default = "default_arrow_down")]
    pub arrow_down: char,
}

fn default_arrow_left() -> char { '←' }
fn default_arrow_right() -> char { '→' }
fn default_arrow_up() -> char { '↑' }
fn default_arrow_down() -> char { '↓' }

impl Default for ArrowStruct {
    fn default() -> Self {
        ArrowStruct {
            arrow_left: default_arrow_left(),
            arrow_right: default_arrow_right(),
            arrow_up: default_arrow_up(),
            arrow_down: default_arrow_down(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ColorsStruct {
    #[serde(default = "default_highlight_fg_color")]
    pub highlight_fg_color: String,
    #[serde(default = "default_highlight_bg_color")]
    pub highlight_bg_color: String,
    #[serde(default = "default_normal_fg_color")]
    pub normal_fg_color: String,
    #[serde(default = "default_normal_bg_color")]
    pub normal_bg_color: String,
}

pub fn to_color(color_name: &str) -> Color {
        match color_name.to_lowercase().as_str() {
            "black" => Color::Black,
            "red" => Color::Red,
            "green" => Color::Green,
            "yellow" => Color::Yellow,
            "blue" => Color::Blue,
            "magenta" => Color::Magenta,
            "cyan" => Color::Cyan,
            "white" => Color::White,
            _ =>  Color::White // Default to white if the string doesn't match
        }
    }

impl Default for ColorsStruct {
    fn default() -> Self {
        ColorsStruct {
            highlight_fg_color: default_highlight_fg_color(),
            highlight_bg_color: default_highlight_bg_color(),
            normal_fg_color: default_normal_fg_color(),
            normal_bg_color: default_normal_bg_color(),
        }
    }
}

fn default_highlight_fg_color() -> String { "black".to_string() }
fn default_highlight_bg_color() -> String { "green".to_string() }
fn default_normal_fg_color() -> String { "white".to_string() }
fn default_normal_bg_color() -> String { "black".to_string() }


#[derive(Serialize, Deserialize, Debug)]
pub struct TopBar {
    #[serde(default = "default_top_command")]
    pub top_command: String,
    #[serde(default = "default_top_separator")]
    pub top_separator: String,
    #[serde(default = "default_top_bar_color")]
    pub top_bar_color: String,
    #[serde(default = "default_top_text_bar_color")]
    pub top_text_bar_color: String,
}

impl Default for TopBar {
    fn default() -> Self {
        TopBar {
            top_command: default_top_command(),
            top_separator: default_top_separator(),
            top_bar_color: default_top_bar_color(),
            top_text_bar_color: default_top_text_bar_color(),
        }
    }
}

fn default_top_command() -> String { "".to_string() }
fn default_top_separator() -> String { "".to_string() }
fn default_top_bar_color() -> String { "black".to_string() }
fn default_top_text_bar_color() -> String { "white".to_string() }


#[derive(Serialize, Deserialize, Debug)]
pub struct BottomBar {
    #[serde(default = "default_bottom_command")]
    pub bottom_command: String,
    #[serde(default = "default_bottom_separator")]
    pub bottom_separator: String,
    #[serde(default = "default_bottom_bar_color")]
    pub bottom_bar_color: String,
    #[serde(default = "default_bottom_text_bar_color")]
    pub bottom_text_bar_color: String,
}

impl Default for BottomBar {
    fn default() -> Self {
        BottomBar {
            bottom_command: default_bottom_command(),
            bottom_separator: default_bottom_separator(),
            bottom_bar_color: default_bottom_bar_color(),
            bottom_text_bar_color: default_bottom_text_bar_color(),
        }
    }
}

fn default_bottom_command() -> String {
    "".to_string()
}
fn default_bottom_separator() -> String {
    "".to_string()
}
fn default_bottom_bar_color() -> String {
    "black".to_string()
}
fn default_bottom_text_bar_color() -> String {
    "white".to_string()
}

pub fn parse_settings(config_path: &str) -> Settings {
    let path = Path::new(config_path);

    // Try to read and parse the file first
    if let Ok(content) = read_to_string(path) {
        if let Ok(parsed_settings) = toml::from_str::<Settings>(&content) {
            return parsed_settings;
        } else {
            println!("Warning: Failed to parse config, using defaults.");
        }
    } else {
        println!("Config file not found. Creating with default values...");
    }

    // Create default settings
    let default_settings = Settings::default();
    let toml_string = toml::to_string(&default_settings).expect("Failed to serialize default settings");

    // Ensure the parent directory exists
    if let Some(parent) = path.parent() {
        if let Err(e) = create_dir_all(parent) {
            println!("Error creating config directory: {}", e);
        }
    }

    // Write defaults to file
    if let Err(e) = File::create(path).and_then(|mut file| file.write_all(toml_string.as_bytes())) {
        println!("Error creating config file: {}", e);
    }
    // Return default settings
    default_settings
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read the config file
    let config_path = "config/config.toml";

    // Parse the TOML file into the Config struct
    let settings = parse_settings(config_path);

    // Access and use the parsed config
    println!("{:#?}", settings);
    Ok(())
}