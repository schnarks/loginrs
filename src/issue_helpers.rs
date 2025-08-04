use std::fs::{create_dir_all, read_to_string, File};
use libc::{uname, utsname};
use std::{path::Path, ffi::CStr, io::{BufRead, BufReader, Write}};
use chrono::{Local, Datelike, Timelike};
use sysinfo::{System};
use libc::{ttyname, STDIN_FILENO};

pub fn get_host_name() -> String {
    let mut uname_data = utsname {
        sysname: [0; 65],
        nodename: [0; 65],
        release: [0; 65],
        version: [0; 65],
        machine: [0; 65],
        domainname: [0; 65],
    };

    unsafe {
        uname(&mut uname_data);
    }

    let nodename = unsafe { CStr::from_ptr(uname_data.nodename.as_ptr()) }
        .to_string_lossy()
        .into_owned();
    let sysname = unsafe { CStr::from_ptr(uname_data.sysname.as_ptr()) }
        .to_string_lossy()
        .into_owned();
    let release = unsafe { CStr::from_ptr(uname_data.release.as_ptr()) }
        .to_string_lossy()
        .into_owned();
    let machine = unsafe { CStr::from_ptr(uname_data.machine.as_ptr()) }
        .to_string_lossy()
        .into_owned();

    format!("{} {} {} {}", nodename, sysname, release, machine)
}

pub fn get_os_name() -> String {
    let file = File::open("/etc/os-release").ok();
    if let Some(file) = file {
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line.unwrap_or_default();
            if line.starts_with("PRETTY_NAME=") || line.starts_with("NAME=") {
                if let Some(start) = line.find('"') {
                    if let Some(end) = line.rfind('"') {
                        return line[start + 1..end].to_string();
                    }
                }
            }
        }
    }
    "Linux".to_string() // Default if /etc/os-release not found
}

#[allow(dead_code)]
pub fn get_os_version() -> String {
    let file = File::open("/etc/os-release").ok();
    if let Some(file) = file {
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line.unwrap_or_default();
            if line.starts_with("VERSION=") {
                if let Some(start) = line.find('"') {
                    if let Some(end) = line.rfind('"') {
                        return line[start + 1..end].to_string();
                    }
                }
            }
        }
    }
    "unknown".to_string() // Default if not found
}

pub fn get_architecture() -> String {
    let mut uname_data = utsname {
        sysname: [0; 65],
        nodename: [0; 65],
        release: [0; 65],
        version: [0; 65],
        machine: [0; 65],
        domainname: [0; 65],
    };

    unsafe {
        if uname(&mut uname_data) == 0 {
            return CStr::from_ptr(uname_data.machine.as_ptr())
                .to_string_lossy()
                .into_owned();
        }
    }
    "unknown".to_string()
}


pub fn get_tty_name() -> String {
    unsafe {
        let ptr = ttyname(STDIN_FILENO);
        if !ptr.is_null() {
            // Convert the C string to a Rust String
            return CStr::from_ptr(ptr).to_string_lossy().into_owned();
        }
    }
    "tty?".to_string()
}

pub fn get_current_time() -> String {
    let now = Local::now();
    format!("{:02}:{:02}:{:02}", now.hour(), now.minute(), now.second())
}

pub fn get_current_date() -> String {
    let now = Local::now();
    format!(
        "{}, {}-{}-{}",
        now.weekday(),
        now.year(),
        now.month(),
        now.day()
    )
}

pub fn get_logged_in_users() -> usize {
    let mut users_count = 0;
    let utmp_path = "/var/run/utmp";
    if let Ok(file) = File::open(utmp_path) {
        let reader = BufReader::new(file);
        for _line in reader.lines() {
            users_count += 1; // Each line represents a user process
        }
    }
    users_count
}

pub fn get_uptime() -> String {
    if let Some(uptime_seconds) = System::uptime().checked_div(1) {
        let days = uptime_seconds / (60 * 60 * 24);
        let hours = (uptime_seconds % (60 * 60 * 24)) / (60 * 60);
        let minutes = (uptime_seconds % (60 * 60)) / 60;

        format!("{} days, {} hours, {} minutes", days, hours, minutes)
    } else {
        "unknown".to_string()
    }
}

pub fn read_or_generate_issue_file(file_path: &str) -> Vec<String> {
    let path = Path::new(file_path);

    // Try to read the issue file
    match read_to_string(path) {
        Ok(issue_content) => {
            return process_issue_content(issue_content);
        }
        Err(_) => {
            // File doesn't exist or cannot be read, try to create it
            if let Err(e) = generate_default_issue_file(file_path) {
                eprintln!("Warning: Failed to create issue file: {}. Using default content.", e);
            }
        }
    }

    // Try reading again after attempting to create
    match read_to_string(path) {
        Ok(issue_content) => process_issue_content(issue_content),
        Err(_) => {
            eprintln!("Warning: Failed to read issue file even after trying to create it. Using default content.");
            process_issue_content(DEFAULT_FILE_CONTENT.to_string())
        }
    }
}

// Helper function to process content
fn process_issue_content(issue_content: String) -> Vec<String> {
    let content = issue_content
        .replace("%u", &get_logged_in_users().to_string())
        .replace("%U", &get_uptime())
        .replace("%s", &get_os_name())
        .replace("%n", &get_host_name())
        .replace("%m", &get_architecture())
        .replace("%l", &get_tty_name())
        .replace("%d", &get_current_date())
        .replace("%t", &get_current_time());

    content.lines().map(|line| line.to_string()).collect()
}


// Embed the file content as a static string
const DEFAULT_FILE_CONTENT: &str = include_str!("../config/issue");

pub(crate) fn generate_default_issue_file(file_path: &str) -> std::io::Result<()> {
    let path = Path::new(file_path);

    // If file exists, return early
    if path.exists() {
        return Ok(());
    }

    // Ensure the parent directory exists
    if let Some(parent) = path.parent() {
        create_dir_all(parent)?;
    }

    // Write the default content to the file
    let mut file = File::create(path)?;
    file.write_all(DEFAULT_FILE_CONTENT.as_bytes())?;

    Ok(())
}

#[allow(dead_code)]
pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing Rust System Information Functions:");

    // Test get_host_name
    let host_name = get_host_name();
    println!("Host Name: {}", host_name);

    // Test get_os_name
    let os_name = get_os_name();
    println!("OS Name: {}", os_name);

    // Test get_os_version
    let os_version = get_os_version();
    println!("OS Version: {}", os_version);

    // Test get_architecture
    let architecture = get_architecture();
    println!("Architecture: {}", architecture);

    // Test get_tty_name
    let tty_name = get_tty_name();
    println!("TTY Name: {}", tty_name);

    // Test get_current_time
    let current_time = get_current_time();
    println!("Current Time: {}", current_time);

    // Test get_current_date
    let current_date = get_current_date();
    println!("Current Date: {}", current_date);

    // Test get_logged_in_users
    let logged_in_users = get_logged_in_users();
    println!("Logged-in Users: {}", logged_in_users);

    // Test get_uptime
    let uptime = get_uptime();
    println!("Uptime: {}", uptime);

    Ok(())
}
