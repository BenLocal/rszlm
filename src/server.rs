use rszlm_sys::*;

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
