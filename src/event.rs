use core::str;
use once_cell::sync::Lazy;
use rszlm_sys::*;
use std::{ffi::CString, sync::RwLock};

use crate::{
    as_cstr_array, const_ptr_to_string, const_str_to_ptr,
    obj::{AuthInvoker, MediaInfo, MediaSource, Parser, RecordInfo, RtcTransport, SockInfo},
};

pub static EVENTS: Lazy<RwLock<Event>> = Lazy::new(|| RwLock::new(Event::new()));

#[derive(Default)]
pub struct Event {
    inner: mk_events,
    on_media_changed: Option<Box<dyn Fn(MediaChangedMessage) + Sync + Send>>,
    on_media_publish: Option<Box<dyn Fn(MediaPublishMessage) + Sync + Send>>,
    on_media_not_found: Option<Box<dyn Fn(MediaNotFoundMessage) -> bool + Sync + Send>>,
    on_media_play: Option<Box<dyn Fn(MediaPlayMessage) -> anyhow::Result<()> + Sync + Send>>,
    on_media_no_reader: Option<Box<dyn Fn(MediaNoReaderMessage) + Sync + Send>>,
    on_http_request: Option<Box<dyn Fn(HttpRequestMessage) -> bool + Sync + Send>>,
    on_http_before_access: Option<Box<dyn Fn(HttpBeforeRequestMessage) -> String + Sync + Send>>,
    on_rtsp_get_realm: Option<Box<dyn Fn(RtspGetRealmMessage) + Sync + Send>>,
    on_rtsp_auth: Option<Box<dyn Fn(RtspAuthMessage) + Sync + Send>>,
    on_record_mp4: Option<Box<dyn Fn(RecordMp4Message) + Sync + Send>>,
    on_record_ts: Option<Box<dyn Fn(RecordTsMessage) + Sync + Send>>,
    on_shell_login: Option<Box<dyn Fn(ShellLoginMessage) + Sync + Send>>,
    on_flow_report: Option<Box<dyn Fn(FlowReportMessage) + Sync + Send>>,
    on_log: Option<Box<dyn Fn(LogMessage) + Sync + Send>>,
    on_media_send_rtp_stop: Option<Box<dyn Fn(MediaSendRtpStopMessage) + Sync + Send>>,
    on_rtc_sctp_connecting: Option<Box<dyn Fn(RtcSctpStateMessage) + Sync + Send>>,
    on_rtc_sctp_connected: Option<Box<dyn Fn(RtcSctpStateMessage) + Sync + Send>>,
    on_rtc_sctp_failed: Option<Box<dyn Fn(RtcSctpStateMessage) + Sync + Send>>,
    on_rtc_sctp_closed: Option<Box<dyn Fn(RtcSctpStateMessage) + Sync + Send>>,
    on_rtc_sctp_send: Option<Box<dyn Fn(RtcSctpStateMessage) + Sync + Send>>,
    on_rtc_sctp_received: Option<Box<dyn Fn(RtcSctpStateMessage) + Sync + Send>>,
}

impl Event {
    fn new() -> Self {
        // let event = mk_events {};
        // unsafe { mk_events_listen(&event as *const mk_events) }
        Event {
            ..Default::default()
        }
    }

    fn subscribe(&mut self) {
        unsafe { mk_events_listen(&self.inner as *const mk_events) }
    }

    pub fn on_media_changed(&mut self, cb: impl Fn(MediaChangedMessage) + Sync + Send + 'static) {
        self.on_media_changed = Some(Box::new(cb));
        self.inner.on_mk_media_changed = Some(on_mk_media_changed);
        self.subscribe();
    }

    pub fn on_media_publish(&mut self, cb: impl Fn(MediaPublishMessage) + Sync + Send + 'static) {
        self.on_media_publish = Some(Box::new(cb));
        self.inner.on_mk_media_publish = Some(on_mk_media_publish);
        self.subscribe();
    }

