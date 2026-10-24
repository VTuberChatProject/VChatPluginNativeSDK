//! Rust SDK：拥有型业务数据与同步 C ABI 视图的安全转换入口。
use crate::{
    abi,
    memory::{Arena, Budget, Value, checked, header},
    proto,
};
use std::ffi::c_void;

macro_rules! with_view {
    ($fn:ident,$model:ident) => {
        /// 在闭包返回前保持所有嵌套视图有效。
        pub fn $fn<R>(
            value: &proto::$model,
            f: impl FnOnce(*const abi::$model) -> R,
        ) -> Result<R, String> {
            let mut arena = Arena::default();
            let view = value.view(&mut arena)?;
            Ok(f(&view))
        }
    };
}
with_view!(with_request, PluginRequest);
with_view!(with_host_request, HostRequest);
with_view!(with_snapshot, StateSnapshot);
macro_rules! read_view {
    ($fn:ident,$model:ident) => {
        /// 将调用期视图深拷贝成插件拥有的数据。
        /// # Safety
        /// 参数及其全部嵌套指针必须指向当前调用期有效内存。
        pub unsafe fn $fn(value: *const abi::$model) -> Result<proto::$model, String> {
            let prefix = unsafe { checked(value.cast::<abi::Header>())? };
            header::<abi::$model>(*prefix)?;
            unsafe { proto::$model::read(*checked(value)?, &mut Budget::default()) }
        }
    };
}
read_view!(read_request, PluginRequest);
read_view!(read_host_request, HostRequest);
read_view!(read_snapshot, StateSnapshot);

fn response(kind: u32) -> abi::Response {
    abi::Response {
        header: abi::Header::of::<abi::Response>(),
        status: abi::STATUS_OK,
        kind,
        error_message: abi::StringView::borrow(""),
        plugin: std::ptr::null(),
        snapshot: std::ptr::null(),
        bytes: abi::BytesView::borrow(&[]),
        paths: std::ptr::null(),
    }
}
/// 在同步 sink 中复制插件回复，返回值传播 sink 失败。
/// # Safety
/// sink 与用户上下文必须在调用期间有效。
pub unsafe fn write_response(
    value: &proto::PluginResponse,
    sink: abi::ReplySink,
    out: *mut c_void,
) -> i32 {
    let mut arena = Arena::default();
    match value.view(&mut arena) {
        Ok(view) => {
            let mut r = response(abi::RESPONSE_PLUGIN);
            r.plugin = &view;
            unsafe { sink(out, &r) }
        }
        Err(e) => {
            tracing::warn!(%e,"插件回复视图校验失败");
            abi::STATUS_INVALID
        }
    }
}
/// 同步复制状态快照。
/// # Safety
/// sink 与用户上下文必须有效。
pub unsafe fn write_snapshot(
    value: &proto::StateSnapshot,
    sink: abi::ReplySink,
    out: *mut c_void,
) -> i32 {
    let mut arena = Arena::default();
    match value.view(&mut arena) {
        Ok(view) => {
            let mut r = response(abi::RESPONSE_SNAPSHOT);
            r.snapshot = &view;
            unsafe { sink(out, &r) }
        }
        Err(_) => abi::STATUS_INVALID,
    }
}
/// 同步复制宿主 JSON/原始字节回复。
/// # Safety
/// sink 与用户上下文必须有效。
pub unsafe fn write_host_response(value: &[u8], sink: abi::ReplySink, out: *mut c_void) -> i32 {
    if value.len() > abi::MAX_BYTES {
        return abi::STATUS_INVALID;
    }
    let mut r = response(abi::RESPONSE_HOST);
    r.bytes = abi::BytesView::borrow(value);
    unsafe { sink(out, &r) }
}
unsafe fn read_base<'a>(
    value: *const abi::Response,
    kind: u32,
) -> Result<&'a abi::Response, String> {
    let prefix = unsafe { checked(value.cast::<abi::Header>())? };
    header::<abi::Response>(*prefix)?;
    let r = unsafe { checked(value)? };
    if r.status != abi::STATUS_OK {
        return Err(unsafe { r.error_message.to_owned()? });
    }
    if r.kind != kind {
        return Err("回复类型不匹配".into());
    }
    Ok(r)
}
/// # Safety
/// 回复及嵌套视图在本次同步回调期间有效。
pub unsafe fn read_response(value: *const abi::Response) -> Result<proto::PluginResponse, String> {
    let r = unsafe { read_base(value, abi::RESPONSE_PLUGIN)? };
    let prefix = unsafe { checked(r.plugin.cast::<abi::Header>())? };
    header::<abi::PluginResponse>(*prefix)?;
    unsafe { proto::PluginResponse::read(*checked(r.plugin)?, &mut Budget::default()) }
}
/// # Safety
/// 回复及嵌套视图在本次同步回调期间有效。
pub unsafe fn read_snapshot_response(
    value: *const abi::Response,
) -> Result<proto::StateSnapshot, String> {
    let r = unsafe { read_base(value, abi::RESPONSE_SNAPSHOT)? };
    unsafe { read_snapshot(r.snapshot) }
}
/// # Safety
/// 回复及嵌套视图在本次同步回调期间有效。
pub unsafe fn read_host_response(value: *const abi::Response) -> Result<Vec<u8>, String> {
    let r = unsafe { read_base(value, abi::RESPONSE_HOST)? };
    if !r.paths.is_null() {
        let paths = unsafe { proto::RuntimePaths::read_ptr(r.paths, &mut Budget::default())? };
        return serde_json::to_vec(&paths).map_err(|e| e.to_string());
    }
    unsafe { Vec::<u8>::read(r.bytes, &mut Budget::default()) }
}

