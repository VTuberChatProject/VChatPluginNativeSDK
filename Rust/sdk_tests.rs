//! ABI 契约回归：结构化字段、内存预算、同步回复和开发宿主共用实现。
use crate::{abi::*, proto, sdk, sdk_host};
use serde_json::{Value, json};
use std::ffi::c_void;
#[test]
fn nana_dialogue_v2_fixture_preserves_card_and_script_contract() {
    let plugin_id = "builtin.nana-dialogue";
    let frontend_id = crate::output_route::frontend_id("internal", plugin_id, "dialogue");
    payload_roundtrip(
        "frontend.register",
        json!({"version":2,"outputs":[
        {"card_id":"dialogue","receipt_mode":"completed"}]}),
    );
    let message = crate::model::Message::new(
        "host",
        "角色",
        crate::model::Document::plain("契约夹具"),
        1,
        1,
    );
    let payload = json!({"version":2,"frontend_id":frontend_id,"plugin_id":plugin_id,
        "card_id":"dialogue","message_id":message.id,"batch_id":"663d2d58-65da-4fea-8768-c785e48227a6",
        "run_id":"3930718f-45b7-48b5-ab2b-629091c321da","generation":7,"message":message});
    let request = proto::PluginRequest {
        protocol_version: 1,
        operation: "message".into(),
        payload_json: serde_json::to_vec(&payload).unwrap(),
        ..Default::default()
    };
    let copy =
        sdk::with_request(&request, |view| unsafe { sdk::read_request(view) }.unwrap()).unwrap();
    assert_eq!(
        serde_json::from_slice::<Value>(&copy.payload_json).unwrap(),
        payload
    );
    payload_roundtrip("frontend.message", payload.clone());
    let mut receipt = payload;
    receipt.as_object_mut().unwrap().remove("message");
    receipt["status"] = "completed".into();
    payload_roundtrip("frontend.receipt", receipt);
}
#[test]
fn output_card_v2_extensions_preserve_abi2_layout_and_identity() {
    let target = crate::output_route::frontend_id("internal", "sample", "live");
    payload_roundtrip(
        "frontend.register",
        json!({"version":2,"outputs":[
        {"card_id":"live","receipt_mode":"completed"},
        {"card_id":"history","receipt_mode":"accepted"}]}),
    );
    payload_roundtrip(
        "frontend.receipt",
        json!({"version":2,"frontend_id":target,
        "plugin_id":"sample","card_id":"live","message_id":"m","batch_id":"b","status":"completed"}),
    );
    assert_eq!(ABI_VERSION, 2);
}
fn payload_roundtrip(service: &str, payload: Value) {
    let request = proto::HostRequest {
        service: service.into(),
        payload_json: serde_json::to_vec(&payload).unwrap(),
        ..Default::default()
    };
    let back = sdk::with_host_request(&request, |view| {
        let view = unsafe { &*view };
        assert_eq!(view.payload_json.len, 0, "{service}");
        unsafe { sdk::read_host_request(view) }.unwrap()
    })
    .unwrap();
    assert_eq!(
        serde_json::from_slice::<Value>(&back.payload_json).unwrap(),
        payload,
        "{service}"
    );
}
#[test]
fn standard_host_requests_use_typed_views() {
    payload_roundtrip(
        "message.post",
        json!({"source_id":"internal:sample","source_message_id":"1","username":"测试 👋","message_type":"remote","avatar_url":"https://example.com/a.png","document":crate::model::Document::plain("富文本"),"origin_ids":["upstream"]}),
    );
    payload_roundtrip(
        "message.types.register",
        json!({"version":1,"types":[{"id":"remote","label":"远程"}]}),
    );
    payload_roundtrip(
        "frontend.register",
        json!({"version":1,"frontend_id":"sample","label":"文件","kind":"file","capabilities":["typewriter"]}),
    );
    payload_roundtrip(
        "frontend.receipt",
        json!({"version":1,"frontend_id":"sample","message_id":"m","batch_id":"b","run_id":"r","generation":9,"status":"completed"}),
    );
    payload_roundtrip(
        "web.route.register",
        json!({"version":1,"routes":[{"kind":"http","method":"GET","path":"/a/*","operation":"custom.http"}]}),
    );
    payload_roundtrip(
        "websocket.send",
        json!({"connection_id":"c","data":{"text":"网页协议"},"preview":false}),
    );
}
#[test]
fn frontend_document_and_metadata_survive_deep_copy() {
    let mut message = crate::model::Message::new(
        "source",
        "发送者",
        crate::model::Document::plain("你好 👨‍👩‍👧‍👦"),
        123,
        4,
    );
    message.document.paragraphs[0].segments[0].style.bold = true;
    message.document.paragraphs[0].segments[0]
        .style
        .extra
        .insert("effect".into(), json!({"color":"blue"}));
    payload_roundtrip(
        "frontend.message",
        json!({"version":1,"frontend_id":"sample","batch_id":"b","run_id":"r","generation":u64::MAX,"message":message}),
    );
}
#[test]
fn web_plugin_metadata_is_structured() {
    let payload = json!({"version":1,"method":"POST","path":"/test","query":"a=1","headers":{"content-type":"text/plain"},"body":[0,255,1]});
    let request = proto::PluginRequest {
        protocol_version: 1,
        operation: "frontend.http".into(),
        payload_json: serde_json::to_vec(&payload).unwrap(),
        ..Default::default()
    };
    let copy = sdk::with_request(&request, |v| {
        let v = unsafe { &*v };
        assert_eq!(v.payload_json.len, 0);
        assert!(!v.web.is_null());
        unsafe { sdk::read_request(v) }.unwrap()
    })
    .unwrap();
    assert_eq!(
        serde_json::from_slice::<Value>(&copy.payload_json).unwrap(),
        payload
    );
}
#[test]
fn invalid_headers_utf8_and_budgets_are_rejected() {
    let short = Header {
        abi_version: ABI_VERSION,
        struct_size: size_of::<Header>() as u32,
    };
    assert!(unsafe { sdk::read_request((&short as *const Header).cast()) }.is_err());
    assert!(
        unsafe {
            StringView {
                data: std::ptr::null(),
                len: 1,
            }
            .to_owned()
        }
        .is_err()
    );
    let invalid = [0xffu8];
    assert!(
        unsafe {
            StringView {
                data: invalid.as_ptr(),
                len: 1,
            }
            .to_owned()
        }
        .is_err()
    );
    let request = proto::PluginRequest {
        operation: "custom".into(),
        payload_json: vec![0; MAX_BYTES + 1],
        ..Default::default()
    };
    assert!(sdk::with_request(&request, |_| ()).is_err());
}
unsafe extern "C" fn paths(
    _: HostContext,
    request: *const HostRequest,
    sink: ReplySink,
    out: *mut c_void,
) -> i32 {
    let request = unsafe { sdk::read_host_request(request) }.unwrap();
    assert_eq!(request.service, "runtime.paths");
    unsafe {
        sdk::write_host_response_for(&request,br#"{"package_root":"/pkg","resources_root":"/pkg/resources","data_root":"/data","assets_root":"/assets"}"#,sink,out)
    }
}
#[test]
fn development_and_static_host_share_structured_paths() {
    let backend = HostApi {
        abi_version: ABI_VERSION,
        struct_size: size_of::<HostApi>() as u32,
        generation: 7,
        context: std::ptr::null_mut(),
        invoke: paths,
    };
    let mut host = std::ptr::null_mut();
    assert_eq!(
        unsafe { sdk_host::vc_interface_create_host(&backend, &mut host) },
        STATUS_OK
    );
    let paths = unsafe { crate::runtime::runtime_paths(&*host) }.unwrap();
    assert_eq!(paths.resources_root.to_str().unwrap(), "/pkg/resources");
    assert_eq!(
        unsafe { sdk_host::vc_interface_destroy_host(host) },
        STATUS_OK
    );
}
unsafe extern "C" fn duplicate(
    _: HostContext,
    _: *const HostRequest,
    sink: ReplySink,
    out: *mut c_void,
) -> i32 {
    unsafe {
        sdk::write_host_response(&[], sink, out);
        sdk::write_host_response(&[], sink, out);
    }
    STATUS_OK
}
#[test]
fn duplicate_replies_fail_the_sync_call() {
    let host = HostApi {
        abi_version: ABI_VERSION,
        struct_size: size_of::<HostApi>() as u32,
        generation: 1,
        context: std::ptr::null_mut(),
        invoke: duplicate,
    };
    assert!(
        unsafe {
            sdk::call_host(
                &host,
                &proto::HostRequest {
                    service: "custom".into(),
                    ..Default::default()
                },
            )
        }
        .is_err()
    );
}
#[test]
fn null_callback_table_is_rejected_before_reading_function_pointer() {
    let raw = [0usize; size_of::<HostApi>() / size_of::<usize>()];
    let mut raw = raw;
    unsafe {
        raw.as_mut_ptr()
            .cast::<Header>()
            .write(Header::of::<HostApi>());
    }
    assert!(unsafe { sdk::read_host_api(raw.as_ptr().cast()) }.is_err());
}

unsafe extern "C" fn stale(
    _: HostContext,
    _: *const HostRequest,
    sink: ReplySink,
    out: *mut c_void,
) -> i32 {
    unsafe { sdk::write_error(STATUS_STALE, "旧代次", sink, out) }
}
#[test]
fn error_reply_is_accepted_and_status_preserved() {
    let host = HostApi {
        abi_version: ABI_VERSION,
        struct_size: size_of::<HostApi>() as u32,
        generation: 1,
        context: std::ptr::null_mut(),
        invoke: stale,
    };
    let error = unsafe {
        sdk::call_host(
            &host,
            &proto::HostRequest {
                service: "custom".into(),
                ..Default::default()
            },
        )
    }
    .unwrap_err();
    assert!(error.contains("2"));
}

#[test]
fn four_byte_aligned_tables_are_rejected_before_wide_reads() {
    let mut words = [0u64; 8];
    let pointer = unsafe { words.as_mut_ptr().cast::<u8>().add(4) };
    assert_eq!(pointer as usize % 4, 0);
    assert_ne!(pointer as usize % std::mem::align_of::<HostApi>(), 0);
    unsafe {
        pointer.cast::<Header>().write(Header::of::<HostApi>());
    }
    assert!(unsafe { sdk::read_host_api(pointer.cast()) }.is_err());
}

#[test]
fn incoming_source_message_id_preserves_missing_null_and_empty() {
    for value in [
        None,
        Some(Value::Null),
        Some(json!("")),
        Some(json!("remote-1")),
    ] {
        let mut payload =
            json!({"username":"来源","document":crate::model::Document::plain("独立消息")});
        if let Some(value) = value {
            payload["source_message_id"] = value;
        }
        payload_roundtrip("message.post", payload);
    }
}
#[test]
fn standard_plugin_operations_cannot_use_extension_payloads() {
    for operation in [
        "transform",
        "event",
        "message",
        "clear",
        "receipt",
        "frontend.message",
        "frontend.http",
        "frontend.ws.open",
    ] {
        let mut request = proto::PluginRequest {
            operation: "custom".into(),
            payload_json: br#"{}"#.to_vec(),
            ..Default::default()
        };
        sdk::with_request(&request, |view| {
            let mut forged = unsafe { *view };
            forged.operation = StringView::borrow(operation);
            assert!(
                unsafe { sdk::read_request(&forged) }.is_err(),
                "{operation}"
            );
        })
        .unwrap();
        request.operation = "custom.route".into();
        request.payload_json = br#"{"version":1,"method":"GET","path":"/custom"}"#.to_vec();
        sdk::with_request(&request, |view| {
            assert!(unsafe { sdk::read_request(view) }.is_ok())
        })
        .unwrap();
    }
}

#[test]
fn plugin_tables_reject_short_misaligned_and_missing_slots() {
    let short = Header::of::<Header>();
    assert!(unsafe { sdk::read_plugin_api((&short as *const Header).cast()) }.is_err());
    let mut words = [0u64; 16];
    let pointer = words.as_mut_ptr().cast::<PluginApi>();
    unsafe {
        pointer.cast::<Header>().write(Header::of::<PluginApi>());
    }
    assert!(unsafe { sdk::read_plugin_api(pointer) }.is_err());
    words[1..=8].fill(1);
    // 每个缺失槽均应在复制为 Rust 函数指针前返回错误。
    for index in [1, 2, 3, 4, 5, 6, 7, 8] {
        words[index] = 0;
        assert!(unsafe { sdk::read_plugin_api(words.as_ptr().cast()) }.is_err());
        words[index] = 1;
    }
    let misaligned = unsafe { words.as_mut_ptr().cast::<u8>().add(4) };
    unsafe {
        misaligned.cast::<Header>().write(Header::of::<PluginApi>());
    }
    assert!(unsafe { sdk::read_plugin_api(misaligned.cast()) }.is_err());
}

#[test]
fn complete_message_preserves_optional_source_id_through_abi() {
    for source_message_id in [None, Some(String::new()), Some("remote-1".to_owned())] {
        let mut message = crate::model::Message::new(
            "upstream",
            "来源",
            crate::model::Document::plain("独立消息"),
            123,
            4,
        );
        message.source_message_id = source_message_id.clone();
        let typed = crate::document::message_to_proto(&message);
        #[cfg(feature = "host-protocol")]
        {
            use prost::Message;
            let wire = typed.encode_to_vec();
            let decoded = proto::Message::decode(wire.as_slice()).unwrap();
            assert_eq!(decoded.source_message_id, source_message_id);
        }
        let request = proto::PluginRequest {
            protocol_version: crate::BUSINESS_MODEL_VERSION,
            operation: "transform".into(),
            payload_json: serde_json::to_vec(&message).unwrap(),
            document: Some(crate::document::document_to_proto(&message.document)),
            message: Some(typed),
            ..Default::default()
        };
        let copy = sdk::with_request(&request, |view| unsafe { sdk::read_request(view) }.unwrap())
            .unwrap();
        assert_eq!(copy.message.unwrap().source_message_id, source_message_id);
        let decoded: crate::model::Message = serde_json::from_slice(&copy.payload_json).unwrap();
        assert_eq!(decoded, message);
        payload_roundtrip(
            "frontend.message",
            json!({"version":1,"frontend_id":"sample","batch_id":"batch","message":message}),
        );
    }
}
