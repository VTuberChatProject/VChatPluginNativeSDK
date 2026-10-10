//! Rust SDK 示例：序号状态、工具栏命令和安全 C ABI。
use prost::Message;
use std::{
    ffi::c_void,
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    thread::JoinHandle,
};
use vchat_plugin_api::{abi::*, proto::*};

struct Context {
    host: HostApi,
    count: u64,
    fail_restore: bool,
    fail_start: bool,
    sleep_ms: u64,
    post_on_start: bool,
    duplicate_start_post: bool,
    background_enabled: bool,
    background: Option<(Arc<AtomicBool>, JoinHandle<()>)>,
    background_posts: Arc<AtomicU64>,
    start_epoch: u64,
}
fn boundary(f: impl FnOnce() -> i32) -> i32 {
    catch_unwind(AssertUnwindSafe(f)).unwrap_or(STATUS_ERROR)
}
unsafe fn context<'a>(p: PluginContext) -> &'a mut Context {
    unsafe { &mut *p.cast::<Context>() }
}
unsafe fn bytes<'a>(p: *const u8, n: usize) -> &'a [u8] {
    if n == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(p, n) }
    }
}
unsafe extern "C" fn create(
    host: *const HostApi,
    data: *const u8,
    len: usize,
    out: *mut PluginContext,
) -> i32 {
    boundary(|| {
        if host.is_null() || out.is_null() {
            return STATUS_ERROR;
        }
        let host = unsafe { *host };
        if host.abi_version != ABI_VERSION
            || host.struct_size < std::mem::size_of::<HostApi>() as u32
        {
            return STATUS_ERROR;
        }
        let config: serde_json::Value =
            serde_json::from_slice(unsafe { bytes(data, len) }).unwrap_or_default();
        let value = Box::new(Context {
            host,
            count: 0,
            fail_restore: config["fail_restore"].as_bool().unwrap_or(false),
            fail_start: config["fail_start"].as_bool().unwrap_or(false),
            post_on_start: config["post_on_start"].as_bool().unwrap_or(false),
            duplicate_start_post: config["duplicate_start_post"].as_bool().unwrap_or(false),
            sleep_ms: config["sleep_ms"].as_u64().unwrap_or(0),
            background_enabled: config["background_post"].as_bool().unwrap_or(false),
            background: None,
            background_posts: Arc::new(AtomicU64::new(0)),
            start_epoch: 0,
        });
        unsafe {
            *out = Box::into_raw(value).cast();
        }
        STATUS_OK
    })
}
unsafe extern "C" fn start(p: PluginContext) -> i32 {
    boundary(|| {
        let ctx = unsafe { context(p) };
        if ctx.background.is_some() {
            return STATUS_ERROR;
        }
        ctx.start_epoch += 1;
        let ui = UiContribution {
            commands: vec![UiCommand {
                id: "append-counter".into(),
                label: "计数".into(),
                icon: "hash".into(),
                tooltip: "插入插件计数".into(),
                enabled: true,
                selected: false,
            }],
            fields: vec![UiField {
                id: "label".into(),
                label: "标签".into(),
                kind: "text".into(),
                value_json: "\"计数\"".into(),
                ..Default::default()
            }],
        };
        let request = HostRequest {
            service: "ui.register".into(),
            ui: Some(ui),
            ..Default::default()
        }
        .encode_to_vec();
        let status = unsafe {
            (ctx.host.invoke)(
                ctx.host.context,
                request.as_ptr(),
                request.len(),
                ignore_write,
                std::ptr::null_mut(),
            )
        };
        if status != STATUS_OK {
            return status;
        }
        // 类型声明先于消息入队，热更新候选的两个请求共同暂存并按序提交。
        if ctx.post_on_start || ctx.background_enabled {
            let request = HostRequest {
                service: "message.types.register".into(),
                payload_json: serde_json::to_vec(&serde_json::json!({
                    "version":1,"types":[{"id":"remote","label":"远程消息"}]
                }))
                .unwrap(),
                ..Default::default()
            }
            .encode_to_vec();
            let status = unsafe {
                (ctx.host.invoke)(
                    ctx.host.context,
                    request.as_ptr(),
                    request.len(),
                    ignore_write,
                    std::ptr::null_mut(),
                )
            };
            if status != STATUS_OK {
                return status;
            }
        }
        if ctx.post_on_start {
            let event = format!("start-{}-{}", ctx.host.generation, ctx.start_epoch);
            let status = post_message(
                ctx.host.invoke,
                ctx.host.context as usize,
                &event,
                "插件启动",
            );
            if status != STATUS_OK {
                return status;
            }
            if ctx.duplicate_start_post {
                let status = post_message(
                    ctx.host.invoke,
                    ctx.host.context as usize,
                    &event,
                    "插件启动",
                );
                if status != STATUS_OK {
                    return status;
                }
            }
        }
        if ctx.fail_start {
            STATUS_ERROR
        } else {
            if ctx.background_enabled {
                let stop = Arc::new(AtomicBool::new(false));
                let worker_stop = stop.clone();
                let host_context = ctx.host.context as usize;
                let host_invoke = ctx.host.invoke;
                let generation = ctx.host.generation;
                let epoch = ctx.start_epoch;
                let posts = ctx.background_posts.clone();
                let handle = std::thread::spawn(move || {
                    let mut sequence = 0;
                    while !worker_stop.load(Ordering::Acquire) {
                        sequence += 1;
                        let event = format!("background-{generation}-{epoch}-{sequence}");
                        if post_message(host_invoke, host_context, &event, "后台插件消息")
                            == STATUS_OK
                        {
                            posts.fetch_add(1, Ordering::AcqRel);
                        }
                        std::thread::park_timeout(std::time::Duration::from_millis(10));
                    }
                });
                ctx.background = Some((stop, handle));
            }
            STATUS_OK
        }
    })
}
/// 后台线程只使用稳定宿主地址、函数指针和自有字节，消息格式与 HTTP 入口共享。
fn post_message(invoke: HostInvoke, context: usize, event: &str, text: &str) -> i32 {
    let payload = serde_json::json!({
        "source_id":"internal:example.counter", "source_message_id":event,
        "username":"示例插件", "origin_ids":[],
        "message_type":"remote",
        "document":{"version":1,"revision":0,"paragraphs":[{"segments":[{"text":text,"style":{}}]}]}
    });
    let bytes = HostRequest {
        service: "message.post".into(),
        event_id: event.into(),
        payload_json: serde_json::to_vec(&payload).unwrap(),
        ..Default::default()
    }
    .encode_to_vec();
    unsafe {
        invoke(
            context as HostContext,
            bytes.as_ptr(),
            bytes.len(),
            ignore_write,
            std::ptr::null_mut(),
        )
    }
}
unsafe extern "C" fn ignore_write(_: *mut c_void, _: *const u8, _: usize) -> i32 {
    STATUS_OK
}
unsafe extern "C" fn invoke(
    p: PluginContext,
    data: *const u8,
    len: usize,
    write: WriteCallback,
    out: *mut c_void,
) -> i32 {
    boundary(|| {
        let ctx = unsafe { context(p) };
        let Ok(request) = PluginRequest::decode(unsafe { bytes(data, len) }) else {
            return STATUS_ERROR;
        };
        if request.protocol_version != 1 {
            return STATUS_ERROR;
        }
        std::thread::sleep(std::time::Duration::from_millis(ctx.sleep_ms));
        ctx.count += 1;
        let mut response = PluginResponse {
            payload_json: request.payload_json,
            ..Default::default()
        };
        if request.operation == "counter" {
            response.payload_json = serde_json::to_vec(&ctx.count).unwrap();
        }
        if request.operation == "background-count" {
            response.payload_json =
                serde_json::to_vec(&ctx.background_posts.load(Ordering::Acquire)).unwrap();
        }
        if request.operation == "ui.command"
            && let Some(mut document) = request.document
        {
            let expected_revision = document.revision;
            if document.paragraphs.is_empty() {
                document.paragraphs.push(Paragraph::default());
            }
            document
                .paragraphs
                .last_mut()
                .unwrap()
                .segments
                .push(Segment {
                    text: format!(" {}", ctx.count),
                    ..Default::default()
                });
            document.revision += 1;
            document.json.clear();
            response.edit = Some(DocumentEdit {
                expected_revision,
                replacement: Some(document),
            });
        }
        let data = response.encode_to_vec();
        unsafe { write(out, data.as_ptr(), data.len()) }
    })
}
/// quiesce 等待后台线程结束；返回成功时所有 HostContext 回调已完成。
unsafe extern "C" fn quiesce(p: PluginContext) -> i32 {
    boundary(|| {
        let ctx = unsafe { context(p) };
        if let Some((stop, handle)) = ctx.background.take() {
            stop.store(true, Ordering::Release);
            handle.thread().unpark();
            if handle.join().is_err() {
                return STATUS_ERROR;
            }
        }
        STATUS_OK
    })
}
unsafe extern "C" fn save(p: PluginContext, write: WriteCallback, out: *mut c_void) -> i32 {
    boundary(|| {
        let ctx = unsafe { context(p) };
        let state = StateSnapshot {
            plugin_id: "example.counter".into(),
            format_version: 1,
            data: ctx.count.to_le_bytes().to_vec(),
        }
        .encode_to_vec();
        unsafe { write(out, state.as_ptr(), state.len()) }
    })
}
unsafe extern "C" fn restore(p: PluginContext, data: *const u8, len: usize) -> i32 {
    boundary(|| {
        let ctx = unsafe { context(p) };
        if ctx.fail_restore {
            return STATUS_ERROR;
        }
        let Ok(snapshot) = StateSnapshot::decode(unsafe { bytes(data, len) }) else {
            return STATUS_ERROR;
        };
        if snapshot.plugin_id != "example.counter" || snapshot.format_version != 1 {
            return STATUS_ERROR;
        }
        let Ok(value) = snapshot.data.try_into() else {
            return STATUS_ERROR;
        };
        ctx.count = u64::from_le_bytes(value);
        STATUS_OK
    })
}
unsafe extern "C" fn destroy(p: PluginContext) -> i32 {
    boundary(|| {
        if !p.is_null() {
            if unsafe { quiesce(p) } != STATUS_OK {
                return STATUS_ERROR;
            }
            unsafe {
                drop(Box::from_raw(p.cast::<Context>()));
            }
        }
        STATUS_OK
    })
}
static API: PluginApi = PluginApi {
    abi_version: ABI_VERSION,
    struct_size: std::mem::size_of::<PluginApi>() as u32,
    create,
    start,
    invoke,
    quiesce,
    resume: start,
    save_state: save,
    restore_state: restore,
    destroy,
};
#[unsafe(no_mangle)]
pub extern "C" fn vchat_internal_plugin_get_api() -> *const PluginApi {
    &API
}
