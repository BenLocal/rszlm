use rszlm_sys::*;

use crate::{box_to_mut_void_ptr, const_ptr_to_string, const_str_to_ptr, obj::MediaSource};

pub struct Pusher(mk_pusher);

impl From<mk_pusher> for Pusher {
    fn from(sender: mk_pusher) -> Self {
        Pusher(sender)
    }
}

impl From<MediaSource> for Pusher {
    fn from(sender: MediaSource) -> Self {
        unsafe { Pusher(mk_pusher_create_src(sender.inner())) }
    }
}

impl Pusher {
    pub fn set_options(&self, key: &str, val: &str) {
        unsafe { mk_pusher_set_option(self.0, const_str_to_ptr!(key), const_str_to_ptr!(val)) }
    }

    pub fn publish(&self, url: &str) {
        unsafe { mk_pusher_publish(self.0, const_str_to_ptr!(url)) }
    }

    pub fn on_result(&self, cb: OnEventCallbackFn) {
        unsafe { mk_pusher_set_on_result(self.0, Some(on_push_event), box_to_mut_void_ptr!(cb)) }
    }

    pub fn on_shutdown(&self, cb: OnEventCallbackFn) {
        unsafe { mk_pusher_set_on_result(self.0, Some(on_push_event), box_to_mut_void_ptr!(cb)) }
    }
}

impl Drop for Pusher {
    fn drop(&mut self) {
        unsafe { mk_pusher_release(self.0) }
    }
}

#[derive(Debug, Default)]
pub struct PusherBuilder {
    schema: String,
    vhost: String,
    app: String,
    stream: String,
}

impl PusherBuilder {
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

    pub fn schema(mut self, schema: &str) -> Self {
        self.schema = schema.to_string();
        self
    }

    pub fn build(self) -> Pusher {
        Pusher(unsafe {
            mk_pusher_create(
                const_str_to_ptr!(self.schema),
                const_str_to_ptr!(self.vhost),
                const_str_to_ptr!(self.app),
                const_str_to_ptr!(self.stream),
            )
        })
    }
}

pub type OnEventCallbackFn = Box<dyn FnMut(i32, &str) + 'static>;
extern "C" fn on_push_event(
    user_data: *mut ::std::os::raw::c_void,
    err_code: ::std::os::raw::c_int,
    err_msg: *const ::std::os::raw::c_char,
) {
    unsafe {
        let cb: &mut OnEventCallbackFn = std::mem::transmute(user_data);
        cb(err_code, const_ptr_to_string!(err_msg).as_str());
    };
}
