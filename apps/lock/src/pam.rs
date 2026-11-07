//! Minimal PAM FFI: the four functions a lock screen needs.
//! Checks the password of the LOGGED-IN user against the PAM service
//! `matrix-lock` (which includes system-auth) — exactly like swaylock/
//! gtklock. No crate; the C API is tiny and stable for decades.

use std::ffi::{c_char, c_int, c_void, CString};

const PAM_SUCCESS: c_int = 0;
const PAM_PROMPT_ECHO_OFF: c_int = 1;
const PAM_PROMPT_ECHO_ON: c_int = 2;

#[repr(C)]
struct PamMessage {
    msg_style: c_int,
    msg: *const c_char,
}
#[repr(C)]
struct PamResponse {
    resp: *mut c_char,
    resp_retcode: c_int,
}
#[repr(C)]
struct PamConv {
    conv: extern "C" fn(c_int, *mut *const PamMessage, *mut *mut PamResponse, *mut c_void) -> c_int,
    appdata_ptr: *mut c_void,
}
type PamHandle = c_void;

extern "C" {
    fn pam_start(
        service: *const c_char,
        user: *const c_char,
        conv: *const PamConv,
        pamh: *mut *mut PamHandle,
    ) -> c_int;
    fn pam_authenticate(pamh: *mut PamHandle, flags: c_int) -> c_int;
    fn pam_acct_mgmt(pamh: *mut PamHandle, flags: c_int) -> c_int;
    fn pam_end(pamh: *mut PamHandle, status: c_int) -> c_int;
    fn strdup(s: *const c_char) -> *mut c_char;
    fn malloc(n: usize) -> *mut c_void;
}

/// The conversation: PAM asks (hidden), we hand over the password.
/// The answers MUST be allocated with malloc/strdup — PAM frees them
/// itself with free().
extern "C" fn conversation(
    num_msg: c_int,
    msg: *mut *const PamMessage,
    resp: *mut *mut PamResponse,
    appdata: *mut c_void,
) -> c_int {
    if num_msg <= 0 {
        return 19; // PAM_CONV_ERR
    }
    unsafe {
        let n = num_msg as usize;
        let answers =
            malloc(n * std::mem::size_of::<PamResponse>()) as *mut PamResponse;
        if answers.is_null() {
            return 5; // PAM_BUF_ERR
        }
        let password = appdata as *const c_char;
        for i in 0..n {
            let m = *msg.add(i);
            let r = answers.add(i);
            (*r).resp_retcode = 0;
            (*r).resp = match (*m).msg_style {
                PAM_PROMPT_ECHO_OFF | PAM_PROMPT_ECHO_ON => strdup(password),
                _ => std::ptr::null_mut(),
            };
        }
        *resp = answers;
    }
    PAM_SUCCESS
}

/// true if user+password are valid (auth AND account status).
pub fn verify(user: &str, password: &str) -> bool {
    let service = CString::new("matrix-lock").unwrap();
    let cuser = match CString::new(user) {
        Ok(c) => c,
        Err(_) => return false,
    };
    let cpass = match CString::new(password) {
        Ok(c) => c,
        Err(_) => return false,
    };
    let conv = PamConv {
        conv: conversation,
        appdata_ptr: cpass.as_ptr() as *mut c_void,
    };
    unsafe {
        let mut pamh: *mut PamHandle = std::ptr::null_mut();
        if pam_start(service.as_ptr(), cuser.as_ptr(), &conv, &mut pamh) != PAM_SUCCESS {
            return false;
        }
        let auth = pam_authenticate(pamh, 0);
        let account = if auth == PAM_SUCCESS {
            pam_acct_mgmt(pamh, 0)
        } else {
            auth
        };
        pam_end(pamh, account);
        // cpass lives until here — the conversation ran synchronously.
        let _ = &cpass;
        auth == PAM_SUCCESS && account == PAM_SUCCESS
    }
}
