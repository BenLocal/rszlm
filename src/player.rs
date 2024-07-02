use std::collections::HashMap;

use rszlm_sys::*;

use crate::{box_to_mut_void_ptr, const_ptr_to_string, const_str_to_ptr};

pub struct ProxyPlayer(mk_proxy_player);

impl From<mk_proxy_player> for ProxyPlayer {
    fn from(sender: mk_proxy_player) -> Self {
        ProxyPlayer(sender)
    }
}

impl ProxyPlayer {
    pub fn set_options(&self, key: &str, val: &str) {
        unsafe {
            mk_proxy_player_set_option(self.0, const_str_to_ptr!(key), const_str_to_ptr!(val))
        }
    }

    pub fn play(&self, url: &str) {
        unsafe { mk_proxy_player_play(self.0, const_str_to_ptr!(url)) };
    }

    pub fn total_reader_count(&self) -> i32 {
        unsafe { mk_proxy_player_total_reader_count(self.0) }
    }

    pub fn on_close(&self, cb: impl FnOnce(i32, &str, i32) + 'static) {
        let cb = Box::new(cb);
        unsafe {
            mk_proxy_player_set_on_close(
                self.0,
                Some(proxy_player_on_close),
                box_to_mut_void_ptr!(cb),
            )
        }
    }
}

impl Drop for ProxyPlayer {
    fn drop(&mut self) {
        unsafe {
            mk_proxy_player_release(self.0);
        }
    }
}

unsafe impl Sync for ProxyPlayer {}
unsafe impl Send for ProxyPlayer {}

#[derive(Debug, Default)]
pub struct ProxyPlayerBuilder {
    vhost: String,
    app: String,
    stream: String,
    hls_enabled: bool,
    mp4_enabled: bool,
    options: HashMap<String, String>,
}

impl ProxyPlayerBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn vhost(mut self, vhost: &str) -> Self {
        self.vhost = vhost.to_string();
        self
    }

    pub fn app(mut self, app: &str) -> Self {
        self.app = app.to_string();
        self
    }

    pub fn stream(mut self, stream: &str) -> Self {
        self.stream = stream.to_string();
        self
    }

    pub fn hls_enabled(mut self, hls_enabled: bool) -> Self {
        self.hls_enabled = hls_enabled;
        self
    }

    pub fn mp4_enabled(mut self, mp4_enabled: bool) -> Self {
        self.mp4_enabled = mp4_enabled;
        self
    }

    /// Add option
    /// key:
    ///     - net_adapter
    ///     - rtp_type：rtsp播放方式:RTP_TCP = 0, RTP_UDP = 1, RTP_MULTICAST = 2
    ///     - rtsp_user： rtsp播放用户名
    ///     - rtsp_pwd： rtsp播放密码
    ///     - protocol_timeout_ms
    ///     - media_timeout_ms
    ///     - beat_interval_ms
    ///     - rtsp_speed
    pub fn add_option(mut self, key: &str, val: &str) -> Self {
        self.options.insert(key.to_string(), val.to_string());
        self
    }

    pub fn build(self) -> ProxyPlayer {
        let tmp = ProxyPlayer(unsafe {
            mk_proxy_player_create(
                const_str_to_ptr!(self.vhost),
                const_str_to_ptr!(self.app),
                const_str_to_ptr!(self.stream),
                self.hls_enabled as i32,
                self.mp4_enabled as i32,
            )
        });

        if !self.options.is_empty() {
            for (key, val) in self.options {
                tmp.set_options(&key, &val);
            }
        }

        tmp
    }
}

pub type OnCloseCallbackFn = Box<dyn FnMut(i32, &str, i32) + 'static>;
extern "C" fn proxy_player_on_close(
    user_data: *mut ::std::os::raw::c_void,
    err: ::std::os::raw::c_int,
    what: *const ::std::os::raw::c_char,
    sys_err: ::std::os::raw::c_int,
) {
    unsafe {
        let cb: &mut OnCloseCallbackFn = std::mem::transmute(user_data);
        cb(err, const_ptr_to_string!(what).as_str(), sys_err);
    };
}
