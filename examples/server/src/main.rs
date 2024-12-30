use std::collections::HashMap;
use std::str::FromStr;

use axum::response::IntoResponse;
use axum::{body::Body, routing::get, Router};
use futures_util::StreamExt;
use hyper_util::{client::legacy::connect::HttpConnector, rt::TokioExecutor};
use once_cell::sync::{Lazy, OnceCell};
use rszlm::{
    event::EVENTS,
    init::{EnvIni, EnvInitBuilder},
    player::ProxyPlayerBuilder,
    server::{http_server_start, rtmp_server_start, rtsp_server_start, stop_all_server},
};
use tokio::{runtime::Handle, sync::RwLock};
use tokio_util::sync::CancellationToken;

static PULL_PROXY_MESSAGE: OnceCell<tokio::sync::mpsc::Sender<ProxyMessageCmd>> = OnceCell::new();

static PULL_STORE: Lazy<RwLock<HashMap<String, ProxyState>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

pub(crate) async fn pull_proxy_message(msg: ProxyMessageCmd) {
    if let Some(tx) = PULL_PROXY_MESSAGE.get() {
        let _ = tx.send(msg).await;
    }
}

const AXUM_PORT: u16 = 8552;

type Client = hyper_util::client::legacy::Client<HttpConnector, Body>;

static CLIENT: Lazy<Client> = Lazy::new(|| {
    hyper_util::client::legacy::Client::<(), ()>::builder(TokioExecutor::new())
        .build(HttpConnector::new())
});

#[tokio::main]
async fn main() {
    let cancel = CancellationToken::new();

    // start zlm
    let cancel_clone = cancel.clone();
    tokio::spawn(zlm_start(cancel_clone));

    let cancel_clone = cancel.clone();
    tokio::spawn(start(cancel_clone));

    let cancel_clone = cancel.clone();
    tokio::spawn(axum_start(AXUM_PORT, cancel_clone));

    tokio::signal::ctrl_c().await.unwrap();
    cancel.cancel();
}

async fn start(cancel: CancellationToken) -> anyhow::Result<()> {
    let (tx, mut rx) = tokio::sync::mpsc::channel::<ProxyMessageCmd>(20);
    {
        let _ = PULL_PROXY_MESSAGE.set(tx);
    }

    loop {
        tokio::select! {
            _ = cancel.cancelled() => {
                break;
            },
            Some(msg) = rx.recv() => {
                match msg {
                    ProxyMessageCmd::Start(m) => {
                        let cancel = CancellationToken::new();
                        let key = format!("{}/{}", m.app, m.stream);
                        {
                            let mut store = PULL_STORE.write().await;
                            if store.contains_key(&key) {
                                continue;
                            }

                            println!("start pull: {}", key);
                            let _ = store.insert(key, ProxyState::new(m.source, m.app, m.stream, cancel));
                        }
                    }
                    ProxyMessageCmd::Stop(m) => {
                        let mut store = PULL_STORE.write().await;
                        let key = format!("{}/{}", m.app, m.stream);
                        if let Some(state) = store.remove(&key) {
                            println!("stop pull: {}", key);
                            state.cancel.cancel();
                        }
                    }
                };
            },
        }
    }

    Ok(())
}

pub struct ProxyState {
    pub source: String,
    pub cancel: CancellationToken,
}

pub enum ProxyMessageCmd {
    Start(StartProxyMessage),
    Stop(StopProxyMessage),
}

impl ProxyState {
    pub fn new(source: String, app: String, stream: String, cancel: CancellationToken) -> Self {
        let cancel_clone = cancel.clone();
        let source_clone = source.clone();

        let s = Self {
            source: source.clone(),
            cancel,
        };

        tokio::spawn(
            async move { proxy_pull_worker(&source_clone, &app, &stream, cancel_clone).await },
        );

        s
    }
}

async fn proxy_pull_worker(source: &str, app: &str, stream: &str, cancel: CancellationToken) {
    let player = ProxyPlayerBuilder::new()
        .vhost("__defaultVhost__")
        .app(app)
        .stream(stream)
        .add_option("rtp_type", "0")
        .build();
    let poll_cancel = CancellationToken::new();
    let poll_cancel_clone = poll_cancel.clone();
    player.on_close(move |_, _, _| {
        poll_cancel_clone.cancel();
    });
    player.play(&source);

    loop {
        tokio::select! {
            _ = cancel.cancelled() => {
                break;
            },
            _ = poll_cancel.cancelled() => {
                // todo retry
                break;
            }
        }
    }
}

impl Drop for ProxyState {
    fn drop(&mut self) {
        if !self.cancel.is_cancelled() {
            self.cancel.cancel();
        }
    }
}

pub struct StartProxyMessage {
    pub source: String,
    pub app: String,
    pub stream: String,
}

pub struct StopProxyMessage {
    pub app: String,
    pub stream: String,
}

