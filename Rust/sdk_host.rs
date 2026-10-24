//! 开发动态库和静态宿主共享的表校验与委派实现。
use crate::{abi::*, sdk};
use std::{
    ffi::c_void,
    panic::{AssertUnwindSafe, catch_unwind},
};
struct DevelopmentHost {
    api: HostApi,
    backend: HostApi,
}
unsafe extern "C" fn dispatch(
    context: HostContext,
    request: *const HostRequest,
    sink: ReplySink,
    out: *mut c_void,
) -> i32 {
    catch_unwind(AssertUnwindSafe(|| {
        if context.is_null() {
            return STATUS_INVALID;
        }
        let host = unsafe { &*context.cast::<DevelopmentHost>() };
        if unsafe { sdk::read_host_request(request) }.is_err() {
            return STATUS_INVALID;
        }
        unsafe { (host.backend.invoke)(host.backend.context, request, sink, out) }
    }))
    .unwrap_or(STATUS_ERROR)
}
/// 创建拥有独立上下文的接口表；后端上下文由调用方持有。
/// # Safety
/// 后端表及回调须有效，out 指向可写指针。
#[cfg_attr(feature = "sdk-dev", unsafe(no_mangle))]
pub unsafe extern "C" fn vc_interface_create_host(
    backend: *const HostApi,
    out: *mut *mut HostApi,
) -> i32 {
    catch_unwind(AssertUnwindSafe(|| {
        if out.is_null() {
            return STATUS_INVALID;
        }
        unsafe {
            *out = std::ptr::null_mut();
        }
        if backend.is_null() {
            return STATUS_INVALID;
        }
        let backend = match unsafe { sdk::read_host_api(backend) } {
            Ok(v) => v,
            Err(error) => {
                tracing::warn!(%error,"开发宿主接口无效");
                return STATUS_INVALID;
            }
        };
        let mut host = Box::new(DevelopmentHost {
            api: HostApi {
                abi_version: ABI_VERSION,
                struct_size: size_of::<HostApi>() as u32,
                generation: backend.generation,
                context: std::ptr::null_mut(),
                invoke: dispatch,
            },
            backend,
        });
        host.api.context = (&mut *host as *mut DevelopmentHost).cast();
        let ptr = Box::into_raw(host);
        unsafe {
            *out = &mut (*ptr).api;
        }
        STATUS_OK
    }))
    .unwrap_or(STATUS_ERROR)
}
/// 销毁前调用方必须排空插件与后台回调。
/// # Safety
/// 参数必须来自 create_host，且仅释放一次。
#[cfg_attr(feature = "sdk-dev", unsafe(no_mangle))]
pub unsafe extern "C" fn vc_interface_destroy_host(host: *mut HostApi) -> i32 {
    if host.is_null() {
        return STATUS_INVALID;
    }
    let context = unsafe { (*host).context };
    if context.is_null() {
        return STATUS_INVALID;
    }
    unsafe {
        drop(Box::from_raw(context.cast::<DevelopmentHost>()));
    }
    STATUS_OK
}
#[cfg_attr(feature = "sdk-dev", unsafe(no_mangle))]
pub extern "C" fn vc_interface_get_version() -> u32 {
    ABI_VERSION
}