    pub fn on_media_not_found(
        &mut self,
        cb: impl Fn(MediaNotFoundMessage) -> bool + Sync + Send + 'static,
    ) {
        self.on_media_not_found = Some(Box::new(cb));
        self.inner.on_mk_media_not_found = Some(on_mk_media_not_found);
        self.subscribe();
    }

    pub fn on_media_no_reader(
        &mut self,
        cb: impl Fn(MediaNoReaderMessage) + Sync + Send + 'static,
    ) {
        self.on_media_no_reader = Some(Box::new(cb));
        self.inner.on_mk_media_no_reader = Some(on_mk_media_no_reader);
        self.subscribe();
    }

    pub fn on_media_play(
        &mut self,
        cb: impl Fn(MediaPlayMessage) -> anyhow::Result<()> + Sync + Send + 'static,
    ) {
        self.on_media_play = Some(Box::new(cb));
        self.inner.on_mk_media_play = Some(on_mk_media_play);
        self.subscribe();
    }

    pub fn on_http_request(
        &mut self,
        cb: impl Fn(HttpRequestMessage) -> bool + Sync + Send + 'static,
    ) {
        self.on_http_request = Some(Box::new(cb));
        self.inner.on_mk_http_request = Some(on_mk_http_request);
        self.subscribe();
    }

    pub fn on_http_before_access(
        &mut self,
        cb: impl Fn(HttpBeforeRequestMessage) -> String + Sync + Send + 'static,
    ) {
        self.on_http_before_access = Some(Box::new(cb));
        self.inner.on_mk_http_before_access = Some(on_mk_http_before_access);
        self.subscribe();
    }

    pub fn on_rtsp_get_realm(&mut self, cb: impl Fn(RtspGetRealmMessage) + Sync + Send + 'static) {
        self.on_rtsp_get_realm = Some(Box::new(cb));
        self.inner.on_mk_rtsp_get_realm = Some(on_mk_rtsp_get_realm);
        self.subscribe();
    }

    pub fn on_rtsp_auth(&mut self, cb: impl Fn(RtspAuthMessage) + Sync + Send + 'static) {
        self.on_rtsp_auth = Some(Box::new(cb));
        self.inner.on_mk_rtsp_auth = Some(on_mk_rtsp_auth);
        self.subscribe();
    }

    pub fn on_record_mp4(&mut self, cb: impl Fn(RecordMp4Message) + Sync + Send + 'static) {
        self.on_record_mp4 = Some(Box::new(cb));
        self.inner.on_mk_record_mp4 = Some(on_mk_record_mp4);
        self.subscribe();
    }

    pub fn on_record_ts(&mut self, cb: impl Fn(RecordTsMessage) + Sync + Send + 'static) {
        self.on_record_ts = Some(Box::new(cb));
        self.inner.on_mk_record_ts = Some(on_mk_record_ts);
        self.subscribe();
    }

    pub fn on_shell_login(&mut self, cb: impl Fn(ShellLoginMessage) + Sync + Send + 'static) {
        self.on_shell_login = Some(Box::new(cb));
        self.inner.on_mk_shell_login = Some(on_mk_shell_login);
        self.subscribe();
    }

    pub fn on_flow_report(&mut self, cb: impl Fn(FlowReportMessage) + Sync + Send + 'static) {
        self.on_flow_report = Some(Box::new(cb));
        self.inner.on_mk_flow_report = Some(on_mk_flow_report);
        self.subscribe();
    }

    pub fn on_log(&mut self, cb: impl Fn(LogMessage) + Sync + Send + 'static) {
        self.on_log = Some(Box::new(cb));
        self.inner.on_mk_log = Some(on_mk_log);
        self.subscribe();
    }

