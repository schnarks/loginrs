use std::process::{Command};

/// Enable or disable Num Lock on TTY
pub(crate) fn set_num_lock_tty(enable: bool) -> std::io::Result<()> {
    let arg = if enable { "+num" } else { "-num" };

    // Execute the `setleds` command with the correct argument
    let output = Command::new("setleds")
        .arg(arg)
        .spawn()?
        .wait_with_output()?;

    if !output.status.success() {
        eprintln!(
            "Failed to execute setleds: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(())
}
pub fn main() {
    if let Err(e) = set_num_lock_tty(true) {
        eprintln!("Failed to set Num Lock on TTY: {}", e);
    }
}
