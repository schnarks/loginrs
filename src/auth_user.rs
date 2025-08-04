use pam_sys::{
    raw::{pam_start, pam_authenticate, pam_end, pam_open_session},
    types::{PamHandle, PamMessage, PamResponse, PamConversation},
};
use std::{ffi::{CStr, CString}, ptr, os::raw::{c_int, c_void}};
use pam_sys::raw::{pam_close_session, pam_set_item};

// Define PAM constants
const PAM_SUCCESS: c_int = 0;
const PAM_PROMPT_ECHO_OFF: c_int = 1;
pub const PAM_TTY: c_int = 3;
extern "C" fn conversation(
    num_msg: c_int,
    msg: *mut *mut PamMessage,
    resp: *mut *mut PamResponse,
    appdata_ptr: *mut c_void,
) -> c_int {
    if num_msg != 1 || msg.is_null() || appdata_ptr.is_null() {
        return -1;
    }

    unsafe {
        let msg_ptr = *msg;
        if msg_ptr.is_null() {
            return -1;
        }

        let pam_message = &*msg_ptr;

        if pam_message.msg_style != PAM_PROMPT_ECHO_OFF {
            return -1;
        }

        // The response to the prompt (password)
        let password = CStr::from_ptr(appdata_ptr as *const i8);

        // Allocate PamResponse on the heap directly using Box
        let pam_response = Box::new(PamResponse {
            resp: CString::new(password.to_string_lossy().into_owned()).unwrap().into_raw(),
            resp_retcode: 0,
        });

        // Convert the Box into a raw pointer and give it to PAM
        *resp = Box::into_raw(pam_response);
    }
    PAM_SUCCESS
}

pub fn auth_user(username: &str, password: &str, tty_path: &str) -> (bool, *mut PamHandle){
    let service_name = CString::new("login").unwrap();
    let c_username = CString::new(username).unwrap();
    let c_password = CString::new(password).unwrap();

    // Correctly declare pam_handle as *mut *const PamHandle
    let mut pam_handle: *mut PamHandle = ptr::null_mut();
    let pam_conv = PamConversation {
        conv: Some(conversation),
        data_ptr: c_password.as_ptr() as *mut c_void,
    };

    // Call pam_start with pam_handle as *mut *const PamHandle
    let result_start_pam = unsafe {
        pam_start(service_name.as_ptr(), c_username.as_ptr(), &pam_conv, &mut pam_handle as *mut *mut PamHandle as *mut *const PamHandle)
    };
    if result_start_pam != PAM_SUCCESS {
        return (false, pam_handle);
    }

    let auth_result = unsafe { pam_authenticate(pam_handle, 0) };
    if auth_result == PAM_SUCCESS {
        unsafe {
            let tty = CString::new(tty_path).unwrap();  // Specify the terminal (tty1)
            let tty_ptr: *const c_void = tty.as_ptr() as *const c_void;  // Cast to *const c_void
            pam_set_item(pam_handle, PAM_TTY, tty_ptr);
            pam_open_session(pam_handle, 0);
        };
    }
    let end_result = unsafe { pam_end(pam_handle, auth_result) };

    if auth_result == PAM_SUCCESS && end_result == PAM_SUCCESS {
        (true, pam_handle)
    } else {
        (false, pam_handle)
    }
}


//fn is_systemd() -> bool {
//    Command::new("systemctl").arg("--version").output().is_ok()
//}
//
//fn is_openrc() -> bool {
//    Command::new("rc-service").arg("--version").output().is_ok()
//}


#[allow(dead_code)]
fn main() {
    // Get username and password from user input
    let username = "user"; // Replace with actual username input
    let password = "pw";     // Replace with the actual password
    let (auth_bool,handle) = auth_user(username, password, "/dev/tty5");
    if auth_bool {
        println!("Authentication successful!");
    } else {
        println!("Authentication failed.");
    }
    unsafe { pam_close_session(handle, 0); }
}