struct Collect {
    value: Option<Result<Vec<u8>, String>>,
    duplicate: bool,
}
unsafe extern "C" fn collect(out: *mut c_void, value: *const abi::Response) -> i32 {
    if out.is_null() {
        return abi::STATUS_INVALID;
    }
    let output = unsafe { &mut *out.cast::<Collect>() };
    if output.value.is_some() {
        output.duplicate = true;
        return abi::STATUS_INVALID;
    }
    let prefix = match unsafe { checked(value.cast::<abi::Header>()) } {
        Ok(v) => v,
        Err(_) => return abi::STATUS_INVALID,
    };
    if header::<abi::Response>(*prefix).is_err() {
        return abi::STATUS_INVALID;
    }
    let wire = match unsafe { checked(value) } {
        Ok(v) => v,
        Err(_) => return abi::STATUS_INVALID,
    };
    if wire.status != abi::STATUS_OK {
        return match unsafe { wire.error_message.to_owned() } {
            Ok(message) => {
                output.value = Some(Err(format!("宿主错误 {}: {message}", wire.status)));
                abi::STATUS_OK
            }
            Err(_) => abi::STATUS_INVALID,
        };
    }
    output.value = Some(unsafe { read_host_response(value) });
    if output.value.as_ref().is_some_and(Result::is_ok) {
        abi::STATUS_OK
    } else {
        abi::STATUS_INVALID
    }
}
/// 调用已注入的实例级宿主接口，并在返回前复制响应。
/// # Safety
/// 宿主表及其上下文必须在实例生命周期内有效，调用不能与 destroy 并发。
pub unsafe fn call_host(
    host: &abi::HostApi,
    request: &proto::HostRequest,
) -> Result<Vec<u8>, String> {
    validate_host(host)?;
    let mut output = Collect {
        value: None,
        duplicate: false,
    };
    let status = with_host_request(request, |view| unsafe {
        (host.invoke)(
            host.context,
            view,
            collect,
            (&mut output as *mut Collect).cast(),
        )
    })?;
    if status != abi::STATUS_OK {
        return match output.value {
            Some(Err(error)) => Err(error),
            _ => Err(format!("宿主调用失败: {status}")),
        };
    }
    if output.duplicate {
        return Err("宿主重复回复".into());
    }
    output.value.ok_or("宿主缺少回复")?
}
/// 检查宿主函数表的完整前缀。
pub fn validate_host(host: &abi::HostApi) -> Result<(), String> {
    if host.abi_version != abi::ABI_VERSION || host.struct_size < size_of::<abi::HostApi>() as u32 {
        return Err("宿主 ABI 不兼容".into());
    }
    Ok(())
}

