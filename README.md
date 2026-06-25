# rszlm

ZLMediaKit rust api

## 构建说明

### 依赖

1. [rust-bindgen](https://github.com/rust-lang/rust-bindgen)  
   需要安装 `Clang` 环境。  
   参考：[bindgen requirements](https://rust-lang.github.io/rust-bindgen/requirements.html)

2. [ZLMediaKit](https://github.com/ZLMediaKit/ZLMediaKit)  
   参考：[快速开始](https://github.com/ZLMediaKit/ZLMediaKit/wiki/%E5%BF%AB%E9%80%9F%E5%BC%80%E5%A7%8B)

   `rszlm-sys` 在构建时按以下优先级获取 ZLMediaKit：

   1. 若设置了 `ZLM_DIR`，直接使用该目录下已编译好的库（该路径需包含 `include`、`lib`、`bin` 文件夹，并将 `bin` 加入 `PATH`）。优先级最高，跳过下载与源码编译。
   2. 否则，**动态库（默认）** 会自动从 [ZLMediaKit-Build releases](https://github.com/BenLocal/ZLMediaKit-Build/releases) 下载与目标平台对应的预编译产物，无需在本地安装 ZLMediaKit 或其编译工具链。下载地址由环境变量 `ZLM_RELEASE_URL` 控制。
   3. **静态库（`static` feature）**，或显式设置 `ZLM_BUILD_FROM_SOURCE=1` 时，会拉取 ZLMediaKit 源码在本地编译（预编译包的 C API 仅以动态库 `libmk_api.so` 提供，无法用于静态链接）。

### 环境变量

| 环境变量 | 说明 | 默认值 |
| --- | --- | --- |
| `ZLM_DIR` | 指向已编译安装好的 ZLMediaKit 目录（含 `include`/`lib`/`bin`）。设置后优先级最高，跳过下载与源码编译。支持 `${VAR}` 形式的变量展开。 | 未设置 |
| `ZLM_RELEASE_URL` | 预编译 release 的下载地址。可为基础 URL（自动追加对应平台的资源文件名），也可为以 `.tar.gz`/`.zip` 结尾的完整压缩包 URL。 | `https://github.com/BenLocal/ZLMediaKit-Build/releases/latest/download` |
| `ZLM_BRANCH` | 选择 release 中对应分支的产物（资源名形如 `zlmediakit_<branch>_<os>_<arch>_latest`）。 | `master` |
| `ZLM_BUILD_FROM_SOURCE` | 设为 `1`/`true`/`ON` 时，即使是动态库也强制从源码编译，而不下载预编译包。 | 未设置 |
| `ZLM_GIT` | 源码编译时使用的 ZLMediaKit 仓库地址。 | ZLMediaKit GitHub 官方仓库 |
| `ZLM_GIT_ZONE` | 设为 `gitee` 时，源码编译改从 Gitee 镜像拉取（ZLMediaKit 与 libsrtp）。 | 未设置 |

> 预编译产物的资源命名规则为 `zlmediakit_{branch}_{os}_{arch}_latest.{tar.gz|zip}`，其中 `os ∈ {linux, macos, windows}`、`arch ∈ {amd64, arm64}`（Windows 为 `.zip`，其余为 `.tar.gz`）。默认地址使用 GitHub 的 `releases/latest/download` 重定向，始终指向最新 release。

### 编译

```shell
./build_zlm.sh
cargo build
```

### features

- `dynamic`和`static`编译

  - `dynamic`(默认)：

    ```toml
    rszlm = { version = "*" }
    ```

  - `static`：

    ```toml
    rszlm = { version = "*", features = ["static"] }
    ```

- `webrtc`

  ```toml
  rszlm = { version = "*", features = ["webrtc"] }
  ```

### examples

- [需要安装`gstreamer`相关依赖](https://gstreamer.freedesktop.org/documentation/installing/on-linux.html?gi-language=c)

参考

- [ZLMediaKit](https://github.com/ZLMediaKit/ZLMediaKit)
- [rust-bindgen](https://github.com/rust-lang/rust-bindgen)
- [ZLMediaKit-Build](https://github.com/BenLocal/ZLMediaKit-Build)
