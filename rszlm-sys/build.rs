use std::{env, io, path::PathBuf, process::Command};

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

    cmake
        .uses_cxx11()
        .profile("Release")
        .out_dir(&src_install_path());

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

fn buildgen() {
    let is_static = is_static();

    let (zlm_install_include, zlm_install_lib) = if env::var("ZLM_DIR").is_ok() {
        let zlm_install = PathBuf::from(env::var("ZLM_DIR").unwrap());
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

        if std::fs::metadata(&src_install_path().join("lib").join("libzlmediakit.a")).is_err() {
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
        println!("find libsrtp2 from pkg_config");
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
