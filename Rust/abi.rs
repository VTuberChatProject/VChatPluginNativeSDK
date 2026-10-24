//! ABI 2 使用结构化调用期视图与同步回复，内存始终由所属模块管理。
pub use crate::request_views::{HostRequest, PluginRequest};
pub use crate::views::*;
use std::ffi::c_void;
pub const ABI_VERSION: u32 = 2;
pub const MAX_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_DEPTH: usize = 64;
pub const STATUS_OK: i32 = 0;
pub const STATUS_ERROR: i32 = 1;
pub const STATUS_STALE: i32 = 2;
pub const STATUS_INVALID: i32 = 3;
pub const STATUS_UNSUPPORTED: i32 = 4;
pub const STATUS_FORBIDDEN: i32 = 5;
pub const STATUS_BUSY: i32 = 6;
pub const STATUS_TIMEOUT: i32 = 7;
pub type PluginContext = *mut c_void;
pub type HostContext = *mut c_void;
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Header {
    pub abi_version: u32,
    pub struct_size: u32,
}
impl Header {
    /// 为完整的具体视图构造版本前缀。
    pub fn of<T>() -> Self {
        Self {
            abi_version: ABI_VERSION,
            struct_size: size_of::<T>() as u32,
        }
    }
}
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct StringView {
    pub data: *const u8,
    pub len: u64,
}
impl StringView {
    pub fn borrow(value: &str) -> Self {
        Self {
            data: value.as_ptr(),
            len: value.len() as u64,
        }
    }
    /// # Safety
    /// 非空指针必须指向调用期可读的 UTF-8 字节。
    pub unsafe fn to_owned(self) -> Result<String, String> {
        let mut budget = crate::memory::Budget::default();
        unsafe { <String as crate::memory::Value>::read(self, &mut budget) }
    }
}
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Slice<T> {
    pub data: *const T,
    pub len: u64,
    pub stride: u64,
}
impl<T> Slice<T> {
    pub fn borrow(value: &[T]) -> Self {
        Self {
            data: value.as_ptr(),
            len: value.len() as u64,
            stride: size_of::<T>() as u64,
        }
    }
}
pub type BytesView = Slice<u8>;
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct OptionalU32 {
    pub has_value: u32,
    pub value: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct OptionalI64 {
    pub has_value: u32,
    pub value: i64,
}
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct OptionalU64 {
    pub has_value: u32,
    pub value: u64,
}
pub const RESPONSE_PLUGIN: u32 = 1;
pub const RESPONSE_SNAPSHOT: u32 = 2;
pub const RESPONSE_HOST: u32 = 3;
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Response {
    pub header: Header,
    pub status: i32,
    pub kind: u32,
    pub error_message: StringView,
    pub plugin: *const PluginResponse,
    pub snapshot: *const StateSnapshot,
    pub bytes: BytesView,
    pub paths: *const RuntimePaths,
}
pub type ReplySink = unsafe extern "C" fn(*mut c_void, *const Response) -> i32;
pub type HostInvoke =
    unsafe extern "C" fn(HostContext, *const HostRequest, ReplySink, *mut c_void) -> i32;
#[repr(C)]
#[derive(Clone, Copy)]
pub struct HostApi {
    pub abi_version: u32,
    pub struct_size: u32,
    pub generation: u64,
    pub context: HostContext,
    pub invoke: HostInvoke,
}
pub type Create = unsafe extern "C" fn(*const HostApi, StringView, *mut PluginContext) -> i32;
pub type Lifecycle = unsafe extern "C" fn(PluginContext) -> i32;
pub type Invoke =
    unsafe extern "C" fn(PluginContext, *const PluginRequest, ReplySink, *mut c_void) -> i32;
pub type SaveState = unsafe extern "C" fn(PluginContext, ReplySink, *mut c_void) -> i32;
pub type RestoreState = unsafe extern "C" fn(PluginContext, *const StateSnapshot) -> i32;
#[repr(C)]
#[derive(Clone, Copy)]
pub struct PluginApi {
    pub abi_version: u32,
    pub struct_size: u32,
    pub create: Create,
    pub start: Lifecycle,
    pub invoke: Invoke,
    pub quiesce: Lifecycle,
    pub resume: Lifecycle,
    pub save_state: SaveState,
    pub restore_state: RestoreState,
    pub destroy: Lifecycle,
}
pub type GetApi = unsafe extern "C" fn() -> *const PluginApi;
