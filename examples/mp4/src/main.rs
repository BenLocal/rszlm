use rszlm::{
    event::EVENTS,
    init::EnvInitBuilder,
    player::Mp4ProxyPlayer,
    server::{rtsp_server_start, stop_all_server},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    EnvInitBuilder::default()
        .log_level(0)
        .log_mask(0)
        .thread_num(20)
        .build();

    rtsp_server_start(8554, false);

    EVENTS.write().unwrap().on_media_play(move |msg| {
        println!("media play: {}", msg.url_info.stream());
        println!("start player");
        Mp4ProxyPlayer::new(
            &msg.url_info.vhost(),
            &msg.url_info.app(),
            &msg.url_info.stream(),
            "/test1.mp4",
            0,
            None,
        );
        Ok(())
    });

    tokio::signal::ctrl_c().await?;
    stop_all_server();
    Ok(())
}
