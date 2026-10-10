//! C ABI 使用调用期借用的字节和宿主复制回调管理内存。
use std::ffi::c_void;

pub const ABI_VERSION: u32 = 1;
pub const STATUS_OK: i32 = 0;
pub const STATUS_ERROR: i32 = 1;
pub const STATUS_STALE: i32 = 2;
pub type PluginContext = *mut c_void;
pub type HostContext = *mut c_void;
pub type WriteCallback = unsafe extern "C" fn(*mut c_void, *const u8, usize) -> i32;
pub type HostInvoke =
    unsafe extern "C" fn(HostContext, *const u8, usize, WriteCallback, *mut c_void) -> i32;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct HostApi {
    pub abi_version: u32,
    pub struct_size: u32,
    pub generation: u64,
    pub context: HostContext,
    pub invoke: HostInvoke,
}
pub type Create = unsafe extern "C" fn(*const HostApi, *const u8, usize, *mut PluginContext) -> i32;
pub type Lifecycle = unsafe extern "C" fn(PluginContext) -> i32;
pub type Invoke =
    unsafe extern "C" fn(PluginContext, *const u8, usize, WriteCallback, *mut c_void) -> i32;
pub type SaveState = unsafe extern "C" fn(PluginContext, WriteCallback, *mut c_void) -> i32;
pub type RestoreState = unsafe extern "C" fn(PluginContext, *const u8, usize) -> i32;

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
