//! 插件通过实例级函数表读取运行目录，数据由调用者拥有。
use crate::{abi::HostApi, proto::HostRequest, sdk};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuntimePaths {
    pub package_root: PathBuf,
    pub resources_root: PathBuf,
    pub data_root: PathBuf,
    #[serde(default)]
    pub assets_root: Option<PathBuf>,
}
/// # Safety
/// 宿主上下文必须在当前插件实例生命周期内有效。
pub unsafe fn runtime_paths(host: &HostApi) -> Result<RuntimePaths, String> {
    let reply = unsafe {
        sdk::call_host(
            host,
            &HostRequest {
                service: "runtime.paths".into(),
                ..Default::default()
            },
        )
    }?;
    serde_json::from_slice(&reply).map_err(|e| format!("插件运行目录无效: {e}"))
}
