# rszlm

ZLMediaKit rust api

### build

- 依赖

  1. [rust-bindgen](https://github.com/rust-lang/rust-bindgen)需要安装`Clang`环境

     - 参考链接：https://rust-lang.github.io/rust-bindgen/requirements.html

  2. [ZLMediaKit](https://github.com/ZLMediaKit/ZLMediaKit)
     - 参考链接：https://github.com/ZLMediaKit/ZLMediaKit/wiki/%E5%BF%AB%E9%80%9F%E5%BC%80%E5%A7%8B

- 编译好并且 install`ZLMediaKit`之后，设置环境变量`ZLM_DIR`，该路径包含 ZLMediaKit 的`include`和`lib`以及`bin`文件夹，并将`bin`设置到`PATH`环境变量中

  如果不设置`ZLM_DIR`环境变量，rszlm-sys 会拉取`ZLMediaKit`git 代码进行编译

- 编译

```shell
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
