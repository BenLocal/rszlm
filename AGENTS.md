# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

`rszlm` is a Rust binding for [ZLMediaKit](https://github.com/ZLMediaKit/ZLMediaKit) (a C/C++ streaming media server). It is a Cargo workspace: the `rszlm` crate (root) is the safe high-level API, `rszlm-sys` is the raw FFI/bindgen layer, and `examples/*` are runnable demos.

## Build & run

```bash
./build_zlm.sh                 # clone + cmake-build ZLMediaKit into ./ZLMediaKit/build/installer (with WEBRTC)
cargo build                    # build the rszlm lib (uses .cargo/config.toml -> ZLM_DIR)
cargo build -p rszlm-sys       # build just the FFI crate (regenerates bindings)
cargo run -p server            # run an example (also: mp4, webrtc, gst)
```

- **bindgen requires `Clang`/libclang** to be installed (it parses `mk_mediakit.h`).
- There is **no test suite** (`doctest = false` on both crates, no `#[test]`/`#[cfg(test)]`). Verification = building the crate and the examples.
- `.cargo/config.toml` sets `ZLM_DIR` to the local install and prepends `${ZLM_DIR}/bin` to `PATH`. For dynamic linking the runtime must find `libmk_api.so` (it's in `ZLM_DIR/bin`).

## How `rszlm-sys/build.rs` acquires ZLMediaKit (the central build concept)

The build script picks a ZLMediaKit source in strict priority order — read it before touching build/link behavior:

1. **`ZLM_DIR` set** → use that prebuilt install directly (`include/`, `lib/`, `bin/`). Highest priority; skips everything else. This is the committed default (`.cargo/config.toml`).
2. **dynamic build, `ZLM_DIR` unset** → download a prebuilt release archive from `ZLM_RELEASE_URL`, extract to `OUT_DIR/zlm-install`. Pure-Rust (`ureq`/`flate2`/`tar`/`zip`) — no `curl`/`tar`/`unzip` subprocess.
3. **static build, or `ZLM_BUILD_FROM_SOURCE=1`** → `git clone` + cmake-build ZLMediaKit from source.

**Prebuilt download is dynamic-only**: the release archive exposes the C API only via `libmk_api.so` (the `mk_*` symbols are not in any bundled `.a`), so `static` builds can never use it — they fall through to a source build. With `webrtc`, the source path also builds libsrtp from source.

### Build-time environment variables

| Var | Effect | Default |
|---|---|---|
| `ZLM_DIR` | Use a prebuilt install dir (supports `${VAR}` expansion). Wins over everything. | set in `.cargo/config.toml` |
| `ZLM_RELEASE_URL` | Base URL (asset name appended) or full `.tar.gz`/`.zip` URL of the prebuilt release. | GitHub `releases/latest/download` (pinned to a tag in config.toml) |
| `ZLM_BRANCH` | Branch variant in the asset name `zlmediakit_<branch>_<os>_<arch>_latest`. | `master` |
| `ZLM_BUILD_FROM_SOURCE` | `1`/`true`/`ON` forces source build even for dynamic. | unset |
| `ZLM_GIT` / `ZLM_GIT_ZONE` | Source-build repo URL / `gitee` mirror. | ZLMediaKit GitHub |

## Cargo features

- `static` (vs default dynamic) and `webrtc`. Defined on both `rszlm` and `rszlm-sys`; the `rszlm` features just re-export `rszlm-sys/{static,webrtc}`.
- **Gotcha:** the `mp4`/`server`/`webrtc` examples declare `rszlm` with `features = ["static"]` / `["static","webrtc"]`. So `cargo check --workspace --all-targets` (e.g. rust-analyzer) unifies **static+webrtc** onto `rszlm-sys`, which forces a from-source ZLMediaKit + libsrtp build unless `ZLM_DIR` points at a real local install. Keep `ZLM_DIR` valid (run `./build_zlm.sh`) to keep workspace checks fast/green.
- The `gst` example additionally needs system GStreamer (`gstreamer-app-1.0.pc`, i.e. `libgstreamer-plugins-base1.0-dev`).

## Code architecture

**`rszlm-sys`** is `include!(concat!(env!("OUT_DIR"), "/bindings.rs"))` — nothing but bindgen output (with non-standard-naming lints allowed). All real logic is `build.rs`.

**`rszlm`** wraps the raw `mk_*` C API into safe types. The pattern is consistent across `init`/`media`/`player`/`pusher`/`recorder`/`server`/`event`/`frame`/`obj`/`webrtc`:

- Each opaque C handle (`mk_*`) is wrapped in a newtype struct; `Drop` calls the matching `mk_*_release`. Config objects use a builder (`EnvInitBuilder`, `ProxyPlayerBuilder`, …).
- **C callback → Rust closure bridge** (the key unsafe idiom): a `Box<dyn FnMut/Fn>` is leaked to a raw `*mut c_void` via the `box_to_mut_void_ptr!` macro and passed as the C `user_data`; an `extern "C"` trampoline recovers it with `std::mem::transmute` and invokes it. See `server.rs::on_rtp_server_*` for the canonical example.
- String marshalling goes through the `lib.rs` macros `const_str_to_ptr!` (Rust→`CString`) and `const_ptr_to_string!` (C `char*`→`String`).
- **Entry point:** `EnvInitBuilder::…::build()` calls `mk_env_init` and must run before any server/media use. `init::EnvIni::global()` holds the global ZLM ini config.
- **Events:** `event::EVENTS` is the global hub — register `on_media_*` / `on_http_*` / `on_record_*` / `on_rtsp_*` callbacks on it, then `listen()` (wraps `mk_events_listen`).
- `DEFAULT_VHOST = "__defaultVhost__"` is ZLMediaKit's default vhost; reuse the const rather than the literal.
