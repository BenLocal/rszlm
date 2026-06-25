use std::{env, io, io::Read, path::PathBuf, process::Command};

fn main() {
    std::thread::Builder::new()
        .name("rszlm-sys-build".into())
        .spawn(buildgen)
        .unwrap()
        .join()
        .unwrap();
}

fn out_dir() -> PathBuf {
    PathBuf::from(env::var("OUT_DIR").unwrap())
}

fn src_path() -> PathBuf {
    out_dir().join("ZLMediaKit")
}

fn src_install_path() -> PathBuf {
    out_dir().join("zlm-install")
}

fn is_static() -> bool {
    cfg!(feature = "static")
}

fn zlm_release_path() -> PathBuf {
    let path = PathBuf::from("release");
    match env::var("CARGO_CFG_TARGET_OS").unwrap().as_str() {
        "windows" => path.join("windows").join("Release").join("Release"),
        "macos" => path.join("darwin").join("Release"),
        "linux" => path.join("linux").join("Release"),
        _a => unimplemented!("Unsupported target_os: {}", _a),
    }
}

fn git_src() -> String {
    env::var("ZLM_GIT").unwrap_or_else(|_| {
        if env::var("ZLM_GIT_ZONE") == Ok("gitee".to_string()) {
            "https://gitee.com/xia-chu/ZLMediaKit".to_string()
        } else {
            "https://github.com/ZLMediaKit/ZLMediaKit".to_string()
        }
    })
}

/// Default base URL of the prebuilt ZLMediaKit release archives.
///
/// Uses GitHub's `/releases/latest/download/<asset>` redirect, which always
/// resolves to the newest (non-prerelease) release, so it never goes stale.
const DEFAULT_ZLM_RELEASE_URL: &str =
    "https://github.com/BenLocal/ZLMediaKit-Build/releases/latest/download";

/// Whether to acquire ZLMediaKit from a prebuilt release archive instead of
/// compiling it from source.
///
/// The release archives expose the C API only through the dynamic
/// `libmk_api.so` (the `mk_*` symbols live there, not in any bundled `.a`), so
/// prebuilt download is only applicable to non-static (dynamic) builds; static
/// builds fall through to a source build. Set `ZLM_BUILD_FROM_SOURCE=1` to
/// force a source build for the dynamic case too.
fn use_prebuilt() -> bool {
    if is_static() {
        return false;
    }
    !matches!(
        env::var("ZLM_BUILD_FROM_SOURCE").as_deref(),
        Ok("1") | Ok("true") | Ok("ON")
    )
}

/// Platform-specific asset file name inside a release, e.g.
/// `zlmediakit_master_linux_amd64_latest.tar.gz`.
///
/// The default branch follows the active features: `webrtc` builds need the
/// `feature_transcode2` package (it ships a `libmk_api` compiled with WebRTC),
/// everything else uses `master`. `ZLM_BRANCH` overrides this.
///
/// Note: as of the current releases, `feature_transcode2` only ships
/// linux/{amd64,arm64} — webrtc dynamic builds on macOS/Windows have no prebuilt
/// and must set `ZLM_BUILD_FROM_SOURCE=1` (or point `ZLM_DIR` at a local build).
fn prebuilt_asset_name() -> String {
    let default_branch = if cfg!(feature = "webrtc") {
        "feature_transcode2"
    } else {
        "master"
    };
    let branch = env::var("ZLM_BRANCH").unwrap_or_else(|_| default_branch.to_string());
    let os = match env::var("CARGO_CFG_TARGET_OS").unwrap().as_str() {
        "windows" => "windows",
        "macos" => "macos",
        "linux" => "linux",
        other => unimplemented!("Unsupported target_os for prebuilt download: {}", other),
    };
    let arch = match env::var("CARGO_CFG_TARGET_ARCH").unwrap().as_str() {
        "x86_64" => "amd64",
        "aarch64" => "arm64",
        other => unimplemented!("Unsupported target_arch for prebuilt download: {}", other),
    };
    let ext = if os == "windows" { "zip" } else { "tar.gz" };
    format!("zlmediakit_{branch}_{os}_{arch}_latest.{ext}")
}

/// Full URL of the prebuilt archive to download.
///
/// Controlled by the `ZLM_RELEASE_URL` environment variable, which holds the
/// release download address. It may be either:
///   * a base URL (the default), to which the platform asset name is appended, or
///   * a full archive URL ending in `.tar.gz` / `.zip`, which is used verbatim.
fn prebuilt_url() -> String {
    let base = expand_env_vars(
        &env::var("ZLM_RELEASE_URL").unwrap_or_else(|_| DEFAULT_ZLM_RELEASE_URL.to_string()),
    );
    if base.ends_with(".tar.gz") || base.ends_with(".zip") {
        base
    } else {
        format!("{}/{}", base.trim_end_matches('/'), prebuilt_asset_name())
    }
}

