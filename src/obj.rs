use std::ptr;

use rszlm_sys::*;

use crate::{const_ptr_to_string, const_str_to_ptr};

#[derive(Debug)]
pub struct SockInfo(mk_sock_info);

impl SockInfo {
    pub fn peer_ip(&self) -> String {
        unsafe {
            let buf = [0; 64].as_mut_ptr().cast();
            const_ptr_to_string!(mk_sock_info_peer_ip(self.0, buf))
        }
    }

    pub fn peer_port(&self) -> u16 {
        unsafe { mk_sock_info_peer_port(self.0) }
    }

    pub fn local_ip(&self) -> String {
        unsafe {
            let buf = [0; 32].as_mut_ptr().cast();
            const_ptr_to_string!(mk_sock_info_local_ip(self.0, buf))
        }
    }

    pub fn local_port(&self) -> u16 {
        unsafe { mk_sock_info_local_port(self.0) }
    }
}

impl From<mk_sock_info> for SockInfo {
    fn from(sender: mk_sock_info) -> Self {
        Self(sender)
    }
}

#[derive(Debug)]
pub struct MediaInfo(mk_media_info);

impl MediaInfo {
    pub fn params(&self) -> String {
        unsafe { const_ptr_to_string!(mk_media_info_get_params(self.0)) }
    }

    pub fn schema(&self) -> String {
        unsafe { const_ptr_to_string!(mk_media_info_get_schema(self.0)) }
    }

    pub fn vhost(&self) -> String {
        unsafe { const_ptr_to_string!(mk_media_info_get_vhost(self.0)) }
    }

    pub fn app(&self) -> String {
        unsafe { const_ptr_to_string!(mk_media_info_get_app(self.0)) }
    }

    pub fn stream(&self) -> String {
        unsafe { const_ptr_to_string!(mk_media_info_get_stream(self.0)) }
    }

    pub fn host(&self) -> String {
        unsafe { const_ptr_to_string!(mk_media_info_get_host(self.0)) }
    }

    pub fn port(&self) -> u16 {
        unsafe { mk_media_info_get_port(self.0) }
    }
}

impl From<mk_media_info> for MediaInfo {
    fn from(value: mk_media_info) -> Self {
        Self(value)
    }
}

#[derive(Debug)]
pub struct MediaSource(mk_media_source);

impl MediaSource {
    pub fn schema(&self) -> String {
        unsafe { const_ptr_to_string!(mk_media_source_get_schema(self.0)) }
    }

    pub fn app(&self) -> String {
        unsafe { const_ptr_to_string!(mk_media_source_get_app(self.0)) }
    }

    pub fn stream(&self) -> String {
        unsafe { const_ptr_to_string!(mk_media_source_get_stream(self.0)) }
    }

    pub fn reader_count(&self) -> i32 {
        unsafe { mk_media_source_get_reader_count(self.0) }
    }

    pub fn total_reader_count(&self) -> i32 {
        unsafe { mk_media_source_get_total_reader_count(self.0) }
    }

    pub fn track_count(&self) -> i32 {
        unsafe { mk_media_source_get_track_count(self.0) }
    }

    pub fn get_track(&self, index: i32) -> Option<Track> {
        unsafe {
            let track = mk_media_source_get_track(self.0, index);
            if track.is_null() {
                None
            } else {
                Some(Track(track))
            }
        }
    }

    pub fn close(&self, force: bool) -> bool {
        match unsafe { mk_media_source_close(self.0, force as i32) } {
            1 => true,
            _ => false,
        }
    }

    pub(crate) fn inner(&self) -> mk_media_source {
        self.0
    }
}

impl From<mk_media_source> for MediaSource {
    fn from(value: mk_media_source) -> Self {
        Self(value)
    }
}

pub struct Track(mk_track);

impl Track {
    pub fn new(codec_id: CodecId, args: Option<CodecArgs>) -> Self {
        let mut args = match args {
            Some(CodecArgs::Video(v)) => codec_args {
                video: codec_args__bindgen_ty_1 {
                    width: v.width,
                    height: v.height,
                    fps: v.fps,
                },
            },
            Some(CodecArgs::Audio(a)) => codec_args {
                audio: codec_args__bindgen_ty_2 {
                    channels: a.channels,
                    sample_rate: a.sample_rate,
                },
            },
            _ => codec_args {
                video: codec_args__bindgen_ty_1 {
                    width: 0,
                    height: 0,
                    fps: 0,
                },
            },
        };
        Self(unsafe { mk_track_create(codec_id.into(), &mut args) })
    }

    pub fn get_codec_id(&self) -> i32 {
        unsafe { mk_track_codec_id(self.0) }
    }

    pub fn get_codec_name(&self) -> String {
        unsafe { const_ptr_to_string!(mk_track_codec_name(self.0)) }
    }

    pub fn get_bit_rate(&self) -> i32 {
        unsafe { mk_track_bit_rate(self.0) }
    }

    pub fn is_video(&self) -> bool {
        unsafe { mk_track_is_video(self.0) == 1 }
    }

    pub fn video_width(&self) -> i32 {
        unsafe { mk_track_video_width(self.0) }
    }

    pub fn video_height(&self) -> i32 {
        unsafe { mk_track_video_height(self.0) }
    }

    pub fn video_fps(&self) -> i32 {
        unsafe { mk_track_video_fps(self.0) }
    }

