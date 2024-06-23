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
    out_dir().join("install")
}

fn is_static() -> bool {
    cfg!(feature = "static")
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

fn zlm_release_path() -> PathBuf {
    let path = PathBuf::from("release");
    match env::var("CARGO_CFG_TARGET_OS").unwrap().as_str() {
        "windows" => path.join("windows").join("Release").join("Release"),
        "macos" => path.join("darwin").join("Release"),
        "linux" => path.join("linux").join("Release"),
        _a => unimplemented!("Unsupported target_os: {}", _a),
    }
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

    // build ZLMediaKit
    let build_dir = &src_path().join("build");
    if build_dir.exists() {
        std::fs::remove_dir_all(&build_dir)?;
    }
    std::fs::create_dir_all(&build_dir)?;

    // cmake
    let mut cmd = Command::new("cmake");
    cmd.arg("-DCMAKE_BUILD_TYPE=Release");

    if is_static() {
        cmd.arg("-DENABLE_API_STATIC_LIB=ON");
    }

    #[cfg(feature = "webrtc")]
    cmd.arg("-DENABLE_WEBRTC=ON");
    #[cfg(not(feature = "webrtc"))]
    cmd.arg("-DENABLE_WEBRTC=OFF");

    cmd.arg("..").current_dir(&build_dir).status()?;

    // make build
    match env::var("CARGO_CFG_TARGET_OS").unwrap().as_str() {
        "windows" => {
            Command::new("cmake")
                .arg("--build")
                .arg(".")
                .args(&["--config", "Release"])
                .current_dir(&build_dir)
                .status()?;
        }
        _ => {
            Command::new("make")
                .arg("-j8")
                .current_dir(&build_dir)
                .status()?;
        }
    }

    // copy to install dir
    let zlm_install_lib = src_install_path().join("lib");
    if zlm_install_lib.exists() {
        std::fs::remove_dir_all(&zlm_install_lib)?;
    }

    std::fs::create_dir_all(&zlm_install_lib)?;
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
                            &zlm_install_lib.join(file_name),
                        )
                        .unwrap();
                    }
                }
            }
        });

    let zlm_install_include = src_install_path().join("include");
    if zlm_install_include.exists() {
        std::fs::remove_dir_all(&zlm_install_include)?;
    }
    std::fs::create_dir_all(&zlm_install_include)?;
    let include_path = src_path().join("api").join("include");
    std::fs::read_dir(&include_path)?.into_iter().for_each(|e| {
        if let Ok(entry) = e {
            if entry.file_type().unwrap().is_file() {
                let file_name = entry.file_name();
                let file_name = file_name.to_str().unwrap();
                if file_name.ends_with(".h") {
                    std::fs::copy(
                        include_path.join(file_name),
                        zlm_install_include.join(file_name),
                    )
                    .unwrap();
                }
            }
        }
    });
    Ok(())
}

fn link_dynamic() {
    println!("cargo:rustc-link-lib=mk_api");
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

    find_libsrtp2();

    let (zlm_install_include, zlm_install_lib) = if env::var("ZLM_DIR").is_ok() {
        let zlm_install = PathBuf::from(env::var("ZLM_DIR").unwrap());
        println!(
            "cargo:rustc-link-search=native={}",
            &zlm_install.join("lib").to_string_lossy()
        );
        (zlm_install.join("include"), zlm_install.join("lib"))
    } else {
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
        link_dynamic();
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
        if build_srtp().is_err() {
            panic!("can't find libsrtp2, please install libsrtp2 or disable feature `webrtc`");
        }
    }
}

fn build_srtp() -> io::Result<()> {
    // download from github
    if !&out_dir().join("libsrtp").exists() {
        Command::new("git")
            .arg("clone")
            .arg("--depth")
            .arg("1")
            .arg("-b")
            .arg("v2.3.0")
            .arg("https://github.com/cisco/libsrtp.git")
            .current_dir(&out_dir())
            .status()?;
    }

    // build srtp
    let mut configure = Command::new("./configure");
    configure.current_dir(&out_dir().join("libsrtp"));
    configure.arg("--enable-openssl");
    configure.status()?;

    Command::new("make")
        .arg("-j8")
        .current_dir(&out_dir().join("libsrtp"))
        .status()?;

    println!("cargo:rustc-link-search={}", &out_dir().to_string_lossy());

    if is_static() {
        println!("cargo:rustc-link-lib=static=srtp2");
    } else {
        println!("cargo:rustc-link-lib=srtp2");
    }
    Ok(())
}
