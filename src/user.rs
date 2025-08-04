use std::fs;
use std::io::{self, BufRead};

#[derive(Debug)]
pub struct User {
    pub name: String,
    pub uid: usize,
    pub gid: usize,
    pub gecos: String,
    pub homedir: String,
    pub shell: String,
}

pub fn parse_valid_users(passwd_file: &str, shell_file: &str, min_uid: usize, include_root: bool) -> Result<Vec<User>, io::Error> {
    // Read the shells file and store valid shells in a set for quick lookup
    let users_shells = load_valid_shells(shell_file);

    // Open the passwd file
    let passwd_file = fs::File::open(passwd_file)?;
    let reader = io::BufReader::new(passwd_file);

    let mut valid_users = Vec::new();

    // Iterate over each line in the passwd file
    for line in reader.lines() {
        let line = line?;

        // Skip empty lines or comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Split the line by ':' to get the user details
        let fields: Vec<&str> = line.split(':').collect();

        if fields.len() != 7 {
            continue; // Skip malformed lines
        }

        // Parse the user details
        let name = fields[0].to_string();
        let uid: usize = fields[2].parse().unwrap_or(0);
        let gid: usize = fields[3].parse().unwrap_or(0);
        let gecos = fields[4].to_string();
        let homedir = fields[5].to_string();
        let shell = fields[6].to_string();

        // Check if the user meets the criteria
        if (include_root && (uid == 0 || uid >= min_uid)) || (!include_root && uid >= min_uid) {
            // Check if the shell is valid for the user
            if users_shells.contains(&shell) {
                valid_users.push(User {
                    name,
                    uid,
                    gid,
                    gecos,
                    homedir,
                    shell,
                });
            }
        }
    }

    Ok(valid_users)
}

// Helper function to load valid shells from the specified file
fn load_valid_shells(shell_file: &str) -> Vec<String> {
    let mut shells = Vec::new();

    if let Ok(file) = fs::File::open(shell_file) {
        let reader = io::BufReader::new(file);

        for line in reader.lines() {
            if let Ok(line) = line {
                let line = line.trim().to_string();
                if !line.is_empty() && !line.starts_with('#') {
                    shells.push(line);
                }
            }
        }
    }

    // Add defaults if they are not already present
    let defaults = vec!["/bin/bash".to_string(), "/bin/sh".to_string()];
    for default in defaults {
        if !shells.contains(&default) {
            shells.push(default);
        }
    }

    shells
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example of calling the function with /etc/passwd and /etc/shells
    let valid_users = parse_valid_users("/etc/passwd", "/etc/shells", 1000, true);

    // Print the valid users
    if let Ok(users) = valid_users {
        for user in users {
            println!("{:#?}", user);
        }
    } else {
        // Handle the error here if necessary
        eprintln!("Failed to retrieve users");
    }
    Ok(())
}
