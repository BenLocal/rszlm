use rszlm_sys::*;

use crate::const_str_to_ptr;

pub struct FlvRecorder(mk_flv_recorder);

impl FlvRecorder {
    pub fn new() -> Self {
        Self(unsafe { mk_flv_recorder_create() })
    }

    pub fn start(&self, vhost: &str, app: &str, stream: &str, file_path: &str) -> i32 {
        unsafe {
            mk_flv_recorder_start(
                self.0,
                const_str_to_ptr!(vhost),
                const_str_to_ptr!(app),
                const_str_to_ptr!(stream),
                const_str_to_ptr!(file_path),
            )
        }
    }
}

impl Drop for FlvRecorder {
    fn drop(&mut self) {
        unsafe { mk_flv_recorder_release(self.0) }
    }
}

pub struct Recorder;

impl Recorder {
    /// 是否正在录制
    /// - typ:
    ///    - 0:hls
    ///    - 1:MP4
    pub fn is_recording(typ: u32, vhost: &str, app: &str, stream: &str) -> bool {
        unsafe {
            mk_recorder_is_recording(
                typ as i32,
                const_str_to_ptr!(vhost),
                const_str_to_ptr!(app),
                const_str_to_ptr!(stream),
            ) == 1
        }
    }

    pub fn start(
        typ: u32,
        vhost: &str,
        app: &str,
        stream: &str,
        file_path: &str,
        max_seconds: usize,
    ) -> i32 {
        unsafe {
            mk_recorder_start(
                typ as i32,
                const_str_to_ptr!(vhost),
                const_str_to_ptr!(app),
                const_str_to_ptr!(stream),
                const_str_to_ptr!(file_path),
                max_seconds as usize,
            )
        }
    }

    pub fn stop(typ: u32, vhost: &str, app: &str, stream: &str) -> i32 {
        unsafe {
            mk_recorder_stop(
                typ as i32,
                const_str_to_ptr!(vhost),
                const_str_to_ptr!(app),
                const_str_to_ptr!(stream),
            )
        }
    }
}
