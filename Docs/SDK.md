<!-- C ABI 2 插件开发、验证及分发规范。 -->
# Native 插件 SDK

本 SDK 提供插件开发所需的接口和工具。ABI 主版本为 2，业务模型版本为 1；`PluginRequest.protocol_version` 对应 `BUSINESS_MODEL_VERSION`。宿主内部 IPC 由主程序管理。

## 内部 C ABI

公共头文件为 VCPluginInterface.h，ABI 主版本为 2。插件导出 vchat_internal_plugin_get_api，返回稳定只读 PluginApi 表。
worker 校验版本、结构大小和函数表，向 create 注入实例级 HostApi。宿主服务通过该表转发到主程序。
Rust 使用 vchat-plugin-api 的 abi/sdk 模块，默认构建包含业务模型及 C ABI 封装，`host-protocol` 特性可选启用拥有型模型的 Protobuf 编解码。
proto 模块保留拥有型业务模型命名，默认构建通过结构化视图交换这些数据。

| SDK 方法 | 用途 |
| --- | --- |
| sdk::read_request / read_host_request | 深拷贝结构化输入，检查版本、长度及嵌套预算 |
| sdk::with_request / with_host_request | 在同步闭包期间保持嵌套视图有效 |
| sdk::write_response / write_snapshot | 同步向回复 sink 复制结果或快照 |
| sdk::call_host | 调用注入表并复制宿主响应 |
| runtime::runtime_paths | 读取当前包、资源、持久化数据与共享 Assets 目录 |

ABI 使用固定宽度整数、自然对齐、UTF-8 字符串、带布局信息的集合和不透明上下文。输入借用到函数返回，回复由 sink 同步深拷贝。
各侧管理各自内存，插件对象由 destroy 释放。合法带 sink 调用恰好回复一次；状态码和函数返回值传播失败。
调用总量上限 16 MiB，深度上限 64。panic/异常在插件内部捕获；生命周期与 invoke 串行，后台线程可调用宿主。
标准消息、文档、UI 与服务数据通过 C 结构传递，配置、表单 schema 和自定义操作保留用途明确的 JSON。
宿主权限为 messages、events、ui、frontend、web；runtime.paths/log 面向全部实例。现有消息类型、前端、窗口与 Web 路由服务继续提供。
编辑提交检查 expected_revision，路由整组校验冲突，剧本回执保留 run_id/batch_id/message_id/generation，过滤退休批次。

## 按输出卡片投递

manifest 声明 `output_cards_v1`，通过版本 2 的 `frontend.register` 登记端点：

```json
{"version":2,"outputs":[{"card_id":"live","receipt_mode":"completed"},{"card_id":"history","receipt_mode":"accepted"}]}
```

`card_id` 对应 `ui.register` 的输出卡片。Rust 的 `output_route::OutputRegistration` 与 `frontend_id(kind, plugin_id, card_id)` 可构造登记与稳定身份。宿主向该卡投递版本 2 信封，包含 `frontend_id/plugin_id/card_id/message_id/batch_id`，消息正文保存在 `message`；剧本另带 `run_id/generation`。清屏沿用 `clear` 操作并携带相同目标定位。

插件通过 `frontend.receipt` 原样回传目标与批次字段。实时端点以 `completed` 确认消费完成，归档端点以 `accepted` 确认接收；失败使用 `failed` 与 `error`。宿主按当前实例会话、卡片和活动批次校验回执，退休批次的回执会被拒绝。普通消息、预览和剧本均遵循用户连线。

预览使用 `preview` 操作和完整卡片、消息、批次身份，插件消费后跳过正式队列回执。

这些字段通过现有 FrontendPayload 扩展区传递，C ABI 2 的结构大小与布局保持稳定。旧版插件继续展示入口，画布提示待适配；升级并登记端点后可手动连接。

## 插件模型

`model` 模块提供拥有型 `Document`、`Message`、`Paragraph`、`Segment`、`TextStyle`、`Image` 和输出卡片类型。`document` 与 `output` 模块负责这些模型与 `proto` 交换模型的转换，默认值、可选字段和扩展 JSON 按现有业务协议保留。

```rust
use vchat_plugin_api::{document::document_to_proto, model::Document};
let document = document_to_proto(&Document::plain("你好，VChat"));
assert_eq!(document.version, 1);
```

## 构建与开发动态库

```powershell
cargo build --locked --workspace
cargo build --locked --release -p vchat-plugin-api --features sdk-dev
cmake -S Examples -B target/cpp-plugin
cmake --build target/cpp-plugin --config Release
python Tools/package-sdk.py
```

开发动态库使用 `VCPluginInterface.dll`、`libVCPluginInterface.so` 或 `libVCPluginInterface.dylib` 发行名。通过显式符号加载提供版本查询、创建和销毁宿主接口表，由测试宿主注入后端。源码构建产物使用 `vchat_plugin_api` 名称，打包工具统一重命名。

```powershell
python Tools/sdk-host.py --library target/dist/VCPluginSDK-windows-x86_64/VCPluginInterface.dll --plugin target/cpp-plugin/Release/vchat_cpp_example.dll
```

测试宿主验证实际动态加载及 `create/start/quiesce/save/resume/destroy` 生命周期。插件发行通过注入函数表使用宿主服务，开发动态库仅用于开发测试。

## 插件包

`.vcplug` 为 ZIP，包含 `manifest.json`、`bin/` 下入口与自身原生依赖，以及可选 `resources/`。清单版本 2、ABI 2；身份、权限和插件版本来自清单。入口格式为 `bin/<动态库文件名>`。

```powershell
python Tools/package-plugin.py --manifest Examples/manifest.json --library target/debug/vchat_example_plugin.dll --output target/example.vcplug
```

`--resources <目录>` 将插件资源完整收集到包内。宿主导入执行包结构、平台、架构与原生依赖校验。Linux 原生依赖使用 `$ORIGIN`，macOS 使用 `@loader_path/@rpath`，Windows 放在同一 `bin/`。插件包应仅收集自身运行依赖。

每代解压路径独立，`data_root` 长期保留，`resources_root` 随代次切换，`assets_root` 随宿主数据目录移动。热更新先创建候选、暂停旧实例、导出/迁移快照并启动候选；失败后恢复旧实例。`quiesce` 需要排空后台线程、任务与回调。候选阶段负责校验与注册，实际输出发生于活动业务调用。

## 验证

```powershell
cargo fmt --all -- --check
cargo test --locked --workspace
cargo test --locked -p vchat-plugin-api --all-features
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo tree --locked -p vchat-plugin-api --no-default-features
```

默认依赖树由 Native 模型、序列化和 C ABI 封装组成。跨平台 CI 验证 Rust、C++ 布局断言、真实动态库生命周期及可重新构建的 SDK 包；正式 worker、热更新和播放链路由主仓库固定 SDK/插件提交执行集成验证。
