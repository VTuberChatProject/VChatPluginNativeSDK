<!-- Native 插件开发 SDK 的独立构建入口。 -->
# VChat Plugin Native SDK

为 VChat 内部插件提供 C ABI 2、Rust 安全封装、独立业务模型、开发接口动态库与 Rust/C++ 示例。插件通过宿主注入的函数表调用服务。

- [开发规范](Docs/SDK.md)
- Rust crate：`Rust/`，包名 `vchat-plugin-api`，版本 `0.2.0`。
- 插件数据模型：`vchat_plugin_api::model`；结构化交换模型：`vchat_plugin_api::proto`。
- 公共 C/C++ 头文件：`Rust/VCPluginInterface.h`。
- WebSocket 插件开发：[Web SDK](https://github.com/VTuberChatProject/VChatPluginWebSDK)。

```powershell
cargo build --locked --workspace
cargo test --locked --workspace
cmake -S Examples -B target/cpp-plugin
cmake --build target/cpp-plugin --config Release
python Tools/package-sdk.py
```

SDK 包输出至 `target/dist/VCPluginSDK-<os>-<arch>/`，包含开发库、头文件、独立 workspace 源码、示例、工具和文档。PowerShell 使用 `Tools/package-sdk.ps1`，Unix 使用 `bash Tools/package-sdk.sh`。

外部插件可固定本仓库提交依赖 Rust crate：

```toml
[dependencies]
vchat-plugin-api = { git = "https://github.com/VTuberChatProject/VChatPluginNativeSDK.git", rev = "<固定提交>" }
```

默认构建使用 C ABI 视图；`sdk-dev` 启用开发动态库导出，`host-protocol` 可选启用拥有型交换模型的 Protobuf 编解码。插件通过 `model` 使用独立文档、消息、样式与输出卡片类型，源码构建完整归属本仓库。

本仓库从 [VChatRev](https://github.com/VTuberChatProject/VChatRev) 提取相关路径历史，迁移源提交为 `985c2a1ae3717f6918cb6030b6dd755d01edff9a`。提取保留原始作者、时间及提交说明；路径重映射产生独立提交 ID。主程序继续管理自身运行契约，跨仓库通过 ABI 与集成测试保持兼容。

许可证：[GPL-3.0-or-later](LICENSE)。
