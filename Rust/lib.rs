//! InternalPlugin 版本化协议与跨语言 ABI。
pub mod abi;
pub mod document;
pub mod runtime;
pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/vchat.plugin.v1.rs"));
}
pub use proto::{UiCommand, UiContribution, UiField};
pub const PROTOCOL_VERSION: u32 = 1;
