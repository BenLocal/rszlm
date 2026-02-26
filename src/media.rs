use rszlm_sys::*;

use crate::{
    const_str_to_ptr,
    frame::Frame,
    obj::{CodecId, Track},
    DEFAULT_VHOST,
};

/// A media source/sender that pushes frames into ZLMediaKit for publishing.
///
/// Create a `Media` with [`Media::new`], configure video/audio with [`init_video`](Media::init_video)
/// and [`init_audio`](Media::init_audio) (or [`init_track`](Media::init_track)), then call
/// [`init_complete`](Media::init_complete) and feed frames via [`input_frame`](Media::input_frame).
///
/// # Example
///
/// ```ignore
/// let media = Media::new(
///     None,
///     "live",
///     "stream1",
///     0.0,
///     true,
///     false,
/// );
/// media.init_video(CodecId::H264, 1280, 720, 25.0, 1_000_000);
/// media.init_complete();
/// media.input_frame(&frame);
/// ```
pub struct Media(mk_media);

impl Media {
    /// Creates a new media source.
    ///
    /// # Arguments
    ///
    /// * `vhost` - Virtual host name; `None` for default.
    /// * `app` - Application name (e.g. `"live"`).
    /// * `stream` - Stream ID.
    /// * `duration` - Recording duration in seconds; `0.0` for no limit.
    /// * `hls_enabled` - Whether to enable HLS output.
    /// * `mp4_enabled` - Whether to enable MP4 recording.
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

    /// Creates a new media source.
    ///
    /// # Arguments
    ///
    /// * `vhost` - default vhost `__defaultVhost__`
    /// * `app` - Application name (e.g. `"live"`).
    /// * `stream` - Stream ID.
    /// * `duration` - Recording duration in seconds; `0.0` for no limit.
    /// * `hls_enabled` - Whether to enable HLS output.
    /// * `mp4_enabled` - Whether to enable MP4 recording.
    pub fn new_with_default_vhost(
        app: &str,
        stream: &str,
        duration: f32,
        hls_enabled: bool,
        mp4_enabled: bool,
    ) -> Self {
        Self::new(
            DEFAULT_VHOST,
            app,
            stream,
            duration,
            hls_enabled,
            mp4_enabled,
        )
    }

    /// Initializes a track from an existing [`Track`] (e.g. parsed from SDP or another source).
    pub fn init_track(&self, track: &Track) {
        unsafe { mk_media_init_track(self.0, track.inner()) }
    }

    /// Initializes the video track.
    ///
    /// Returns `true` on success. Call this before pushing video frames.
    ///
    /// # Arguments
    ///
    /// * `codec_id` - Video codec (e.g. H264, H265).
    /// * `width` - Frame width in pixels.
    /// * `height` - Frame height in pixels.
    /// * `fps` - Frames per second.
    /// * `bit_rate` - Bitrate in bits per second.
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

    /// Initializes the audio track.
    ///
    /// Returns `true` on success. Call this before pushing audio frames.
    ///
    /// # Arguments
    ///
    /// * `codec_id` - Audio codec (e.g. AAC).
    /// * `sample_rate` - Sample rate in Hz.
    /// * `channels` - Number of channels.
    /// * `bit_rate` - Bitrate in bits per second.
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

    /// Marks track setup as complete.
    ///
    /// Call this after all video/audio tracks are initialized (e.g. after `init_video` and/or
    /// `init_audio`). For single-track streams, ZLMediaKit otherwise waits ~3 seconds to see if
    /// more tracks will be added; calling this avoids that delay.
    pub fn init_complete(&self) {
        unsafe { mk_media_init_complete(self.0) }
    }

    /// Feeds one encoded frame (video or audio) into the media source.
    ///
    /// Returns `true` if the frame was accepted.
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
