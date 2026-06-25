use std::{os::raw::c_char, sync::Arc};

use rszlm_sys::*;

use crate::{box_to_mut_void_ptr, obj::CodecId};

pub struct Frame(mk_frame);

impl Frame {
    pub fn as_c_ptr(&self) -> mk_frame {
        self.0
    }

    // cb是None时, 内部会做数据拷贝
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

    pub fn new_zero_copy(codec_id: CodecId, dts: u64, pts: u64, data: Arc<[u8]>) -> Self {
        unsafe extern "C" fn release_arc_data(
            user_data: *mut ::std::os::raw::c_void,
            _ptr: *mut ::std::os::raw::c_char,
        ) {
            if !user_data.is_null() {
                // release the Arc
                let _ = Box::from_raw(user_data as *mut Arc<[u8]>);
            }
        }

        let data_ptr = data.as_ptr() as *const c_char;
        let data_len = data.len();

        let user_data = Box::into_raw(Box::new(data)) as *mut ::std::os::raw::c_void;

        Self(unsafe {
            mk_frame_create(
                codec_id.into(),
                dts,
                pts,
                data_ptr,
                data_len,
                Some(release_arc_data),
                user_data,
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

pub type OnH264SplitterFrameFn = Box<dyn FnMut(&[u8]) + Send + Sync + 'static>;
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

pub struct H264Splitter(mk_h264_splitter, *mut ::std::os::raw::c_void);

impl H264Splitter {
    pub fn new(on_frame: OnH264SplitterFrameFn, h265: bool) -> Self {
        let user_data: *mut ::std::os::raw::c_void = box_to_mut_void_ptr!(on_frame);
        Self(
            unsafe {
                mk_h264_splitter_create(Some(on_mk_h264_splitter_frame), user_data, h265 as i32)
            },
            user_data,
        )
    }

    pub fn input(&self, data: &[u8]) {
        unsafe {
            mk_h264_splitter_input_data(self.0, data.as_ptr() as *const c_char, data.len() as i32)
        }
    }
}

impl Drop for H264Splitter {
    fn drop(&mut self) {
        unsafe {
            mk_h264_splitter_release(self.0);
            if !self.1.is_null() {
                // reclaim the boxed callback leaked into `user_data`
                let _ = Box::from_raw(self.1 as *mut OnH264SplitterFrameFn);
            }
        }
    }
}

unsafe impl Send for H264Splitter {}
unsafe impl Sync for H264Splitter {}
