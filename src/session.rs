use std::{fmt, fs, io::{self, BufRead, Write}, path::Path};
use serde::{Serialize, Deserialize};
use toml;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionType {
    X11,
    Wayland,
    Shell,
}

impl fmt::Display for SessionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let lowercase_str = match self {
            SessionType::X11 => "x11",
            SessionType::Wayland => "wayland",
            SessionType::Shell => "tty",
        };
        write!(f, "{}", lowercase_str)
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub name: String,
    pub cmd: String,
    pub session_type: SessionType,
}

#[derive(Serialize, Deserialize)]
struct SessionList {
    sessions: Vec<Session>,
}

// Function to read sessions from TOML or parse & save if missing
pub fn get_sessions(toml_path: &str, shell_file: &str, x11_folder: &str, wayland_folder: &str) -> io::Result<Vec<Session>> {
    // Try to load from TOML
    if let Ok(sessions) = parse_sessions_from_toml(toml_path) {
        println!("Loaded sessions from TOML.");
        return Ok(sessions);
    }

    println!("TOML file not found. Parsing sessions...");

    // If TOML is missing, parse from system files
    let all_sessions = parse_all_sessions(shell_file,x11_folder,wayland_folder)?;
    // Save parsed sessions to TOML for future use
    // Attempt to save parsed sessions to TOML, log an error if it fails
    if let Err(e) = save_sessions_to_toml(toml_path, &all_sessions) {
        eprintln!("Warning: Failed to save sessions to TOML: {}", e);
    }
    Ok(all_sessions)
}


fn parse_sessions_from_toml(toml_path: &str) -> io::Result<Vec<Session>> {
    if !Path::new(toml_path).exists() {
        return Err(io::Error::new(io::ErrorKind::NotFound, "TOML file not found"));
    }

    let toml_content = fs::read_to_string(toml_path)?;
    let parsed: SessionList = toml::from_str(&toml_content)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Failed to parse TOML"))?;

    Ok(parsed.sessions)
}

fn save_sessions_to_toml(toml_path: &str, sessions: &[Session]) -> io::Result<()> {
    let session_list = SessionList {
        sessions: sessions.to_vec(),
    };
    let toml_string = toml::to_string(&session_list)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Failed to serialize to TOML"))?;

    if let Some(parent) = Path::new(toml_path).parent() {
        fs::create_dir_all(parent)?;
    }

    let mut file = fs::File::create(toml_path)?;
    file.write_all(toml_string.as_bytes())?;
    Ok(())
}

fn parse_shell_sessions(path: &str) -> io::Result<Vec<Session>> {
    let mut sessions = Vec::new();
    let file = fs::File::open(path)?;
    let reader = io::BufReader::new(file);

    for line in reader.lines() {
        let line = line?;
        if line.starts_with("/") {
            let session_name = line.split('/').last().unwrap_or("Unknown").to_string();
            sessions.push(Session {
                name: session_name,
                cmd: line,
                session_type: SessionType::Shell,
            });
        }
    }

    Ok(sessions)
}

fn parse_desktop_file(file_path: &Path) -> io::Result<String> {
    let file = fs::File::open(file_path)?;
    let reader = io::BufReader::new(file);

    for line in reader.lines() {
        let line = line?;
        if line.starts_with("Exec=") {
            return Ok(line.trim_start_matches("Exec=").to_string());
        }
    }
    Err(io::Error::new(io::ErrorKind::NotFound, "Exec field not found"))
}

fn parse_sessions_from_directory(directory: &str, session_type: SessionType) -> io::Result<Vec<Session>> {
    let mut sessions = Vec::new();

    if Path::new(directory).exists() {
        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            if entry.path().extension() == Some(std::ffi::OsStr::new("desktop")) {
                let mut session_name = entry.file_name().to_string_lossy().to_string();
                let cmd = parse_desktop_file(&entry.path())?;
                if let Some(stripped_name) = session_name.strip_suffix(".desktop") {
                    session_name = stripped_name.to_string();
                }
                sessions.push(Session {
                    name: session_name,
                    cmd,
                    session_type: session_type.clone(),
                });
            }
        }
    }

    Ok(sessions)
}

fn parse_all_sessions(shell_file: &str, x11_folder: &str, wayland_folder: &str) -> io::Result<Vec<Session>> {
    let mut all_sessions = Vec::new();

    all_sessions.extend(parse_shell_sessions(shell_file)?);
    all_sessions.extend(parse_sessions_from_directory(x11_folder, SessionType::X11)?);
    all_sessions.extend(parse_sessions_from_directory(wayland_folder, SessionType::Wayland)?);
    Ok(all_sessions)
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let toml_path = "/etc/nclogin/sessions.toml";
    let sessions = get_sessions(toml_path, "/etc/shells", "/usr/share/xsessions", "/usr/share/wayland-sessions")?;

    for session in &sessions {
        println!("{:?}", session);
    }

    Ok(())
}