async fn zlm_start(cancel: CancellationToken) -> anyhow::Result<()> {
    let cancel_clone = cancel.clone();
    let (tx, mut rx) = tokio::sync::mpsc::channel::<ProxyMessageCmd>(100);

    let runtime = Handle::current();
    let _ = start_zlm_background(cancel_clone, tx, runtime);

    loop {
        tokio::select! {
            _ = cancel.cancelled() => {
                break;
            },
            Some(msg) = rx.recv() => {
                pull_proxy_message(msg).await;
            }
        }
    }

    Ok(())
}

fn start_zlm_background(
    cancel: CancellationToken,
    tx: tokio::sync::mpsc::Sender<ProxyMessageCmd>,
    runtime: tokio::runtime::Handle,
) -> anyhow::Result<()> {
    tokio::task::spawn_blocking(move || {
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
        rtsp_server_start(8554, false);
        rtmp_server_start(8555, false);
        {
            let mut events = EVENTS.write().unwrap();
            let tx_clone = tx.clone();
            events.on_media_not_found(move |msg| {
                let app = msg.url_info.app();
                let stream = msg.url_info.stream();

                if app == "live" && stream == "test" {
                    let _ = tx_clone.blocking_send(ProxyMessageCmd::Start(StartProxyMessage {
                        source: "mp4".to_string(),
                        app,
                        stream,
                    }));
                } else {
                    let _ = tx_clone.blocking_send(ProxyMessageCmd::Start(StartProxyMessage {
                        source: "rtsp://192.168.0.14:554/t01/1".to_string(),
                        app,
                        stream,
                    }));
                }

                false
            });

            let tx_clone = tx.clone();
            events.on_media_no_reader(move |msg| {
                println!(
                    "MediaNoReader: app: {}, stream: {}",
                    msg.sender.app(),
                    msg.sender.stream()
                );
                let _ = tx_clone.blocking_send(ProxyMessageCmd::Stop(StopProxyMessage {
                    app: msg.sender.app(),
                    stream: msg.sender.stream(),
                }));
            });

            events.on_http_request(move |msg| {
                let url = msg.parser.url();

                if url.starts_with("/test") {
                    let headers = vec!["Content-Type".to_string(), "text/plain".to_string()];
                    let body = "hello world";
                    msg.invoker.invoke(200, headers, body);
                    return true;
                } else if url.starts_with("/proxy") {
                    let path = &url["/proxy".len()..];
                    let query = msg.parser.query_str();
                    let path_query = if query.is_empty() {
                        path.to_owned()
                    } else {
                        format!("{}?{}", path, query)
                    };

                    let uri = format!("http://127.0.0.1:{}{}", AXUM_PORT, path_query);
                    let headers = msg.parser.headers();
                    println!("uri: {}", uri);
                    println!("headers: {:?}", headers);
                    if let Ok(mut req) = hyper::Request::builder()
                        .method(msg.parser.method().as_str())
                        .uri(uri)
                        .body(Body::from(msg.parser.body()))
                    {
                        if !headers.is_empty() {
                            for (k, v) in headers {
                                req.headers_mut().insert(
                                    hyper::http::HeaderName::from_str(&k).unwrap(),
                                    hyper::http::HeaderValue::from_str(&v).unwrap(),
                                );
                            }
                        }

                        let resp = runtime.block_on(async move {
                            CLIENT
                                .request(req)
                                .await
                                .map_err(|_| hyper::StatusCode::BAD_REQUEST)
                                .into_response()
                        });
                        let status = resp.status();
                        let header = resp
                            .headers()
                            .iter()
                            .map(|(k, v)| vec![k.to_string(), v.to_str().unwrap().to_string()])
                            .flatten()
                            .collect::<Vec<_>>();
                        let body = resp.into_body();
                        let body_str = runtime
                            .block_on(async move {
                                let mut b = body.into_data_stream();
                                let mut data = String::new();
                                while let Some(chunk) = b.next().await {
                                    data.push_str(&String::from_utf8_lossy(&chunk?));
                                }
                                anyhow::Ok(data)
                            })
                            .unwrap();

                        msg.invoker
                            .invoke(status.as_u16() as i32, header, &body_str);
                        return true;
                    }
                }
                false
            });
        }

        loop {
            if cancel.is_cancelled() {
                break;
            }

            std::thread::sleep(std::time::Duration::from_millis(1000));
        }

        stop_all_server();
        println!("zlm server stopped");
    });

    Ok(())
}

async fn axum_start(port: u16, cancel: CancellationToken) -> anyhow::Result<()> {
    let app = Router::new()
        .route("/", get(|| async { "Hello, axum world!" }))
        .route("/test", get(|| async { "Hello, axum test!" }));

    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
    println!("listening on {}", listener.local_addr()?);
    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            tokio::select! {
                _ = cancel.cancelled() => {
                    println!("cancel");
                },
            }
        })
        .await
        .map_err(|e| e.into())
}