/// Download and extract a prebuilt ZLMediaKit release into `src_install_path()`,
/// producing the same `include/` + `lib/` layout the rest of the build expects.
///
/// Implemented entirely with Rust crates (`ureq` + `flate2`/`tar` + `zip`) so the
/// build does not depend on external `curl`/`tar`/`unzip` executables.
fn download_prebuilt() -> io::Result<()> {
    let url = prebuilt_url();
    let install = src_install_path();
    std::fs::create_dir_all(&install)?;

    let is_zip = url.ends_with(".zip");

    println!("cargo:warning=downloading prebuilt ZLMediaKit from {url}");

    let resp = ureq::get(&url).call().map_err(|e| {
        io::Error::new(
            io::ErrorKind::Other,
            format!("failed to download prebuilt ZLMediaKit from {url}: {e}"),
        )
    })?;
    // owned body reader: streams without ureq's default 10 MB read cap
    let (_, body) = resp.into_parts();
    let mut reader = body.into_reader();

    // archive layout is `./include/...` and `./lib/...` at the top level
    if is_zip {
        // zip needs Read + Seek, so buffer the archive in memory first
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        let mut zip = zip::ZipArchive::new(io::Cursor::new(buf)).map_err(|e| {
            io::Error::new(io::ErrorKind::Other, format!("invalid zip archive: {e}"))
        })?;
        zip.extract(&install).map_err(|e| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("failed to extract prebuilt zip archive: {e}"),
            )
        })?;
    } else {
        // stream gzip -> tar straight to disk, no intermediate buffer
        let gz = flate2::read::GzDecoder::new(reader);
        tar::Archive::new(gz).unpack(&install)?;
    }

    Ok(())
}

fn build() -> io::Result<()> {
    let src = src_path();
    // download from github or gitee
    if !&src.parent().unwrap().exists() {
        std::fs::create_dir_all(&src.parent().unwrap())?;
    }

    if !&src.exists() {
        Command::new("git")
            .arg("clone")
            .arg("--depth")
            .arg("1")
            .arg(git_src())
            .current_dir(&src.parent().unwrap())
            .status()?;

        Command::new("git")
            .arg("submodule")
            .args(vec!["update", "--init"])
            .current_dir(src)
            .status()?;
    }

    let mut cmake = cmake::Config::new(&src_path());

    // Add parallel build configuration
    #[cfg(target_os = "windows")]
    cmake.define("CMAKE_BUILD_PARALLEL_LEVEL", num_cpus::get().to_string());
    #[cfg(not(target_os = "windows"))]
    cmake.build_arg(format!("-j{}", num_cpus::get()));

    cmake.profile("Release").out_dir(&src_install_path());

    if is_static() {
        cmake.define("ENABLE_API_STATIC_LIB", "ON");
        cmake.define("OPENSSL_USE_STATIC_LIBS", "ON");
    }

    #[cfg(feature = "webrtc")]
    cmake.define("ENABLE_WEBRTC", "ON");
    #[cfg(not(feature = "webrtc"))]
    cmake.define("ENABLE_WEBRTC", "OFF");

    cmake.register_dep("OPENSSL");

    let dst = cmake.build();
    println!("cargo:root={}", dst.to_string_lossy());

    if is_static() {
        // when static build, zlm only install mk_api static lib
        // so we copy all static libs to zlm-install/lib
        std::fs::read_dir(src_path().join(zlm_release_path()))?
            .into_iter()
            .for_each(|e| {
                if let Ok(entry) = e {
                    if entry.file_type().unwrap().is_file() {
                        let file_name = entry.file_name();
                        let file_name = file_name.to_str().unwrap();
                        if file_name.ends_with(".a")
                            || file_name.ends_with(".lib")
                            || file_name.ends_with(".so")
                            || file_name.ends_with(".dylib")
                            || file_name.ends_with(".dll")
                        {
                            std::fs::copy(
                                &src_path().join(zlm_release_path()).join(file_name),
                                &src_install_path().join("lib").join(file_name),
                            )
                            .unwrap();
                        }
                    }
                }
            });
    }

    Ok(())
}

fn link_dynamic(_zlm_link_path: &PathBuf) {
    println!("cargo:rustc-link-lib=dylib=mk_api");
}

fn link_static(zlm_link_path: &PathBuf) {
    //println!("cargo:rustc-link-lib=mk_api_static");
    select_static_libs(zlm_link_path)
        .unwrap()
        .iter()
        .for_each(|lib| {
            println!("cargo:rustc-link-lib=static={}", lib);
        });
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    if target_os == "macos" {
        println!("cargo:rustc-link-lib=ssl");
        println!("cargo:rustc-link-lib=crypto");
        println!("cargo:rustc-link-lib=c++");
        println!("cargo:rustc-link-lib=framework=Foundation");
    } else if target_os == "linux" || target_os == "android" {
        println!("cargo:rustc-link-lib=ssl");
        println!("cargo:rustc-link-lib=crypto");
        println!("cargo:rustc-link-lib=stdc++");
    }
}