    pub fn on_media_send_rtp_stop(
        &mut self,
        cb: impl Fn(MediaSendRtpStopMessage) + Sync + Send + 'static,
    ) {
        self.on_media_send_rtp_stop = Some(Box::new(cb));
        self.inner.on_mk_media_send_rtp_stop = Some(on_mk_media_send_rtp_stop);
        self.subscribe();
    }

    pub fn on_rtc_sctp_connecting(
        &mut self,
        cb: impl Fn(RtcSctpStateMessage) + Sync + Send + 'static,
    ) {
        self.on_rtc_sctp_connecting = Some(Box::new(cb));
        self.inner.on_mk_rtc_sctp_connecting = Some(on_mk_rtc_sctp_connecting);
        self.subscribe();
    }

    pub fn on_rtc_sctp_connected(
        &mut self,
        cb: impl Fn(RtcSctpStateMessage) + Sync + Send + 'static,
    ) {
        self.on_rtc_sctp_connected = Some(Box::new(cb));
        self.inner.on_mk_rtc_sctp_connected = Some(on_mk_rtc_sctp_connected);
        self.subscribe();
    }

    pub fn on_rtc_sctp_closed(&mut self, cb: impl Fn(RtcSctpStateMessage) + Sync + Send + 'static) {
        self.on_rtc_sctp_closed = Some(Box::new(cb));
        self.inner.on_mk_rtc_sctp_closed = Some(on_mk_rtc_sctp_closed);
        self.subscribe();
    }

    pub fn on_rtc_sctp_send(&mut self, cb: impl Fn(RtcSctpStateMessage) + Sync + Send + 'static) {
        self.on_rtc_sctp_send = Some(Box::new(cb));
        self.inner.on_mk_rtc_sctp_send = Some(on_mk_rtc_sctp_send);
        self.subscribe();
    }

    pub fn on_rtc_sctp_received(
        &mut self,
        cb: impl Fn(RtcSctpStateMessage) + Sync + Send + 'static,
    ) {
        self.on_rtc_sctp_received = Some(Box::new(cb));
        self.inner.on_mk_rtc_sctp_received = Some(on_mk_rtc_sctp_received);
        self.subscribe();
    }

    pub fn on_rtc_sctp_failed(&mut self, cb: impl Fn(RtcSctpStateMessage) + Sync + Send + 'static) {
        self.on_rtc_sctp_failed = Some(Box::new(cb));
        self.inner.on_mk_rtc_sctp_failed = Some(on_mk_rtc_sctp_failed);
        self.subscribe();
    }
}

pub enum RtcSctpStateMessage {
    Connecting(RtcTransport),
    Connected(RtcTransport),
    Closed(RtcTransport),
    Failed(RtcTransport),
    Send(RtcTransport, Vec<u8>),
    Received(RtcTransport, u16, u32, Vec<u8>),
}

extern "C" fn on_mk_rtc_sctp_failed(rtc_transport: mk_rtc_transport) {
    if let Some(cb) = &EVENTS.read().unwrap().on_rtc_sctp_failed {
        cb(RtcSctpStateMessage::Failed(rtc_transport.into()));
    }
}

extern "C" fn on_mk_rtc_sctp_received(
    rtc_transport: mk_rtc_transport,
    stream_id: u16,
    ppid: u32,
    msg: *const u8,
    len: usize,
) {
    if let Some(cb) = &EVENTS.read().unwrap().on_rtc_sctp_received {
        let data = unsafe { std::slice::from_raw_parts(msg, len) };
        cb(RtcSctpStateMessage::Received(
            rtc_transport.into(),
            stream_id,
            ppid,
            data.to_vec(),
        ));
    }
}

extern "C" fn on_mk_rtc_sctp_send(rtc_transport: mk_rtc_transport, msg: *const u8, len: usize) {
    if let Some(cb) = &EVENTS.read().unwrap().on_rtc_sctp_send {
        let data = unsafe { std::slice::from_raw_parts(msg, len) };
        cb(RtcSctpStateMessage::Send(
            rtc_transport.into(),
            data.to_vec(),
        ));
    }
}

