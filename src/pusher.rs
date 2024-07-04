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
        let key = const_str_to_ptr!(key);
        let val = const_str_to_ptr!(val);
        unsafe { mk_pusher_set_option(self.0, key.as_ptr(), val.as_ptr()) }
    }

    pub fn publish(&self, url: &str) {
        let url = const_str_to_ptr!(url);
        unsafe { mk_pusher_publish(self.0, url.as_ptr()) }
    }

    pub fn on_result<T>(&self, cb: T)
    where
        T: FnMut(i32, String) + 'static,
    {
        self.on_result_inner(Box::new(cb))
    }

    fn on_result_inner(&self, cb: OnEventCallbackFn) {
        unsafe { mk_pusher_set_on_result(self.0, Some(on_push_event), box_to_mut_void_ptr!(cb)) }
    }

    pub fn on_shutdown<T>(&self, cb: T)
    where
        T: FnMut(i32, String) + 'static,
    {
        self.on_shutdown_inner(Box::new(cb))
    }

    pub fn on_shutdown_inner(&self, cb: OnEventCallbackFn) {
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
        let vhost = const_str_to_ptr!(self.vhost);
        let app = const_str_to_ptr!(self.app);
        let stream = const_str_to_ptr!(self.stream);
        let schema = const_str_to_ptr!(self.schema);

        Pusher(unsafe {
            mk_pusher_create(
                schema.as_ptr(),
                vhost.as_ptr(),
                app.as_ptr(),
                stream.as_ptr(),
            )
        })
    }
}

pub type OnEventCallbackFn = Box<dyn FnMut(i32, String) + 'static>;
extern "C" fn on_push_event(
    user_data: *mut ::std::os::raw::c_void,
    err_code: ::std::os::raw::c_int,
    err_msg: *const ::std::os::raw::c_char,
) {
    unsafe {
        let cb: &mut OnEventCallbackFn = std::mem::transmute(user_data);
        cb(err_code, const_ptr_to_string!(err_msg));
    };
}
