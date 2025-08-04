use crate::session::Session;
use crate::user::User;
use std::{fs::File, io, io::{BufRead,Write}};

fn read_last_usage(file_path: &str) -> io::Result<(String, String)> {
    let file = File::open(file_path)?;
    let reader = io::BufReader::new(file);
    let lines: Vec<String> = reader.lines().collect::<Result<_, _>>()?;

    if lines.len() < 2 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "File is malformed"));
    }

    let username = lines[0].clone();
    let session = lines[1].clone();

    Ok((username, session))
}

pub fn get_default_indices(file_path: &str, users: &Vec<User>, sessions: &Vec<Session>) -> io::Result< (usize, usize)>  {
    let(default_username, default_session) = read_last_usage(file_path)?;

    let mut default_user_index : usize =0;
    let mut default_session_index :usize =0;

    // Find index of the last used user
    if let Some(index) = users.iter().position(|user| user.name == default_username) {
        default_user_index = index;
    }
    // Find index of the last used session
    if let Some(index) = sessions.iter().position(|session| session.name == default_session) {
        default_session_index = index;
    }
    Ok((default_user_index, default_session_index))
}

pub fn write_selection(file_path: &String, selected_user: &User, selected_session: &Session) -> io::Result<()> {
    // Open the file in write mode, truncate the contents if the file already exists
    let mut file = File::create(file_path)?;
    // Write the selected username on the first line, selected session to the second
    writeln!(file, "{}\n{}", selected_user.name,selected_session.name)?;
    Ok(())
}
