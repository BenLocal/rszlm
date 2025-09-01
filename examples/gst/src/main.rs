use std::{
    fs::OpenOptions,
    io::Write as _,
    sync::{Arc, Mutex},
};

use gstreamer::prelude::*;
use rszlm::{
    frame::H264Splitter,
    init::{EnvIni, EnvInitBuilder},
    media::Media,
    obj::CodecId,
    server::{http_server_start, rtmp_server_start, rtsp_server_start},
};
use tokio_util::sync::CancellationToken;

#[tokio::main]
async fn main() {
    let cancel = CancellationToken::new();

    run_zlm(cancel.clone());
    run_test_video();

    tokio::signal::ctrl_c().await.unwrap();
    cancel.cancel();
    println!("Shutting down...");
    std::thread::sleep(std::time::Duration::from_secs(1));
}

fn run_test_video() {
    gstreamer::init().expect("Failed to initialize GStreamer");
    println!(
        "GStreamer initialized successfully!, version: {}",
        gstreamer::version_string()
    );

    let pipeline = gstreamer::Pipeline::new();

    let src = gstreamer::ElementFactory::make("videotestsrc")
        .property_from_str("pattern", "ball")
        .property_from_str("is-live", "true")
        //    .property("key-int-max", 30) // 每30帧一个关键帧
        // .property("bframes", 0) // 禁用B帧，简化时间戳处理
        .build()
        .expect("Failed to create videotestsrc element");

    let enc = gstreamer::ElementFactory::make("x264enc")
        .property_from_str("tune", "zerolatency")
        .property_from_str("speed-preset", "ultrafast")
        .property("byte-stream", true)
        .build()
        .expect("Failed to create x264enc element");
    let parse = gstreamer::ElementFactory::make("h264parse")
        .property("config-interval", -1i32) // 每个IDR前插入SPS/PPS
        .build()
        .expect("Failed to create h264parse element");
    let capsfilter = gstreamer::ElementFactory::make("capsfilter")
        .property(
            "caps",
            gstreamer::Caps::builder("video/x-h264")
                .field("stream-format", "byte-stream") // 关键：Annex-B格式
                .field("alignment", "au") // 访问单元对齐
                .build(),
        )
        .build()
        .expect("Failed to create capsfilter");
    let appsink = gstreamer_app::AppSink::builder()
        .caps(
            &gstreamer::Caps::builder("video/x-h264")
                .field("stream-format", "byte-stream")
                .field("alignment", "au")
                .build(),
        )
        .build();

    let media = Arc::new(Media::new(
        "__defaultVhost__",
        "live",
        "test",
        0.0,
        false,
        false,
    ));
    media.init_track(&rszlm::obj::Track::new(CodecId::H264, None));
    media.init_complete();
    println!("Media created");
    let media_clone = media.clone();

    let mut dts = 0;
    let sp = Arc::new(H264Splitter::new(
        Box::new(move |data: &[u8]| {
            let frame = rszlm::frame::Frame::new(CodecId::H264, dts, dts, data);
            if !media_clone.input_frame(&frame) {
                eprintln!("Failed to input frame from splitter, size={}", data.len());
            } else {
                // println!("Frame input from splitter, size={}", data.len());
            }

            dts += 40;
        }),
        false,
    ));

    std::fs::remove_file("output.h264").ok();
    use std::sync::atomic::{AtomicU64, Ordering};
    let start_time = Arc::new(AtomicU64::new(0));
    let start_time_clone = start_time.clone();
    let sp_clone = sp.clone();
    // let mut file_clone = h264_file.clone();
    appsink.set_callbacks(
        gstreamer_app::AppSinkCallbacks::builder()
            .new_sample(move |appsink| {
                // println!("New sample received in appsink");
                appsink
                    .pull_sample()
                    .map(|sample| {
                        // Process the sample (e.g., encode and push to RTSP server)
                        let buffer = sample.buffer().unwrap();
                        let pts = buffer.pts().unwrap();
                        let dts = buffer.dts().unwrap_or(pts);
                        let duration = buffer.duration().unwrap_or(gstreamer::ClockTime::ZERO);

                        let map = buffer.map_readable().unwrap();
                        let data = map.as_slice();
                        if data.len() < 4 {
                            return; // 数据太小，忽略
                        }

                        let mut file = std::fs::File::options()
                            .create(true)
                            .append(true)
                            .open("output.h264")
                            .unwrap();
                        file.write_all(data).unwrap();

                        let start_ns = start_time_clone.load(Ordering::Relaxed);
                        if start_ns == 0 {
                            start_time_clone.store(dts.nseconds(), Ordering::Relaxed);
                        }
                        let start_offset = start_time_clone.load(Ordering::Relaxed);

                        let dts_ms = (dts.nseconds().saturating_sub(start_offset)) / 1_000_000;
                        let pts_ms = (pts.nseconds().saturating_sub(start_offset)) / 1_000_000;
                        let duration_ms = duration.nseconds() / 1_000_000;
                        sp_clone.input(data);

                        // let frame =
                        //     rszlm::frame::Frame::new(CodecId::H264, dts_ms, pts_ms, data);
                        // if !media_clone.input_frame(&frame) {
                        //     eprintln!(
                        //         "Failed to input frame: pts={} dts={} duration={} size={} valid_h264={}",
                        //         dts_ms,
                        //         pts_ms,
                        //         duration_ms,
                        //         data.len(),
                        //         validate_h264_stream(data)
                        //     );
                        // } else {
                        //     // println!(
                        //     //     "Frame input: pts={} dts={} duration={} size={} valid_h264={}",
                        //     //     dts_ms,
                        //     //     pts_ms,
                        //     //     duration_ms,
                        //     //     data.len(),
                        //     //     validate_h264_stream(data)
                        //     // );
                        // }
                    })
                    .map_err(|_| gstreamer::FlowError::Eos)?;
                Ok(gstreamer::FlowSuccess::Ok)
            })
            .build(),
    );

    let appsinkel = appsink.upcast_ref();
    pipeline
        .add_many(&[&src, &enc, &parse, &capsfilter, &appsinkel])
        .expect("Failed to add elements to pipeline");

    // 连接
    gstreamer::Element::link_many(&[&src, &enc, &parse, &capsfilter, &appsinkel])
        .expect("Failed to link src, enc, pay");
    pipeline
        .set_state(gstreamer::State::Playing)
        .expect("Unable to set the pipeline to the `Playing` state");

    let pipeline_clone = pipeline.clone();
    let _ = std::thread::Builder::new()
        .name("pipeline_bus".to_string())
        .spawn(move || {
            loop {
                std::thread::sleep(std::time::Duration::from_secs(5));
                // decode_valve.set_property("drop", &true); // 阻断 decodebin 分支
                // decode_valve.sync_state_with_parent().unwrap();

                let (_, state, pending) =
                    pipeline_clone.state(gstreamer::ClockTime::from_seconds(0));
                println!("🔧 Pipeline state: {:?} (pending: {:?})", state, pending);

                // if state == gstreamer::State::Paused {}

                for element in pipeline_clone.iterate_elements() {
                    let elem = element.unwrap();
                    let name = elem.name();
                    let (res, state, pending) = elem.state(gstreamer::ClockTime::from_seconds(0));
                    println!(
                        "Element: {:<30} | State: {:?} | Pending: {:?} | Result: {:?}",
                        name, state, pending, res
                    );
                }
            }
        });
}

fn validate_h264_stream(data: &[u8]) -> bool {
    if data.len() < 4 {
        return false;
    }

    // 检查是否以start code开头 (0x00 0x00 0x00 0x01 或 0x00 0x00 0x01)
    (data[0] == 0x00 && data[1] == 0x00 && data[2] == 0x00 && data[3] == 0x01)
        || (data[0] == 0x00 && data[1] == 0x00 && data[2] == 0x01)
}

fn run_zlm(cancel: CancellationToken) {
    let _ = std::thread::Builder::new()
        .name("zlm_serve".to_string())
        .spawn(move || {
            EnvInitBuilder::default()
                .log_level(0)
                .log_mask(0)
                .thread_num(20)
                .build();

            http_server_start(8553, false);
            rtsp_server_start(8554, false);
            rtmp_server_start(8555, false);

            loop {
                if cancel.is_cancelled() {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        });
}
