# rszlm

ZLMediaKit rust api

## 构建说明

### 依赖

1. [rust-bindgen](https://github.com/rust-lang/rust-bindgen)  
   需要安装 `Clang` 环境。  
   参考：[bindgen requirements](https://rust-lang.github.io/rust-bindgen/requirements.html)

2. [ZLMediaKit](https://github.com/ZLMediaKit/ZLMediaKit)  
   参考：[快速开始](https://github.com/ZLMediaKit/ZLMediaKit/wiki/%E5%BF%AB%E9%80%9F%E5%BC%80%E5%A7%8B)

   - 编译并安装好 ZLMediaKit 后，设置环境变量 `ZLM_DIR`，该路径需包含 ZLMediaKit 的 `include`、`lib` 和 `bin` 文件夹，并将 `bin` 添加到 `PATH` 环境变量中。
   - 如果未设置 `ZLM_DIR`，`rszlm-sys` 会自动拉取 ZLMediaKit 源码进行编译。


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

参考
- [ZLMediaKit](https://github.com/ZLMediaKit/ZLMediaKit)
- [rust-bindgen](https://github.com/rust-lang/rust-bindgen)