extern "C" fn on_mk_rtc_sctp_closed(rtc_transport: mk_rtc_transport) {
    if let Some(cb) = &EVENTS.read().unwrap().on_rtc_sctp_closed {
        cb(RtcSctpStateMessage::Closed(rtc_transport.into()));
    }
}

extern "C" fn on_mk_rtc_sctp_connected(rtc_transport: mk_rtc_transport) {
    if let Some(cb) = &EVENTS.read().unwrap().on_rtc_sctp_connected {
        cb(RtcSctpStateMessage::Connected(rtc_transport.into()));
    }
}

extern "C" fn on_mk_rtc_sctp_connecting(rtc_transport: mk_rtc_transport) {
    if let Some(cb) = &EVENTS.read().unwrap().on_rtc_sctp_connecting {
        cb(RtcSctpStateMessage::Connecting(rtc_transport.into()));
    }
}

pub struct MediaSendRtpStopMessage {
    pub vhost: String,
    pub app: String,
    pub stream: String,
    pub ssrc: String,
    pub err: i32,
    pub msg: String,
}

extern "C" fn on_mk_media_send_rtp_stop(
    vhost: *const ::std::os::raw::c_char,
    app: *const ::std::os::raw::c_char,
    stream: *const ::std::os::raw::c_char,
    ssrc: *const ::std::os::raw::c_char,
    err: ::std::os::raw::c_int,
    msg: *const ::std::os::raw::c_char,
) {
    if let Some(cb) = &EVENTS.read().unwrap().on_media_send_rtp_stop {
        let (vhost, app, stream, ssrc, err, msg) = unsafe {
            (
                const_ptr_to_string!(vhost),
                const_ptr_to_string!(app),
                const_ptr_to_string!(stream),
                const_ptr_to_string!(ssrc),
                err,
                const_ptr_to_string!(msg),
            )
        };

        cb(MediaSendRtpStopMessage {
            vhost,
            app,
            stream,
            ssrc,
            err,
            msg,
        })
    }
}

pub struct LogMessage {
    pub level: i32,
    pub file: String,
    pub line: i32,
    pub function: String,
    pub message: String,
}

extern "C" fn on_mk_log(
    level: ::std::os::raw::c_int,
    file: *const ::std::os::raw::c_char,
    line: ::std::os::raw::c_int,
    function: *const ::std::os::raw::c_char,
    message: *const ::std::os::raw::c_char,
) {
    if let Some(cb) = &EVENTS.read().unwrap().on_log {
        let (file, function, message) = unsafe {
            (
                const_ptr_to_string!(file),
                const_ptr_to_string!(function),
                const_ptr_to_string!(message),
            )
        };

        cb(LogMessage {
            level,
            file,
            line,
            function,
            message,
        })
    }
}

pub struct FlowReportMessage {
    pub url_info: MediaInfo,
    pub total_bytes: usize,
    pub total_seconds: usize,
    pub is_player: bool,
    pub sender: SockInfo,
}

extern "C" fn on_mk_flow_report(
    url_info: mk_media_info,
    total_bytes: usize,
    total_seconds: usize,
    is_player: ::std::os::raw::c_int,
    sender: mk_sock_info,
) {
    if let Some(cb) = &EVENTS.read().unwrap().on_flow_report {
        cb(FlowReportMessage {
            url_info: url_info.into(),
            total_bytes,
            total_seconds,
            is_player: is_player != 0,
            sender: sender.into(),
        })
    }
}

pub struct ShellLoginMessage {
    pub user_name: String,
    pub passwd: String,
    pub invoker: AuthInvoker,
    pub sender: SockInfo,
}

