pub mod event;
pub mod frame;
pub mod init;
pub mod media;
pub mod obj;
pub mod player;
pub mod pusher;
pub mod server;
#[cfg(feature = "webrtc")]
pub mod webrtc;

#[macro_export]
macro_rules! const_ptr_to_string {
    ($a:ident) => {
        std::ffi::CStr::from_ptr($a).to_string_lossy().to_string()
    };
    ($a:expr) => {
        std::ffi::CStr::from_ptr($a).to_string_lossy().to_string()
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
        std::ffi::CString::new($a).unwrap().into_raw()
    };
    ($a:expr) => {
        std::ffi::CString::new($a).unwrap().into_raw()
    };
}

#[macro_export]
macro_rules! box_to_mut_void_ptr {
    ($a:ident) => {
        Box::into_raw(Box::new($a)) as *mut _
    };
}

pub(crate) fn as_cstr_array<T: Into<Vec<u8>> + Clone>(
    arr: &[T],
) -> *mut *const ::std::os::raw::c_char {
    let c_strings = arr
        .iter()
        .map(|s| std::ffi::CString::new::<T>(s.to_owned()).unwrap())
        .collect::<Vec<_>>();
    let c_char_pointers = c_strings.iter().map(|s| s.as_ptr()).collect::<Vec<_>>();
    let mut c_char_pointers_with_null = c_char_pointers.clone();
    c_char_pointers_with_null.push(std::ptr::null());
    c_char_pointers_with_null.as_mut_ptr()
}
