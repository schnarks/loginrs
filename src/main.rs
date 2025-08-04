pub mod auth_user;
pub mod default_selection;
pub mod environment;
pub mod issue_helpers;
pub mod session;
pub mod settings;
pub mod user;
pub mod num_lock;

use crate::auth_user::auth_user;
use crossterm::{cursor::{Hide, Show, MoveTo}, event::{read, Event, KeyCode, KeyEvent}, execute, style::{Print, ResetColor, SetBackgroundColor, SetForegroundColor}, terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen}};
use std::env;
use std::io::{self, Write};
use std::process::Command;
use crossterm::cursor::{DisableBlinking, EnableBlinking};
use crossterm::terminal::size;
use crate::settings::to_color;

fn main() -> Result<(), Box<dyn std::error::Error>> {

    let mut stdout = io::stdout();

    // Enable alternate screen and clear it
    execute!(stdout, EnterAlternateScreen, Hide)?;
    execute!(stdout.lock(), Clear(ClearType::All))?;

    terminal::enable_raw_mode()?;

    let tty_path = environment::get_tty_path();

    // Set the config path to default or to arg1 if provided
    let default_path = String::from("/etc/loginrs/config.toml");
    let args: Vec<String> = env::args().collect();
    let config_path = args.get(1).unwrap_or(&default_path);

    // Parse the settings TOML file into the config struct if the file exists
    // Otherwise try to create the file using default config
    // If fails use default
    let config = settings::parse_settings(config_path);

    // Read the issue file if the file exists
    // Otherwise write default to the file
    // If fails use default
    let issue_lines = issue_helpers::read_or_generate_issue_file(&config.issue_file_settings.issue_file);

    // Read Sessions from sessions TOML file if the file exists
    // Otherwise try to parse sessions from shell file, x11 dir and wayland dir and write them to toml file
    let sessions = session::get_sessions(
        &config.login_behaviour.session_file,
        &config.login_behaviour.shells_file,
        &config.login_behaviour.x11_session_folder,
        &config.login_behaviour.wayland_session_folder)?;


    // Num lock on startup
    if config.login_behaviour.activate_num_lock {
        if let Err(e) = num_lock::set_num_lock_tty(true) {
            eprintln!("Failed to set Num Lock on TTY: {}", e);
        }
    }


    // Parse users from provided pwd file
    let users = user::parse_valid_users(
        &config.login_behaviour.user_file,
        &config.login_behaviour.shells_file,
        *&config.login_behaviour.min_uid,
        *&config.login_behaviour.include_root_user
    )?;

    //
    let (mut selected_user, mut selected_session) = default_selection::get_default_indices(
        &config.login_behaviour.default_selection_file,
        &users,
        &sessions,
    ).unwrap_or((0, 0));

    //yvars
    let mut y_clear_max= 0;
    let (_, y_screen) = size()?;
    let y_cmd = y_screen -2;
    execute!(stdout.lock(), Clear(ClearType::All))?;

    loop {
        execute!(stdout, Hide)?;
        // Clear screen
        clear_lines(config.user_prompt.user_option_row_gap as u16, y_clear_max as u16)?;
        // Display issue file
        let mut y = config.issue_file_settings.issue_row_gap;
        for line in &issue_lines {
            execute!(stdout, MoveTo(config.issue_file_settings.issue_col_gap as u16, y as u16), Print(line))?;
            y += 1;
        }

        // Display user selection prompt
        let mut y = config.user_prompt.user_option_row_gap;
        execute!(stdout, MoveTo(config.user_prompt.user_option_col_gap as u16, y as u16), Print(&config.user_prompt.user_option_prompt))?;
        y += 1;

        // Display user list with arrows
        let arrow_x = (&config.user_prompt.user_option_col_gap +(&config.user_prompt.user_option_prompt.len() / 2)-1) as u16;
        execute!(stdout, MoveTo(arrow_x, y as u16))?;
        execute!(stdout, Print(config.arrows.arrow_up))?;
        y += 1;
        for (i, user) in users.iter().enumerate() {
            execute!(stdout, MoveTo(config.user_prompt.user_option_col_gap as u16, y as u16))?;
            if i == selected_user {
                execute!(stdout, SetForegroundColor(to_color(&*config.colors.highlight_fg_color)), SetBackgroundColor(to_color(&*config.colors.highlight_bg_color)))?;
            }
            execute!(stdout, Print(&user.name))?;
            execute!(stdout, ResetColor)?;
            y += 1;
        }
        execute!(stdout, MoveTo(arrow_x, y as u16))?;
        execute!(stdout, Print(config.arrows.arrow_down))?;


        // Display session selection prompt
        y += config.start_prompt.start_option_row_gap;
        execute!(stdout, MoveTo(config.start_prompt.start_option_col_gap as u16, y as u16), Print(&config.start_prompt.start_option_prompt))?;
        y += 2;

        execute!(stdout, MoveTo(config.start_prompt.start_option_col_gap as u16, y as u16), Print(config.arrows.arrow_left))?;
        execute!(stdout, SetForegroundColor(to_color(&*config.colors.highlight_fg_color)), SetBackgroundColor(to_color(&*config.colors.highlight_bg_color)))?;
        execute!(stdout, Print(&sessions[selected_session].name))?;
        execute!(stdout, ResetColor)?;
        execute!(stdout, Print(config.arrows.arrow_right))?;
        y_clear_max = y;
        // Handle keyboard input
        match read()? {
            Event::Key(KeyEvent { code, .. }) => match code {
                KeyCode::Up => {
                    if selected_user > 0 {
                        selected_user -= 1;
                    }
                }
                KeyCode::Down => {
                    if selected_user < users.len() - 1 {
                        selected_user += 1;
                    }
                }
                KeyCode::Left => {
                    if selected_session == 0 {
                        selected_session = sessions.len() - 1;
                    } else {
                        selected_session -= 1;
                    }
                }
                KeyCode::Right => {
                    selected_session = (selected_session + 1) % sessions.len();
                }
                KeyCode::Enter => {
                    // CMD line
                    let command = &sessions[selected_session].cmd;
                    execute!(stdout, MoveTo(1 as u16, y_cmd -3 ), Print(format!("→ {}", command)))?;

                    // Password prompt
                    y += config.password_prompt.password_row_gap;
                    execute!(stdout, MoveTo(config.password_prompt.password_col_gap as u16, y as u16), Print(&config.password_prompt.password_prompt))?;
                    stdout.flush()?;

                    // Read password (masked)
                    let password = read_password()?;
                    let (auth_bool, pam_handle) = auth_user(&users[selected_user].name, &password, &tty_path);
                    if auth_bool{
                        // Save last selection
                        if config.login_behaviour.write_last_to_default_selection {
                            let _ = default_selection::write_selection(
                                &config.login_behaviour.default_selection_file,
                                &users[selected_user],
                                &sessions[selected_session],
                            );
                        }
                        execute!(stdout, LeaveAlternateScreen, Show)?;
                        terminal::disable_raw_mode()?;
                        execute!(stdout.lock(), Clear(ClearType::All))?;
                        execute!(stdout, MoveTo(0, 0))?;
                        environment::exec_session_as_user(&users[selected_user], &sessions[selected_session], pam_handle);
                        terminal::enable_raw_mode()?;
                        execute!(stdout, MoveTo(0, 0))?;
                        execute!(stdout.lock(), Clear(ClearType::All))?;
                    } else {
                        execute!(stdout, MoveTo(config.password_prompt.password_col_gap as u16, (y + 2) as u16), Print("Authentication failed. Press enter to try again..."))?;
                    }
                }
                KeyCode::F(1) => {
                    execute!(stdout, MoveTo(1, y_screen-4), Print("→ reboot"))?;
                    if issue_helpers::get_logged_in_users() < 1 {
                        let _ = Command::new("reboot").status();
                    } else {
                        execute!(stdout, MoveTo(1, y_screen -3), Print("→ reboot not possible, users are logged in"))?;
                    }
                }
                KeyCode::F(2) => {
                    execute!(stdout, MoveTo(1, y_screen-4), Print("→ shutdown"))?;
                    if issue_helpers::get_logged_in_users() < 1 {
                        let _ = Command::new("shutdown").arg("--poweroff").status();
                    } else {
                        execute!(stdout, MoveTo(1, y_screen -3), Print("→ shutdown not possible, users are logged in"))?;
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
}

// Helper function to read password without echoing input
fn read_password() -> io::Result<String> {
    // Enable the blinking cursor
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    execute!(handle, Show)?;
    execute!(handle, EnableBlinking)?;
    let mut password = String::new();
    loop {
        match read()? {
            Event::Key(KeyEvent { code, .. }) => match code {
                KeyCode::Enter => break,
                KeyCode::Backspace => {
                    password.pop();
                }
                KeyCode::Char(c) => {
                    password.push(c);
                }
                _ => {}
            },
            _ => {}
        }
    }
    execute!(handle, DisableBlinking)?;

    Ok(password)
}

fn clear_lines(y_startline: u16, y_endline: u16) -> io::Result<()> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();

    // Loop through each line from y_startline to y_endline
    for y in y_startline..=y_endline {
        // Move the cursor to the start of the line
        execute!(handle, MoveTo(0, y))?;
        // Clear the line
        execute!(handle, Clear(crossterm::terminal::ClearType::CurrentLine))?;
    }

    // Flush the output to ensure the commands are executed
    handle.flush()?;
    Ok(())
}