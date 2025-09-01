use std::os::raw::c_char;

use rszlm_sys::*;

use crate::{box_to_mut_void_ptr, obj::CodecId};

pub struct Frame(mk_frame);

impl Frame {
    pub fn as_c_ptr(&self) -> mk_frame {
        self.0
    }

    pub fn new<T: AsRef<[u8]>>(codec_id: CodecId, dts: u64, pts: u64, buf: T) -> Self {
        Self(unsafe {
            mk_frame_create(
                codec_id.into(),
                dts,
                pts,
                buf.as_ref().as_ptr() as *const c_char,
                buf.as_ref().len(),
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

unsafe impl Send for Frame {}
unsafe impl Sync for Frame {}

pub type OnH264SplitterFrameFn = Box<dyn FnMut(&[u8]) + 'static>;
unsafe extern "C" fn on_mk_h264_splitter_frame(
    user_data: *mut ::std::os::raw::c_void,
    _splitter: mk_h264_splitter,
    frame: *const ::std::os::raw::c_char,
    size: ::std::os::raw::c_int,
) {
    unsafe {
        let data = std::slice::from_raw_parts(frame as *const u8, size as usize);
        let cb: &mut OnH264SplitterFrameFn = std::mem::transmute(user_data);
        cb(data);
    };
}

pub struct H264Splitter(mk_h264_splitter);

impl H264Splitter {
    pub fn new(on_frame: OnH264SplitterFrameFn, h265: bool) -> Self {
        Self(unsafe {
            mk_h264_splitter_create(
                Some(on_mk_h264_splitter_frame),
                box_to_mut_void_ptr!(on_frame),
                h265 as i32,
            )
        })
    }

    pub fn input(&self, data: &[u8]) {
        unsafe {
            mk_h264_splitter_input_data(self.0, data.as_ptr() as *const c_char, data.len() as i32)
        }
    }
}

unsafe impl Send for H264Splitter {}
unsafe impl Sync for H264Splitter {}
