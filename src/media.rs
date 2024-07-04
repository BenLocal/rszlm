use rszlm_sys::*;

use crate::{const_str_to_ptr, frame::Frame, obj::Track};

pub struct Media(mk_media);

impl Media {
    pub fn new(
        vhost: &str,
        app: &str,
        stream: &str,
        duration: f32,
        hls_enabled: bool,
        mp4_enabled: bool,
    ) -> Self {
        let vhost = const_str_to_ptr!(vhost);
        let app = const_str_to_ptr!(app);
        let stream = const_str_to_ptr!(stream);
        unsafe {
            mk_media_create(
                vhost.as_ptr(),
                app.as_ptr(),
                stream.as_ptr(),
                duration,
                hls_enabled as i32,
                mp4_enabled as i32,
            )
        }
        .into()
    }

    pub fn init_track(&self, track: &Track) {
        unsafe { mk_media_init_track(self.0, track.inner()) }
    }

    pub fn init_complete(&self) {
        unsafe { mk_media_init_complete(self.0) }
    }

    pub fn input_frame(&self, frame: &Frame) -> bool {
        unsafe { mk_media_input_frame(self.0, frame.as_c_ptr()) == 1 }
    }
}

impl Drop for Media {
    fn drop(&mut self) {
        unsafe { mk_media_release(self.0) }
    }
}

impl From<mk_media> for Media {
    fn from(sender: mk_media) -> Self {
        Media(sender)
    }
}
