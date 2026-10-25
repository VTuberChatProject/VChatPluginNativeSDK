//! 稳定 C ABI 2、插件业务模型及独立开发 SDK。
pub mod abi;
pub mod document;
mod memory;
pub mod model;
pub mod output;
mod projection;
#[path = "models.rs"]
pub mod proto;
mod request_views;
pub mod runtime;
pub mod sdk;
pub mod sdk_host;
mod service_models;
mod views;
pub use proto::{UiCommand, UiContribution, UiField};
/// 内部拥有型请求的业务模型版本；ABI 和私有 IPC 分别独立版本化。
pub const BUSINESS_MODEL_VERSION: u32 = 1;
/// 历史字段 protocol_version 使用的业务模型版本别名。
pub const PROTOCOL_VERSION: u32 = BUSINESS_MODEL_VERSION;

#[cfg(test)]
mod sdk_tests;
