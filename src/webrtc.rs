use rszlm_sys::*;

use crate::{box_to_mut_void_ptr, const_ptr_to_string, const_str_to_ptr, obj::on_user_data_free};

pub fn rtc_server_start(port: u16) {
    unsafe {
        mk_rtc_server_start(port);
    }
}

/// get answer sdp
/// - return (answer, err)
pub type WebrtcAnswerSdpCallbackFn = Box<dyn FnMut(Option<String>, Option<String>) + 'static>;

pub fn get_answer_sdp(cb: WebrtcAnswerSdpCallbackFn, typ: &str, offer: &str, url: &str) {
    let typ = const_str_to_ptr!(typ);
    let offer = const_str_to_ptr!(offer);
    let url = const_str_to_ptr!(url);
    unsafe {
        mk_webrtc_get_answer_sdp2(
            box_to_mut_void_ptr!(cb),
            Some(on_user_data_free),
            Some(on_webrtc_get_answer_sdp),
            typ.as_ptr(),
            offer.as_ptr(),
            url.as_ptr(),
        );
    }
}
extern "C" fn on_webrtc_get_answer_sdp(
    user_data: *mut ::std::os::raw::c_void,
    answer: *const ::std::os::raw::c_char,
    err: *const ::std::os::raw::c_char,
) {
    let cb: &mut WebrtcAnswerSdpCallbackFn = unsafe { std::mem::transmute(user_data) };
    let answer = if answer.is_null() {
        None
    } else {
        unsafe { Some(const_ptr_to_string!(answer)) }
    };

    let err = if err.is_null() {
        None
    } else {
        unsafe { Some(const_ptr_to_string!(err)) }
    };

    cb(answer, err);
}