    pub fn audio_sample_rate(&self) -> i32 {
        unsafe { mk_track_audio_sample_rate(self.0) }
    }

    pub fn audio_channel(&self) -> i32 {
        unsafe { mk_track_audio_channel(self.0) }
    }

    pub fn audio_sample_bit(&self) -> i32 {
        unsafe { mk_track_audio_sample_bit(self.0) }
    }

    pub(crate) fn inner(&self) -> mk_track {
        self.0
    }
}

impl Drop for Track {
    fn drop(&mut self) {
        unsafe { mk_track_unref(self.0) }
    }
}

pub enum CodecArgs {
    Video(VideoCodecArgs),
    Audio(AudioCodecArgs),
}

pub struct VideoCodecArgs {
    pub width: i32,
    pub height: i32,
    pub fps: i32,
}

pub struct AudioCodecArgs {
    pub channels: i32,
    pub sample_rate: i32,
}

#[derive(Debug)]
pub struct AuthInvoker(mk_auth_invoker, bool);

impl AuthInvoker {
    pub fn new(inner: mk_auth_invoker) -> Self {
        Self(inner, false)
    }

    pub fn allow(&self) {
        unsafe { mk_auth_invoker_do(self.0, ptr::null()) }
    }

    pub fn deny(&self, error: &str) {
        unsafe { mk_auth_invoker_do(self.0, const_str_to_ptr!(error)) }
    }
}

impl Clone for AuthInvoker {
    fn clone(&self) -> Self {
        Self(unsafe { mk_auth_invoker_clone(self.0) }, true)
    }
}

impl Drop for AuthInvoker {
    fn drop(&mut self) {
        unsafe {
            if self.1 {
                mk_auth_invoker_clone_release(self.0);
            }
        }
    }
}

impl From<mk_auth_invoker> for AuthInvoker {
    fn from(value: mk_auth_invoker) -> Self {
        Self(value, false)
    }
}

#[derive(Debug)]
pub struct Parser(mk_parser);

impl Parser {
    pub fn method(&self) -> String {
        unsafe { const_ptr_to_string!(mk_parser_get_method(self.0)) }
    }

    pub fn url(&self) -> String {
        unsafe { const_ptr_to_string!(mk_parser_get_url(self.0)) }
    }

    pub fn query_str(&self) -> String {
        unsafe { const_ptr_to_string!(mk_parser_get_url_params(self.0)) }
    }

    pub fn query(&self, key: &str) -> String {
        unsafe { const_ptr_to_string!(mk_parser_get_url_param(self.0, const_str_to_ptr!(key))) }
    }

    pub fn header(&self, key: &str) -> String {
        unsafe { const_ptr_to_string!(mk_parser_get_header(self.0, const_str_to_ptr!(key))) }
    }

    pub fn body(&self) -> String {
        unsafe { const_ptr_to_string!(mk_parser_get_tail(self.0)) }
    }
}

impl From<mk_parser> for Parser {
    fn from(value: mk_parser) -> Self {
        Parser(value)
    }
}

pub enum CodecId {
    H264,
    H265,
}

impl Into<i32> for CodecId {
    fn into(self) -> i32 {
        match self {
            CodecId::H264 => unsafe { MKCodecH264 },
            CodecId::H265 => unsafe { MKCodecH265 },
        }
    }
}

#[derive(Debug)]
pub struct RecordInfo(mk_record_info);

impl From<mk_record_info> for RecordInfo {
    fn from(value: mk_record_info) -> Self {
        RecordInfo(value)
    }
}

impl RecordInfo {
    pub fn start_time(&self) -> u64 {
        unsafe { mk_record_info_get_start_time(self.0) }
    }

    pub fn duration(&self) -> f32 {
        unsafe { mk_record_info_get_time_len(self.0) }
    }

    pub fn file_size(&self) -> usize {
        unsafe { mk_record_info_get_file_size(self.0) }
    }

    pub fn file_name(&self) -> String {
        unsafe { const_ptr_to_string!(mk_record_info_get_file_name(self.0)) }
    }

    pub fn file_path(&self) -> String {
        unsafe { const_ptr_to_string!(mk_record_info_get_file_path(self.0)) }
    }

    pub fn folder(&self) -> String {
        unsafe { const_ptr_to_string!(mk_record_info_get_folder(self.0)) }
    }

    pub fn app(&self) -> String {
        unsafe { const_ptr_to_string!(mk_record_info_get_app(self.0)) }
    }

    pub fn stream(&self) -> String {
        unsafe { const_ptr_to_string!(mk_record_info_get_stream(self.0)) }
    }

    pub fn vhost(&self) -> String {
        unsafe { const_ptr_to_string!(mk_record_info_get_vhost(self.0)) }
    }
}

pub struct RtcTransport(mk_rtc_transport);

impl From<mk_rtc_transport> for RtcTransport {
    fn from(value: mk_rtc_transport) -> Self {
        RtcTransport(value)
    }
}

impl RtcTransport {
    pub fn send_datachannel(&self, data: &[u8], sid: u16, ppid: u32) {
        unsafe {
            mk_rtc_send_datachannel(self.0, sid, ppid, data.as_ptr() as *const i8, data.len())
        }
    }
}

#[allow(dead_code)]
pub(crate) extern "C" fn on_user_data_free(_user_data: *mut std::os::raw::c_void) {
    // do nothing
}
