//! 插件通过同步宿主服务读取本代次资源路径，用户配置保持独立。
use crate::{abi::*, proto::HostRequest};
use prost::Message;
use serde::{Deserialize, Serialize};
use std::{ffi::c_void, path::PathBuf};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuntimePaths {
    pub package_root: PathBuf,
    pub resources_root: PathBuf,
    pub data_root: PathBuf,
    /// 宿主提供的共享可写资源根目录；旧宿主可省略。
    #[serde(default)]
    pub assets_root: Option<PathBuf>,
}

/// 在 create 或后续生命周期调用中获取宿主管理的绝对路径。
///
/// # Safety
/// `host` 的上下文与回调必须在当前插件实例的生命周期内有效。
pub unsafe fn runtime_paths(host: &HostApi) -> Result<RuntimePaths, String> {
    let request = HostRequest {
        service: "runtime.paths".into(),
        ..Default::default()
    }
    .encode_to_vec();
    let mut response = Vec::<u8>::new();
    // SAFETY: 调用方保证 HostApi 有效，宿主只在当前同步调用内使用缓冲区。
    let status = unsafe {
        (host.invoke)(
            host.context,
            request.as_ptr(),
            request.len(),
            collect,
            (&mut response as *mut Vec<u8>).cast(),
        )
    };
    if status != STATUS_OK {
        return Err(format!("读取插件运行路径失败: {status}"));
    }
    serde_json::from_slice(&response).map_err(|error| format!("插件运行路径响应无效: {error}"))
}

unsafe extern "C" fn collect(context: *mut c_void, data: *const u8, len: usize) -> i32 {
    if context.is_null() || (len > 0 && data.is_null()) || len > 1024 * 1024 {
        return STATUS_ERROR;
    }
    // SAFETY: 上下文由 runtime_paths 持有，数据在宿主回调期间有效。
    let response = unsafe { &mut *context.cast::<Vec<u8>>() };
    if response.len().saturating_add(len) > 1024 * 1024 {
        return STATUS_ERROR;
    }
    if len > 0 {
        response.extend_from_slice(unsafe { std::slice::from_raw_parts(data, len) });
    }
    STATUS_OK
}

#[cfg(test)]
mod tests {
    use super::*;

    unsafe extern "C" fn respond(
        context: HostContext,
        data: *const u8,
        len: usize,
        write: WriteCallback,
        out: *mut c_void,
    ) -> i32 {
        let request =
            HostRequest::decode(unsafe { std::slice::from_raw_parts(data, len) }).unwrap();
        assert_eq!(request.service, "runtime.paths");
        let response = unsafe { &*context.cast::<Vec<u8>>() };
        unsafe { write(out, response.as_ptr(), response.len()) }
    }

    #[test]
    fn legacy_paths_allow_missing_assets_root() {
        let paths: RuntimePaths = serde_json::from_value(serde_json::json!({
            "package_root":"package", "resources_root":"package/resources", "data_root":"data"
        }))
        .unwrap();
        assert!(paths.assets_root.is_none());
    }

    #[test]
    fn paths_come_from_host_and_invalid_responses_are_rejected() {
        let paths = RuntimePaths {
            package_root: PathBuf::from("包目录"),
            resources_root: PathBuf::from("包目录/resources"),
            data_root: PathBuf::from("持久化数据"),
            assets_root: Some(PathBuf::from("Assets")),
        };
        let mut response = serde_json::to_vec(&paths).unwrap();
        let host = HostApi {
            abi_version: ABI_VERSION,
            struct_size: std::mem::size_of::<HostApi>() as u32,
            generation: 1,
            context: (&mut response as *mut Vec<u8>).cast(),
            invoke: respond,
        };
        let actual = unsafe { runtime_paths(&host) }.unwrap();
        assert_eq!(actual.resources_root, paths.resources_root);
        response.clear();
        assert!(unsafe { runtime_paths(&host) }.is_err());
    }
}
