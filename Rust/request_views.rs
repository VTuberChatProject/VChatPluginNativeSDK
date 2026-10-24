//! 标准请求采用类型化载荷，JSON 字段仅用于配置、声明式 schema 及自定义扩展。
use crate::{abi::*, memory::*, projection as p, proto as m};
use std::ptr;
#[repr(C)]
#[derive(Clone, Copy)]
pub struct HostRequest {
    pub header: Header,
    pub kind: u32,
    pub service: StringView,
    pub payload_json: BytesView,
    pub ui: *const UiContribution,
    pub event_id: StringView,
    pub message: *const MessageInput,
    pub message_types: *const MessageTypes,
    pub frontend: *const FrontendPayload,
    pub web: *const WebPayload,
    pub log: *const LogPayload,
    pub subscription: *const Subscription,
    pub window: *const WindowPayload,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct PluginRequest {
    pub header: Header,
    pub kind: u32,
    pub protocol_version: u32,
    pub operation: StringView,
    pub payload_json: BytesView,
    pub document: *const Document,
    pub selection: *const Selection,
    pub event_id: StringView,
    pub message: *const Message,
    pub frontend: *const FrontendPayload,
    pub web: *const WebPayload,
    pub event: *const EventPayload,
    pub transform: *const TransformPayload,
}
fn keep<T: Value + 'static>(v: T, a: &mut Arena) -> Result<*const T::Ffi, String> {
    let owned = a.keep(v);
    let view = unsafe { (&*owned).view(a)? };
    Ok(a.keep(view))
}
unsafe fn owned<T: Value>(v: *const T::Ffi, b: &mut Budget) -> Result<T, String> {
    unsafe { T::read_ptr(v, b) }
}
fn json<T: serde::de::DeserializeOwned>(input: &[u8]) -> Result<T, String> {
    serde_json::from_slice(input).map_err(|e| e.to_string())
}
fn out<T: serde::Serialize>(v: &T) -> Result<Vec<u8>, String> {
    serde_json::to_vec(v).map_err(|e| e.to_string())
}
impl Value for m::HostRequest {
    type Ffi = HostRequest;
    fn view(&self, a: &mut Arena) -> Result<HostRequest, String> {
        if self.payload_json.len() > MAX_BYTES {
            return Err("宿主载荷超过限额".into());
        }
        let mut r = HostRequest {
            header: Header::of::<HostRequest>(),
            kind: 0,
            service: self.service.view(a)?,
            payload_json: BytesView::borrow(&[]),
            ui: self.ui.view(a)?,
            event_id: self.event_id.view(a)?,
            message: ptr::null(),
            message_types: ptr::null(),
            frontend: ptr::null(),
            web: ptr::null(),
            log: ptr::null(),
            subscription: ptr::null(),
            window: ptr::null(),
        };
        match self.service.as_str() {
            "message.post" => {
                r.kind = 1;
                r.message = keep(p::message_input(&self.payload_json)?, a)?;
            }
            "message.types.register" => {
                r.kind = 2;
                r.message_types = keep(json::<m::MessageTypes>(&self.payload_json)?, a)?;
            }
            "ui.register" => r.kind = 3,
            s if s.starts_with("frontend.") => {
                r.kind = 4;
                r.frontend = keep(p::frontend(&self.payload_json)?, a)?;
            }
            "web.route.register" | "websocket.send" | "websocket.broadcast" => {
                r.kind = 5;
                r.web = keep(p::web(&self.payload_json)?, a)?;
            }
            "runtime.paths" => r.kind = 6,
            "log" => {
                r.kind = 7;
                let value =
                    String::from_utf8(self.payload_json.clone()).map_err(|e| e.to_string())?;
                let log = match serde_json::from_slice::<serde_json::Value>(&self.payload_json) {
                    Ok(v) => m::LogPayload {
                        message: v["message"].as_str().unwrap_or(&value).into(),
                        level: v["level"].as_str().unwrap_or("info").into(),
                        context_json: self.payload_json.clone(),
                    },
                    Err(_) => m::LogPayload {
                        message: value,
                        level: "info".into(),
                        context_json: Vec::new(),
                    },
                };
                r.log = keep(log, a)?;
            }
            "event.subscribe" => {
                r.kind = 8;
                r.subscription = keep(json::<m::Subscription>(&self.payload_json)?, a)?;
            }
            s if s.starts_with("ui.window.") => {
                r.kind = 9;
                r.window = keep(p::window(&self.payload_json)?, a)?;
            }
            _ => r.payload_json = self.payload_json.view(a)?,
        }
        Ok(r)
    }
    unsafe fn read(v: HostRequest, b: &mut Budget) -> Result<Self, String> {
        header::<HostRequest>(v.header)?;
        let service = unsafe { String::read(v.service, b)? };
        let payload_json = match v.kind {
            0 => unsafe { Vec::<u8>::read(v.payload_json, b)? },
            1 => p::message_input_bytes(&unsafe { owned::<m::MessageInput>(v.message, b)? })?,
            2 => out(&unsafe { owned::<m::MessageTypes>(v.message_types, b)? })?,
            3 | 6 => Vec::new(),
            4 => p::frontend_bytes(&unsafe { owned::<m::FrontendPayload>(v.frontend, b)? })?,
            5 => p::web_bytes(&unsafe { owned::<m::WebPayload>(v.web, b)? })?,
            7 => {
                let log = unsafe { owned::<m::LogPayload>(v.log, b)? };
                if log.context_json.is_empty() {
                    log.message.into_bytes()
                } else {
                    log.context_json
                }
            }
            8 => out(&unsafe { owned::<m::Subscription>(v.subscription, b)? })?,
            9 => p::window_bytes(&unsafe { owned::<m::WindowPayload>(v.window, b)? })?,
            _ => return Err("宿主请求种类无效".into()),
        };
        let expected = match service.as_str() {
            "message.post" => 1,
            "message.types.register" => 2,
            "ui.register" => 3,
            s if s.starts_with("frontend.") => 4,
            "web.route.register" | "websocket.send" | "websocket.broadcast" => 5,
            "runtime.paths" => 6,
            "log" => 7,
            "event.subscribe" => 8,
            s if s.starts_with("ui.window.") => 9,
            _ => 0,
        };
        if expected != v.kind {
            return Err("宿主请求种类与服务不匹配".into());
        }
        if v.kind != 0 && v.payload_json.len != 0 {
            return Err("标准服务必须使用结构化载荷".into());
        }
        Ok(Self {
            service,
            payload_json,
            ui: unsafe { Option::<m::UiContribution>::read(v.ui, b)? },
            event_id: unsafe { String::read(v.event_id, b)? },
        })
    }
}
impl Value for m::PluginRequest {
    type Ffi = PluginRequest;
    fn view(&self, a: &mut Arena) -> Result<PluginRequest, String> {
        if self.payload_json.len() > MAX_BYTES {
            return Err("插件载荷超过限额".into());
        }
        let mut r = PluginRequest {
            header: Header::of::<PluginRequest>(),
            kind: 0,
            protocol_version: self.protocol_version,
            operation: self.operation.view(a)?,
            payload_json: BytesView::borrow(&[]),
            document: self.document.view(a)?,
            selection: self.selection.view(a)?,
            event_id: self.event_id.view(a)?,
            message: self.message.view(a)?,
            frontend: ptr::null(),
            web: ptr::null(),
            event: ptr::null(),
            transform: ptr::null(),
        };
        match self.operation.as_str() {
            "transform" => {
                r.kind = 1;
                r.transform = keep(
                    p::transform(&self.payload_json, &self.message, &self.document)?,
                    a,
                )?;
            }
            "event" => {
                r.kind = 2;
                r.event = keep(p::event(&self.payload_json)?, a)?;
            }
            "message" | "clear" | "receipt" | "frontend.message" | "frontend.clear"
            | "frontend.receipt" => {
                r.kind = 3;
                r.frontend = keep(p::frontend(&self.payload_json)?, a)?;
            }
            s if s.starts_with("frontend.http")
                || s.starts_with("frontend.ws.")
                || s.starts_with("web.http")
                || s.starts_with("web.ws.") =>
            {
                r.kind = 4;
                r.web = keep(p::web(&self.payload_json)?, a)?;
            }
            _ if serde_json::from_slice::<serde_json::Value>(&self.payload_json)
                .ok()
                .is_some_and(|p| {
                    p.get("path").is_some()
                        && (p.get("method").is_some() || p.get("connection_id").is_some())
                }) =>
            {
                r.kind = 4;
                r.web = keep(p::web(&self.payload_json)?, a)?;
            }
            _ => r.payload_json = self.payload_json.view(a)?,
        }
        Ok(r)
    }
    unsafe fn read(v: PluginRequest, b: &mut Budget) -> Result<Self, String> {
        header::<PluginRequest>(v.header)?;
        let operation = unsafe { String::read(v.operation, b)? };
        let expected = match operation.as_str() {
            "transform" => Some(1),
            "event" => Some(2),
            "message" | "clear" | "receipt" | "frontend.message" | "frontend.clear"
            | "frontend.receipt" => Some(3),
            s if s.starts_with("frontend.http")
                || s.starts_with("frontend.ws.")
                || s.starts_with("web.http")
                || s.starts_with("web.ws.") =>
            {
                Some(4)
            }
            _ => None,
        };
        // 自定义路由可使用 Web 分支；其余自定义操作使用扩展分支。
        if expected.is_some_and(|kind| kind != v.kind)
            || (expected.is_none() && !matches!(v.kind, 0 | 4))
        {
            return Err("插件请求种类与操作不匹配".into());
        }
        let payload_json = match v.kind {
            0 => unsafe { Vec::<u8>::read(v.payload_json, b)? },
            1 => p::transform_bytes(&unsafe { owned::<m::TransformPayload>(v.transform, b)? })?,
            2 => p::event_bytes(&unsafe { owned::<m::EventPayload>(v.event, b)? })?,
            3 => p::frontend_bytes(&unsafe { owned::<m::FrontendPayload>(v.frontend, b)? })?,
            4 => p::web_bytes(&unsafe { owned::<m::WebPayload>(v.web, b)? })?,
            _ => return Err("插件请求种类无效".into()),
        };
        if v.kind != 0 && v.payload_json.len != 0 {
            return Err("标准插件调用必须使用结构化载荷".into());
        }
        Ok(Self {
            protocol_version: v.protocol_version,
            operation,
            payload_json,
            document: unsafe { Option::<m::Document>::read(v.document, b)? },
            selection: unsafe { Option::<m::Selection>::read(v.selection, b)? },
            event_id: unsafe { String::read(v.event_id, b)? },
            message: unsafe { Option::<m::Message>::read(v.message, b)? },
        })
    }
}