/// 将错误回复给同步等待方，并保留错误状态。
/// # Safety
/// sink 与上下文必须有效。
pub unsafe fn write_error(
    status: i32,
    message: &str,
    sink: abi::ReplySink,
    out: *mut c_void,
) -> i32 {
    let mut r = response(abi::RESPONSE_HOST);
    r.status = status;
    r.error_message = abi::StringView::borrow(message);
    let result = unsafe { sink(out, &r) };
    if result == abi::STATUS_OK {
        status
    } else {
        result
    }
}

/// 根据标准服务生成结构化响应；配置和自定义操作保留 JSON 字节。
/// # Safety
/// sink 与上下文必须在同步调用期间有效。
pub unsafe fn write_host_response_for(
    request: &proto::HostRequest,
    value: &[u8],
    sink: abi::ReplySink,
    out: *mut c_void,
) -> i32 {
    if request.service == "runtime.paths" {
        let paths = match serde_json::from_slice::<proto::RuntimePaths>(value) {
            Ok(v) => v,
            Err(_) => return abi::STATUS_INVALID,
        };
        let mut arena = Arena::default();
        let view = match paths.view(&mut arena) {
            Ok(v) => v,
            Err(_) => return abi::STATUS_INVALID,
        };
        let mut r = response(abi::RESPONSE_HOST);
        r.paths = &view;
        return unsafe { sink(out, &r) };
    }
    unsafe { write_host_response(value, sink, out) }
}

/// 从外部指针校验并复制完整宿主表，读取函数指针之前验证其非空。
/// # Safety
/// 指针应指向至少可读 Header 的内存，并按 struct_size 提供后续字节。
pub unsafe fn read_host_api(ptr: *const abi::HostApi) -> Result<abi::HostApi, String> {
    if !(ptr as usize).is_multiple_of(std::mem::align_of::<abi::HostApi>()) {
        return Err("宿主表地址未对齐".into());
    }
    let prefix = unsafe { checked(ptr.cast::<abi::Header>())? };
    header::<abi::HostApi>(*prefix)?;
    let callback = unsafe {
        ptr.cast::<u8>()
            .add(std::mem::offset_of!(abi::HostApi, invoke))
            .cast::<usize>()
            .read()
    };
    if callback == 0 {
        return Err("宿主回调为空".into());
    }
    Ok(unsafe { *checked(ptr)? })
}

/// 在复制插件表前检查完整前缀、自然对齐和全部必需函数槽。
/// # Safety
/// 指针至少提供可读 Header，并按 struct_size 提供后续字节；库保持加载。
pub unsafe fn read_plugin_api(ptr: *const abi::PluginApi) -> Result<abi::PluginApi, String> {
    if !(ptr as usize).is_multiple_of(std::mem::align_of::<abi::PluginApi>()) {
        return Err("插件表地址未对齐".into());
    }
    let prefix = unsafe { checked(ptr.cast::<abi::Header>())? };
    header::<abi::PluginApi>(*prefix)?;
    let slots = [
        std::mem::offset_of!(abi::PluginApi, create),
        std::mem::offset_of!(abi::PluginApi, start),
        std::mem::offset_of!(abi::PluginApi, invoke),
        std::mem::offset_of!(abi::PluginApi, quiesce),
        std::mem::offset_of!(abi::PluginApi, resume),
        std::mem::offset_of!(abi::PluginApi, save_state),
        std::mem::offset_of!(abi::PluginApi, restore_state),
        std::mem::offset_of!(abi::PluginApi, destroy),
    ];
    for slot in slots {
        if unsafe { ptr.cast::<u8>().add(slot).cast::<usize>().read() } == 0 {
            return Err("插件表缺少必需函数".into());
        }
    }
    Ok(unsafe { *checked(ptr)? })
}