fn select_static_libs(zlm_link_path: &PathBuf) -> io::Result<Vec<String>> {
    Ok(std::fs::read_dir(zlm_link_path)?
        .into_iter()
        .filter_map(|entry| {
            let entry = entry.ok()?;
            if entry.file_type().unwrap().is_file() {
                let file_name = entry.file_name();
                let file_name = file_name.to_str().unwrap();
                let extension = match env::var("CARGO_CFG_TARGET_OS").unwrap().as_str() {
                    "windows" => Some(".lib"),
                    "linux" | "macos" | "ios" | "android" => Some(".a"),
                    _ => None,
                };
                if let Some(ext) = extension {
                    if file_name.ends_with(ext) {
                        let mut link_name = file_name.trim_end_matches(ext);
                        if ext == ".a" {
                            link_name = link_name.trim_start_matches("lib");
                        }
                        return Some(link_name.to_string());
                    }
                }
                None
            } else {
                None
            }
        })
        .collect::<Vec<_>>())
}

fn expand_env_vars(value: &str) -> String {
    let re = regex::Regex::new(r"\$\{([A-Za-z0-9_]+)\}").unwrap();
    let mut result = value.to_string();

    while let Some(caps) = re.captures(&result) {
        if let Some(matched) = caps.get(0) {
            let var_name = &caps[1];
            if let Ok(var_value) = env::var(var_name) {
                result = result.replace(matched.as_str(), &var_value);
            }
        }
    }

    result
}

fn buildgen() {
    // re-run the build script when any of the controlling env vars change
    for var in [
        "ZLM_DIR",
        "ZLM_RELEASE_URL",
        "ZLM_BRANCH",
        "ZLM_BUILD_FROM_SOURCE",
        "ZLM_GIT",
        "ZLM_GIT_ZONE",
    ] {
        println!("cargo:rerun-if-env-changed={var}");
    }

    let is_static = is_static();

    let (zlm_install_include, zlm_install_lib) = if env::var("ZLM_DIR").is_ok() {
        let zlm_install = PathBuf::from(expand_env_vars(&env::var("ZLM_DIR").unwrap()));
        println!(
            "cargo:rustc-link-search=native={}",
            &zlm_install.join("lib").to_string_lossy()
        );
        (zlm_install.join("include"), zlm_install.join("lib"))
    } else {
        find_libsrtp2();

        println!(
            "cargo:rustc-link-search=native={}",
            src_install_path().join("lib").to_string_lossy()
        );

        if use_prebuilt() {
            // download prebuilt binaries from the release unless already extracted
            if std::fs::metadata(&src_install_path().join("include").join("mk_mediakit.h"))
                .is_err()
            {
                download_prebuilt().unwrap();
            }
        } else if std::fs::metadata(&src_install_path().join("lib").join("libzlmediakit.a"))
            .is_err()
        {
            build().unwrap();
        }

        (
            src_install_path().join("include"),
            src_install_path().join("lib"),
        )
    };

    // link lib
    if is_static {
        link_static(&zlm_install_lib);
    } else {
        link_dynamic(&zlm_install_lib);
    }

    // generate bindings
    let bindings = bindgen::Builder::default()
        .header(zlm_install_include.join("mk_mediakit.h").to_string_lossy())
        .derive_default(true)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(out_dir().join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

#[cfg(not(feature = "webrtc"))]
fn find_libsrtp2() {
    // nothing
}

#[cfg(feature = "webrtc")]
fn find_libsrtp2() {
    if let Ok(_lib) = pkg_config::Config::new()
        .atleast_version("2.3.0")
        .statik(is_static())
        .probe("libsrtp2")
    {
        println!(
            "find libsrtp2 from pkg_config, if you want to use webrtc, please install libsrtp2"
        );
    } else {
        build_srtp();
        // check srtp build result
        // if srtp.is_err() {
        //     panic!("can't find libsrtp2, please install libsrtp2 or disable feature `webrtc`");
        // }
    }
}

#[allow(dead_code)]
fn build_srtp() {
    let git_url = if env::var("ZLM_GIT_ZONE") == Ok("gitee".to_string()) {
        "https://gitee.com/mirrors/libsrtp.git".to_string()
    } else {
        "https://github.com/cisco/libsrtp.git".to_string()
    };

    // download from github
    if !&out_dir().join("libsrtp").exists() {
        Command::new("git")
            .arg("clone")
            .arg("-b")
            .arg("v2.3.0")
            .arg(git_url)
            .current_dir(&out_dir())
            .status()
            .unwrap();
    }

    println!("env: {:?}", env::vars());

    // build srtp
    let mut cmake = cmake::Config::new(&out_dir().join("libsrtp"));
    cmake
        .profile("Release")
        //.define("ENABLE_OPENSSL", "ON")
        // libsrtp 2.3.0 declares a tentative `bit_string` global in a shared
        // header; under GCC 10+ (default -fno-common) that collides at link
        // time ("multiple definition of `bit_string`"). Restore -fcommon.
        .cflag("-fcommon")
        .out_dir(&out_dir().join("srtp-install"))
        .register_dep("OPENSSL");

    cmake.build();
    println!(
        "cargo:rustc-link-search={}",
        &out_dir().join("srtp-install").join("lib").to_string_lossy()
    );

    if is_static() {
        println!("cargo:rustc-link-lib=static=srtp2");
    } else {
        println!("cargo:rustc-link-lib=srtp2");
    }
}