extern "C" fn on_mk_shell_login(
    user_name: *const ::std::os::raw::c_char,
    passwd: *const ::std::os::raw::c_char,
    invoker: mk_auth_invoker,
    sender: mk_sock_info,
) {
    if let Some(cb) = &EVENTS.read().unwrap().on_shell_login {
        let (u, p) = unsafe {
            (
                const_ptr_to_string!(user_name),
                const_ptr_to_string!(passwd),
            )
        };
        cb(ShellLoginMessage {
            user_name: u,
            passwd: p,
            invoker: AuthInvoker::from(invoker),
            sender: SockInfo::from(sender),
        })
    }
}

pub struct RecordTsMessage {
    pub ts: RecordInfo,
}

extern "C" fn on_mk_record_ts(ts: mk_record_info) {
    if let Some(cb) = &EVENTS.read().unwrap().on_record_ts {
        cb(RecordTsMessage {
            ts: RecordInfo::from(ts),
        })
    }
}

pub struct RecordMp4Message {
    pub mp4: RecordInfo,
}

extern "C" fn on_mk_record_mp4(mp4: mk_record_info) {
    if let Some(cb) = &EVENTS.read().unwrap().on_record_mp4 {
        cb(RecordMp4Message {
            mp4: RecordInfo::from(mp4),
        })
    }
}

extern "C" fn on_mk_rtsp_auth(
    url_info: mk_media_info,
    realm: *const ::std::os::raw::c_char,
    user_name: *const ::std::os::raw::c_char,
    must_no_encrypt: ::std::os::raw::c_int,
    invoker: mk_rtsp_auth_invoker,
    sender: mk_sock_info,
) {
    if let Some(cb) = &EVENTS.read().unwrap().on_rtsp_auth {
        let (realm, user_name) =
            unsafe { (const_ptr_to_string!(realm), const_ptr_to_string!(user_name)) };

        cb(RtspAuthMessage {
            url_info: url_info.into(),
            realm,
            user_name,
            must_no_encrypt: must_no_encrypt != 0,
            invoker: invoker.into(),
            sender: sender.into(),
        })
    }
}

pub struct RtspAuthMessage {
    pub url_info: MediaInfo,
    pub realm: String,
    pub user_name: String,
    pub must_no_encrypt: bool,
    pub invoker: RtspAuthInvoker,
    pub sender: SockInfo,
}

extern "C" fn on_mk_rtsp_get_realm(
    url_info: mk_media_info,
    invoker: mk_rtsp_get_realm_invoker,
    sender: mk_sock_info,
) {
    if let Some(cb) = &EVENTS.read().unwrap().on_rtsp_get_realm {
        cb(RtspGetRealmMessage {
            url_info: url_info.into(),
            sender: sender.into(),
            invoker: invoker.into(),
        })
    }
}

pub struct RtspGetRealmMessage {
    pub url_info: MediaInfo,
    pub sender: SockInfo,
    pub invoker: RtspGetRealmInvoker,
}

pub struct RtspAuthInvoker(mk_rtsp_auth_invoker, bool);

impl RtspAuthInvoker {
    pub fn new(invoker: mk_rtsp_auth_invoker) -> Self {
        Self(invoker, false)
    }

    pub fn call(&self, pwd_or_md5: &str, encrypted: bool) {
        unsafe { mk_rtsp_auth_invoker_do(self.0, encrypted as i32, const_str_to_ptr!(pwd_or_md5)) }
    }
}

impl Clone for RtspAuthInvoker {
    fn clone(&self) -> Self {
        RtspAuthInvoker(unsafe { mk_rtsp_auth_invoker_clone(self.0) }, true)
    }
}

impl Drop for RtspAuthInvoker {
    fn drop(&mut self) {
        if self.1 {
            unsafe { mk_rtsp_auth_invoker_clone_release(self.0) }
        }
    }
}

impl From<mk_rtsp_auth_invoker> for RtspAuthInvoker {
    fn from(inner: mk_rtsp_auth_invoker) -> Self {
        Self(inner, false)
    }
}

