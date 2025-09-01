use std::sync::Arc;

use gstreamer::{glib, prelude::*};
use rszlm::{
    init::{EnvIni, EnvInitBuilder},
    media::Media,
    obj::CodecId,
    server::http_server_start,
};
use tokio_util::sync::CancellationToken;

#[tokio::main]
async fn main() {
    let cancel = CancellationToken::new();

    run_zlm(cancel.clone());
    run_test_video();

    tokio::signal::ctrl_c().await.unwrap();
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
        .build()
        .expect("Failed to create videotestsrc element");

    let enc = gstreamer::ElementFactory::make("x264enc")
        .property_from_str("tune", "zerolatency")
        .property_from_str("speed-preset", "ultrafast")
        .build()
        .expect("Failed to create x264enc element");
    let appsink = gstreamer_app::AppSink::builder().build();

    let media = Arc::new(Media::new(
        "__defaultVhost__",
        "live",
        "test",
        0.0,
        false,
        false,
    ));

    let mut init_track = false;

    let media_clone = media.clone();
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

                        if !init_track {
                            let structure = sample.caps().unwrap().structure(0).unwrap();
                            let width = structure.get::<i32>("width").unwrap();
                            let height = structure.get::<i32>("height").unwrap();
                            let fps = structure.get::<gstreamer::Fraction>("framerate").unwrap();
                            media_clone.init_video(
                                CodecId::H264,
                                width,
                                height,
                                (fps.numer() as f32) / (fps.denom() as f32),
                                0,
                            );
                            media_clone.init_complete();
                            init_track = true;
                        }
                        let map = buffer.map_readable().unwrap();
                        let data = map.as_slice();
                        let frame = rszlm::frame::Frame::new(
                            CodecId::H264,
                            dts.nseconds(),
                            pts.nseconds(),
                            data.to_vec(),
                        );
                        media_clone.input_frame(&frame);
                    })
                    .map_err(|_| gstreamer::FlowError::Eos)?;
                Ok(gstreamer::FlowSuccess::Ok)
            })
            .build(),
    );

    let appsinkel = appsink.upcast_ref();
    pipeline
        .add_many(&[&src, &enc, &appsinkel])
        .expect("Failed to add elements to pipeline");

    // 连接
    gstreamer::Element::link_many(&[&src, &enc, &appsinkel]).expect("Failed to link src, enc, pay");
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

fn run_zlm(cancel: CancellationToken) {
    let _ = std::thread::Builder::new()
        .name("zlm_serve".to_string())
        .spawn(move || {
            EnvInitBuilder::default()
                .log_level(0)
                .log_mask(0)
                .thread_num(20)
                .build();
            {
                let ini = EnvIni::global().lock().unwrap();
                ini.set_option_int("protocol.hls_demand", 1);
                ini.set_option_int("protocol.rtsp_demand", 1);
                ini.set_option_int("protocol.rtmp_demand", 1);
                ini.set_option_int("protocol.ts_demand", 1);
                ini.set_option_int("protocol.fmp4_demand", 1);

                println!("ini: {}", ini.dump());
            }

            http_server_start(8553, false);
            http_server_start(8554, false);
            http_server_start(8555, false);

            loop {
                if cancel.is_cancelled() {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        });
}
