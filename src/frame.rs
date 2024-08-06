use std::os::raw::c_char;

use rszlm_sys::*;

use crate::obj::CodecId;

pub struct Frame(mk_frame);

impl Frame {
    pub fn as_c_ptr(&self) -> mk_frame {
        self.0
    }

    pub fn new(codec_id: CodecId, dts: u64, pts: u64, buf: Vec<u8>) -> Self {
        Self(unsafe {
            mk_frame_create(
                codec_id.into(),
                dts,
                pts,
                buf.as_ptr() as *const c_char,
                buf.len(),
                None,
                std::ptr::null_mut(),
            )
        })
    }
}

impl Drop for Frame {
    fn drop(&mut self) {
        unsafe { mk_frame_unref(self.0) }
    }
}