#[derive(Debug)]
pub struct RtspGetRealmInvoker(mk_rtsp_get_realm_invoker, bool);

impl RtspGetRealmInvoker {
    pub fn new(invoker: mk_rtsp_get_realm_invoker) -> Self {
        Self(invoker, false)
    }

    pub fn call(&self, realm: &str) {
        unsafe { mk_rtsp_get_realm_invoker_do(self.0, const_str_to_ptr!(realm)) }
    }
}

impl Clone for RtspGetRealmInvoker {
    fn clone(&self) -> Self {
        RtspGetRealmInvoker(unsafe { mk_rtsp_get_realm_invoker_clone(self.0) }, true)
    }
}

impl Drop for RtspGetRealmInvoker {
    fn drop(&mut self) {
        if self.1 {
            unsafe { mk_rtsp_get_realm_invoker_clone_release(self.0) }
        }
    }
}

impl From<mk_rtsp_get_realm_invoker> for RtspGetRealmInvoker {
    fn from(value: mk_rtsp_get_realm_invoker) -> Self {
        Self(value, false)
    }
}

extern "C" fn on_mk_http_before_access(
    parser: mk_parser,
    mut path: *mut ::std::os::raw::c_char,
    sender: mk_sock_info,
) {
    if let Some(cb) = &EVENTS.read().unwrap().on_http_before_access {
        let old_path = unsafe { const_ptr_to_string!(path) };
        let u = cb(HttpBeforeRequestMessage {
            sender: sender.into(),
            parser: parser.into(),
            path: old_path,
        });

        let _ = CString::new(u).map(|v| path = v.into_raw());
    }
}

pub struct HttpBeforeRequestMessage {
    pub sender: SockInfo,
    pub parser: Parser,
    pub path: String,
}

extern "C" fn on_mk_http_request(
    parser: mk_parser,
    invoker: mk_http_response_invoker,
    consumed: *mut ::std::os::raw::c_int,
    sender: mk_sock_info,
) {
    if let Some(cb) = &EVENTS.read().unwrap().on_http_request {
        let res = cb(HttpRequestMessage {
            sender: sender.into(),
            parser: parser.into(),
            invoker: HttpResponseInvoker::from(invoker),
        });
        unsafe { *consumed = res as i32 };
    }
}

#[derive(Debug)]
pub struct HttpRequestMessage {
    pub sender: SockInfo,
    pub parser: Parser,
    pub invoker: HttpResponseInvoker,
}

#[derive(Debug)]
pub struct HttpResponseInvoker(mk_http_response_invoker, bool);

impl HttpResponseInvoker {
    pub fn invoke(&self, code: i32, headers: Vec<String>, body: &str) {
        let header_ptr = as_cstr_array(&headers);
        unsafe {
            mk_http_response_invoker_do_string(self.0, code, header_ptr, const_str_to_ptr!(body))
        }
    }
}

impl From<mk_http_response_invoker> for HttpResponseInvoker {
    fn from(invoker: mk_http_response_invoker) -> Self {
        HttpResponseInvoker(invoker, false)
    }
}

impl Clone for HttpResponseInvoker {
    fn clone(&self) -> Self {
        HttpResponseInvoker(unsafe { mk_http_response_invoker_clone(self.0) }, true)
    }
}

impl Drop for HttpResponseInvoker {
    fn drop(&mut self) {
        if self.1 {
            unsafe {
                mk_http_response_invoker_clone(self.0);
            }
        }
    }
}

pub(crate) extern "C" fn on_mk_media_changed(
    regist: ::std::os::raw::c_int,
    sender: mk_media_source,
) {
    if let Some(cb) = &EVENTS.read().unwrap().on_media_changed {
        match regist {
            0 => cb(MediaChangedMessage::UnRegist(sender.into())),
            1 => cb(MediaChangedMessage::Regist(sender.into())),
            _ => {}
        };
    }
}

