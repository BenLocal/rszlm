use rszlm_sys::*;

use crate::{
    const_str_to_ptr,
    frame::Frame,
    obj::{CodecId, Track},
};

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

    pub fn init_video(
        &self,
        codec_id: CodecId,
        width: i32,
        height: i32,
        fps: f32,
        bit_rate: i32,
    ) -> bool {
        unsafe { mk_media_init_video(self.0, codec_id.into(), width, height, fps, bit_rate) == 1 }
    }

    pub fn init_audio(
        &self,
        codec_id: CodecId,
        sample_rate: i32,
        channels: i32,
        bit_rate: i32,
    ) -> bool {
        unsafe {
            mk_media_init_audio(self.0, codec_id.into(), sample_rate, channels, bit_rate) == 1
        }
    }

    /// 初始化h264/h265/aac完毕后调用此函数，
    /// 在单track(只有音频或视频)时，因为ZLMediaKit不知道后续是否还要添加track，所以会多等待3秒钟
    /// 如果产生的流是单Track类型，请调用此函数以便加快流生成速度，当然不调用该函数，影响也不大(会多等待3秒)
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

unsafe impl Send for Media {}
unsafe impl Sync for Media {}
