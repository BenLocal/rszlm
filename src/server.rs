use rszlm_sys::*;

use crate::{box_to_mut_void_ptr, const_ptr_to_string, const_str_to_ptr};

pub fn http_server_start(port: u16, ssl: bool) {
    unsafe {
        mk_http_server_start(port, ssl as i32);
    }
}

pub fn rtsp_server_start(port: u16, ssl: bool) {
    unsafe {
        mk_rtsp_server_start(port, ssl as i32);
    }
}

pub fn rtmp_server_start(port: u16, ssl: bool) {
    unsafe {
        mk_rtmp_server_start(port, ssl as i32);
    }
}

pub fn rtp_server_start(port: u16) {
    unsafe {
        mk_rtp_server_start(port);
    }
}

pub fn srt_server_start(port: u16) {
    unsafe {
        mk_srt_server_start(port);
    }
}

pub fn stop_all_server() {
    unsafe {
        mk_stop_all_server();
    }
}

pub struct RtpServer(mk_rtp_server);

impl RtpServer {
    pub fn new(port: u16, tcp_mode: i32, stream_id: &str) -> Self {
        Self(unsafe { mk_rtp_server_create(port, tcp_mode, const_str_to_ptr!(stream_id)) })
    }

    pub fn bind_port(&self) -> u16 {
        unsafe { mk_rtp_server_port(self.0) }
    }

    pub fn on_detach(&self, cb: impl FnOnce() + 'static) {
        let cb = Box::new(cb);
        unsafe {
            mk_rtp_server_set_on_detach(
                self.0,
                Some(on_rtp_server_detach),
                box_to_mut_void_ptr!(cb),
            )
        }
    }

    pub fn connect(&self, url: &str, dst_port: u16, cb: impl FnOnce(i32, &str, i32) + 'static) {
        let cb = Box::new(cb);
        unsafe {
            mk_rtp_server_connect(
                self.0,
                const_str_to_ptr!(url),
                dst_port,
                Some(on_rtp_server_connected),
                box_to_mut_void_ptr!(cb),
            )
        }
    }
}

extern "C" fn on_rtp_server_detach(user_data: *mut ::std::os::raw::c_void) {
    unsafe {
        let cb: &mut Box<dyn FnMut() + 'static> = std::mem::transmute(user_data);
        cb();
    };
}

extern "C" fn on_rtp_server_connected(
    user_data: *mut ::std::os::raw::c_void,
    err: ::std::os::raw::c_int,
    what: *const ::std::os::raw::c_char,
    sys_err: ::std::os::raw::c_int,
) {
    unsafe {
        let cb: &mut Box<dyn FnMut(i32, &str, i32) + 'static> = std::mem::transmute(user_data);
        cb(err, const_ptr_to_string!(what).as_str(), sys_err);
    };
}

impl Drop for RtpServer {
    fn drop(&mut self) {
        unsafe { mk_rtp_server_release(self.0) }
    }
}