pub enum MediaChangedMessage {
    Regist(MediaSource),
    UnRegist(MediaSource),
}

pub(crate) extern "C" fn on_mk_media_no_reader(sender: mk_media_source) {
    if let Some(cb) = &EVENTS.read().unwrap().on_media_no_reader {
        cb(MediaNoReaderMessage {
            sender: sender.into(),
        });
    }
}

#[derive(Debug)]
pub struct MediaNoReaderMessage {
    pub sender: MediaSource,
}

pub(crate) extern "C" fn on_mk_media_not_found(
    url_info: mk_media_info,
    sender: mk_sock_info,
) -> std::os::raw::c_int {
    if let Some(cb) = &EVENTS.read().unwrap().on_media_not_found {
        return cb(MediaNotFoundMessage {
            url_info: url_info.into(),
            sock_info: sender.into(),
        }) as i32;
    }

    0
}

#[derive(Debug)]
pub struct MediaNotFoundMessage {
    pub url_info: MediaInfo,
    pub sock_info: SockInfo,
}

pub(crate) extern "C" fn on_mk_media_play(
    url_info: mk_media_info,
    invoker: mk_auth_invoker,
    sender: mk_sock_info,
) {
    let invoker = AuthInvoker::new(invoker);
    if let Some(cb) = &EVENTS.read().unwrap().on_media_play {
        let url_info = MediaInfo::from(url_info);
        let sock_info = SockInfo::from(sender);

        match cb(MediaPlayMessage {
            url_info,
            sender: sock_info,
        }) {
            Ok(_) => invoker.allow(),
            Err(e) => invoker.deny(&format!("on_media_play callback error: {:?}", e)),
        }
    } else {
        invoker.allow()
    }
}

#[derive(Debug)]
pub struct MediaPlayMessage {
    pub url_info: MediaInfo,
    pub sender: SockInfo,
}

pub(crate) extern "C" fn on_mk_media_publish(
    url_info: mk_media_info,
    invoker: mk_publish_auth_invoker,
    sender: mk_sock_info,
) {
    if let Some(cb) = &EVENTS.read().unwrap().on_media_publish {
        cb(MediaPublishMessage {
            url_info: url_info.into(),
            auth_invoker: invoker.into(),
            sender_inner: sender.into(),
        });
    }
}

pub struct MediaPublishMessage {
    pub url_info: MediaInfo,
    pub auth_invoker: PublishAuthInvoker,
    pub sender_inner: SockInfo,
}

pub struct PublishAuthInvoker(mk_publish_auth_invoker, bool);

impl PublishAuthInvoker {
    pub fn call(&self, err_msg: &str, enable_mp4: bool, enable_hls: bool) -> anyhow::Result<()> {
        unsafe {
            mk_publish_auth_invoker_do(
                self.0,
                CString::new(err_msg)?.as_ptr(),
                enable_mp4 as i32,
                enable_hls as i32,
            )
        };
        Ok(())
    }

    #[allow(dead_code)]
    pub fn call_with_config(&self, err_msg: &str) -> anyhow::Result<()> {
        unsafe {
            let init = mk_ini_default();
            mk_publish_auth_invoker_do2(self.0, CString::new(err_msg)?.as_ptr(), init)
        }

        Ok(())
    }
}

impl Clone for PublishAuthInvoker {
    fn clone(&self) -> Self {
        PublishAuthInvoker(unsafe { mk_publish_auth_invoker_clone(self.0) }, true)
    }
}

impl Drop for PublishAuthInvoker {
    fn drop(&mut self) {
        unsafe {
            if self.1 {
                mk_publish_auth_invoker_clone_release(self.0);
            }
        }
    }
}

impl From<mk_publish_auth_invoker> for PublishAuthInvoker {
    fn from(inner: mk_publish_auth_invoker) -> Self {
        Self(inner, false)
    }
}
