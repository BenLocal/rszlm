pub mod event;
pub mod frame;
pub mod init;
pub mod media;
pub mod obj;
pub mod player;
pub mod pusher;
pub mod recorder;
pub mod server;
#[cfg(feature = "webrtc")]
pub mod webrtc;

/// Converts a C `const char *` to an owned `String`.
///
/// Null-safe: ZLMediaKit may hand a null pointer (e.g. an empty error message),
/// and `CStr::from_ptr(null)` is UB, so a null pointer yields an empty `String`.
/// The pointer expression is evaluated exactly once.
#[macro_export]
macro_rules! const_ptr_to_string {
    ($a:ident) => {{
        let __p = $a;
        if __p.is_null() {
            String::new()
        } else {
            std::ffi::CStr::from_ptr(__p).to_string_lossy().into_owned()
        }
    }};
    ($a:expr) => {{
        let __p = $a;
        if __p.is_null() {
            String::new()
        } else {
            std::ffi::CStr::from_ptr(__p).to_string_lossy().into_owned()
        }
    }};
    ($a:ident, $def:literal) => {{
        let __p = $a;
        if __p.is_null() {
            Ok($def)
        } else {
            Ok(std::ffi::CStr::from_ptr(__p).to_str().map_or($def, |x| x))
        }
    }};
}

#[macro_export]
macro_rules! const_str_to_ptr {
    ($a:ident) => {
        std::ffi::CString::new($a.as_bytes().to_vec()).unwrap()
    };
    ($a:expr) => {
        std::ffi::CString::new($a.as_bytes().to_vec()).unwrap()
    };
    ($a:expr, $default:expr) => {
        std::ffi::CString::new($a.unwrap_or($default).as_bytes().to_vec()).unwrap()
    };
}

#[macro_export]
macro_rules! box_to_mut_void_ptr {
    ($a:ident) => {
        Box::into_raw(Box::new($a)) as *mut _
    };
    ($a:expr) => {
        Box::into_raw(Box::new($a)) as *mut _
    };
}

pub const DEFAULT_VHOST: &str = "__defaultVhost__";
