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

#[macro_export]
macro_rules! const_ptr_to_string {
    ($a:ident) => {
        std::ffi::CStr::from_ptr($a).to_string_lossy().into_owned()
    };
    ($a:expr) => {
        std::ffi::CStr::from_ptr($a).to_string_lossy().into_owned()
    };
    ($a:ident, $def:literal) => {
        Ok(std::ffi::CStr::from_ptr(schema)
            .to_str()
            .map_or($def, |x| x))
    };
}

#[macro_export]
macro_rules! const_str_to_ptr {
    ($a:ident) => {
        std::ffi::CString::new($a).unwrap()
    };
    ($a:expr) => {
        std::ffi::CString::new($a).unwrap()
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
