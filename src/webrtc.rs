use crate::{
    box_to_mut_void_ptr, const_ptr_to_string, const_str_to_ptr, mk_rtc_server_start,
    mk_webrtc_get_answer_sdp2, obj::on_user_data_free,
};

pub fn rtc_server_start(port: u16) {
    unsafe {
        mk_rtc_server_start(port);
    }
}

pub type WebrtcAnswerSdpCallbackFn = Box<dyn FnMut(String, String) + 'static>;

pub fn get_answer_sdp(cb: WebrtcAnswerSdpCallbackFn, typ: &str, offer: &str, url: &str) {
    unsafe {
        mk_webrtc_get_answer_sdp2(
            box_to_mut_void_ptr!(cb),
            Some(on_user_data_free),
            Some(on_webrtc_get_answer_sdp),
            const_str_to_ptr!(typ),
            const_str_to_ptr!(offer),
            const_str_to_ptr!(url),
        );
    }
}
extern "C" fn on_webrtc_get_answer_sdp(
    user_data: *mut ::std::os::raw::c_void,
    answer: *const ::std::os::raw::c_char,
    err: *const ::std::os::raw::c_char,
) {
    let cb: &mut WebrtcAnswerSdpCallbackFn = unsafe { std::mem::transmute(user_data) };

    let (answer, err) = unsafe { (const_ptr_to_string!(answer), const_ptr_to_string!(err)) };
    cb(answer, err);
}
