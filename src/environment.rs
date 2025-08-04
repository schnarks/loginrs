use std::{fs, ptr, process::{Command, Stdio}, mem::zeroed, ffi::CString, os::unix::{fs::chown, process::CommandExt}, path::{Path, PathBuf}};
use std::env::{remove_var,set_var,var,set_current_dir};
use regex::Regex;
use libc::{self, gettimeofday, timeval, setutxent, utmpx, c_short, pututxline, endutxent, getutxline, c_char, sleep};
use pam_sys::PamHandle;
use pam_sys::raw::pam_close_session;
use crate::session::Session;
use crate::user::User;

fn prepare_environment(user: &User, session: &Session) {

    // Set user-specific environment variables
    set_var("SHELL", &user.shell);
    set_var("LOGNAME", &user.name);
    set_var("USER", &user.name);
    set_var("PWD", &user.homedir);
    set_var("HOME", &user.homedir);

    // Set session-specific environment variables
    set_var("XDG_SESSION_TYPE", session.session_type.to_string());
    //set_var("XDG_CURRENT_DESKTOP", &session.name);
    set_var("XDG_DATA_HOME", format!("{}{}", &user.homedir,"/.local/share"));
    set_var("XDG_CONFIG_HOME", format!("{}{}", &user.homedir,"/.config"));
    set_var("XDG_CACHE_HOME", format!("{}{}", &user.homedir,"/.cache"));
    set_var("DBUS_SESSION_BUS_ADDRESS", format!("unix:path=/run/user/{}/bus", &user.uid));

    // more complicated than needed. somehow i cant get pam_systemd to set my session id? wtf.
    let (session, seat) = get_session_and_seat(&*get_tty_name(), &user.name);

    // onlY set if not set already, systemd pam or openrc may take already care
    if var("XDG_SESSION_CLASS").is_err() {
        set_var("XDG_SESSION_CLASS", "user");
    }
    if var("XDG_RUNTIME_DIR").is_err() {
        set_var("XDG_RUNTIME_DIR", format!("/run/user/{}", user.uid));
    }
    if var("XDG_VTNR").is_err() {
        set_var("XDG_VTNR", get_tty_nr().unwrap_or(0).to_string());
    }
    if var("XDG_SEAT").is_err() {
        set_var("XDG_SEAT", &seat);
    }
    if var("XDG_SESSION_ID").is_err() {
        set_var("XDG_SESSION_ID", &session);
    }
}

fn env_reset() {
    remove_var("SHELL");
    remove_var("LOGNAME");
    remove_var("USER");
    remove_var("PWD");
    remove_var("HOME");
    // Set session-specific environment variables
    remove_var("XDG_SESSION_TYPE");
    remove_var("XDG_CURRENT_DESKTOP");
    remove_var("XDG_DATA_HOME");
    remove_var("XDG_CONFIG_HOME");
    remove_var("XDG_CACHE_HOME");
    remove_var("XDG_SESSION_CLASS");
    remove_var("XDG_RUNTIME_DIR");
    remove_var("XDG_VTNR");
    remove_var("XDG_SEAT");
    remove_var("XDG_SESSION_ID");
}


// Function to get the tty path (/dev/tty?)
pub fn get_tty_path() -> String {
    fs::read_link("/proc/self/fd/0").unwrap_or(PathBuf::from("/dev/tty?")).to_string_lossy().to_string()
}

// Function to get the tty name (tty?)
pub fn get_tty_name() -> String {
    let tty_path = get_tty_path();
    tty_path.trim_start_matches("/dev/").to_string()
}

// Function to get the tty number (?)
pub fn get_tty_nr() -> Option<i32> {
    let tty = get_tty_path();

    // Create a regex pattern to capture the number at the end of the string
    let re = Regex::new(r"(\d+)$").ok()?;

    if let Some(captures) = re.captures(&tty) {
        if let Some(tty_number_str) = captures.get(1) {
            return tty_number_str.as_str().parse().ok();
        }
    }
    None
}

fn get_session_and_seat(target_tty: &str, target_user: &str) -> (String, String) {
    // Run `loginctl --no-legend` and capture the output
    let output = Command::new("loginctl")
        .arg("list-sessions")
        .arg("--no-legend") // Avoid headers, makes parsing easier
        .output()
        .expect("Failed to execute loginctl command");

    let output_str = String::from_utf8_lossy(&output.stdout);

    // Process each line
    for line in output_str.lines() {
        let fields: Vec<&str> = line.split_whitespace().collect();

        // Ensure the line has enough columns
        if fields.len() < 4 {
            continue;
        }

        let session = fields.get(0).unwrap_or(&"1").to_string();
        let user = fields.get(2).unwrap_or(&"-");
        let seat = fields.get(3).unwrap_or(&"seat0").to_string();
        let tty = fields.get(5).unwrap_or(&"-");

        // Match user and tty
        if *user == target_user && *tty == target_tty {
            return (session, seat.to_string());
        }
    }

    // Default session & seat if not found
    ("1".to_string(), "seat0".to_string())
}


const LOGIN_PROCESS: c_short = 6;
const USER_PROCESS: c_short = 7;

