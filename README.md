# rszlm

ZLMediaKit rust api

### build

- 依赖

  1. [rust-bindgen](https://github.com/rust-lang/rust-bindgen)需要安装`Clang`

     - 参考链接：https://rust-lang.github.io/rust-bindgen/requirements.html

  2. [ZLMediaKit](https://github.com/ZLMediaKit/ZLMediaKit)

     - 参考链接：https://github.com/ZLMediaKit/ZLMediaKit/wiki/%E5%BF%AB%E9%80%9F%E5%BC%80%E5%A7%8B

- 编译

```shell
cargo build
```

### features

- `dynamic`和`static`编译，目前只在 macos 下测试成功

  - `dynamic`(默认)：

    ```toml
    rszlm = { version = "*" }
    ```

  - `static`：

    ```toml
    rszlm = { version = "*", features = ["static"] }
    ```

- `webrtc`, 暂未支持

### examples

example 中使用了`ffmpeg-next`，运行前请参照[dependencies](https://github.com/zmwangx/rust-ffmpeg/wiki/Notes-on-building#dependencies)安装相关环境