fn add_utmpx_entry(username: &str, tty: &str, pid: i32) {
    unsafe {
        // Open utmpx file for updating
        setutxent();

        // Create a new utmpx entry
        let mut entry: utmpx = zeroed();
        entry.ut_type = USER_PROCESS;
        entry.ut_pid = pid;

        // Set the tty name (without `/dev/`)
        let ttyname = tty.trim_start_matches("/dev/");
        let tty_cstr = CString::new(ttyname).unwrap();

        // Ensure that the ttyname fits into the field
        let tty_len = std::cmp::min(tty_cstr.to_bytes().len(), entry.ut_line.len());
        ptr::copy_nonoverlapping(tty_cstr.as_ptr(), entry.ut_line.as_mut_ptr(), tty_len);
        ptr::copy_nonoverlapping(tty_cstr.as_ptr(), entry.ut_id.as_mut_ptr(), tty_len);

        // Set the username (ensure it fits in the buffer)
        let user_cstr = CString::new(username).unwrap();
        let user_len = std::cmp::min(user_cstr.to_bytes().len(), entry.ut_user.len());
        ptr::copy_nonoverlapping(user_cstr.as_ptr(), entry.ut_user.as_mut_ptr(), user_len);

        // Set timestamp
        let mut tv: timeval = zeroed();
        gettimeofday(&mut tv, ptr::null_mut());
        entry.ut_tv.tv_sec = tv.tv_sec as i32;
        entry.ut_tv.tv_usec = tv.tv_usec as i32;

        // Write entry to utmpx
        let result = pututxline(&mut entry);
        if result.is_null() {
            eprintln!("Failed to write to utmpx.");
        }
        // Close utmpx
        endutxent();
    }
}


fn remove_utmpx_entry() {
    unsafe {
        setutxent(); // Open utmpx file for reading & writing
        let tty_name = get_tty_name();
        let mut entry: utmpx = zeroed();
        let c_tty = CString::new(tty_name.clone()).unwrap();
        ptr::copy_nonoverlapping(c_tty.as_ptr(), entry.ut_line.as_mut_ptr(), tty_name.len());

        // Find the entry
        while let Some(current) = getutxline(&mut entry).as_mut() {
            if current.ut_line == entry.ut_line {
                current.ut_type = LOGIN_PROCESS;
                // Set ut_user to "LOGIN" (32 bytes, padded with 0)
                let user_bytes = b"LOGIN";
                let mut user_buf = [0 as c_char; 32]; // Use c_char for this array
                for (i, &byte) in user_bytes.iter().enumerate() {
                    user_buf[i] = byte as c_char;
                }
                current.ut_user = user_buf;

                pututxline(current); // Write update
                break;
            }
        }

        endutxent(); // Close utmpx
    }
}

fn change_tty_ownership(user_uid: u32, tty_path_str: &str) -> Result<(), nix::Error> {
    chown(Path::new(tty_path_str), Some(user_uid), None).unwrap();
    Ok(())
}

pub fn exec_session_as_user(user: &User, session: &Session, pam_handle: *mut PamHandle) {
    // Get tty infos
    let tty_path = get_tty_path();
    let tty_name = get_tty_name();

    // Change tty ownership
    change_tty_ownership(user.uid as u32, &tty_path).expect("Couldn't change tty permissions");

    // Cd to user's home directory
    if let Err(e) = set_current_dir(&user.homedir) {
        eprintln!("Failed to change directory to home directory: {}", e);
    }
    prepare_environment(user, session);

    // Execute the session / shell
    let mut cmd = Command::new(&session.cmd);
    cmd
        .uid(user.uid as u32)
        .gid(user.gid as u32)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    match cmd.spawn() {
        Ok(mut child) => {
            let child_pid = child.id(); // Get the child process PID
            // Add utmp entry
            add_utmpx_entry(&user.name, &tty_name, child_pid as i32);
            let _ = child.wait(); // Wait for the child process to finish
        }
        Err(e) =>
            {
                eprintln!("Failed to execute command: {}", e);
                // sleep so error msg parsing is possible
                unsafe {
                    sleep(1);
                }
            }
    }
    // PAM close session
    unsafe {
        pam_close_session(pam_handle, 0);
    }

    // Reset Env vars
    env_reset();

    // Reset tty permission
    change_tty_ownership(0, &tty_path).expect("Couldn't change tty permissions");

    // Clean up the utmp entry
    remove_utmpx_entry();

}

/*
#define EMPTY         0 /* Record does not contain valid info
                                      (formerly known as UT_UNKNOWN on Linux) */
#define RUN_LVL       1 /* Change in system run-level (see
                                      init(1)) */
#define BOOT_TIME     2 /* Time of system boot (in ut_tv) */
#define NEW_TIME      3 /* Time after system clock change
                                      (in ut_tv) */
#define OLD_TIME      4 /* Time before system clock change
                                      (in ut_tv) */
#define INIT_PROCESS  5 /* Process spawned by init(1) */
#define LOGIN_PROCESS 6 /* Session leader process for user login */
#define USER_PROCESS  7 /* Normal process */
#define DEAD_PROCESS  8 /* Terminated process */
#define ACCOUNTING    9 /* Not implemented */
*/