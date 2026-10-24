//! 标准业务载荷的集中投影，C 边界只传递结构化字段与显式扩展。
use crate::{document, proto as m};
use serde_json::{Map, Value, json};
type Result<T> = std::result::Result<T, String>;
fn object(bytes: &[u8]) -> Result<Map<String, Value>> {
    if bytes.is_empty() {
        return Ok(Map::new());
    }
    let v: Value = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
    v.as_object().cloned().ok_or("业务载荷必须为对象".into())
}
fn take<T: serde::de::DeserializeOwned + Default>(
    o: &mut Map<String, Value>,
    key: &str,
) -> Result<T> {
    match o.remove(key) {
        None | Some(Value::Null) => Ok(T::default()),
        Some(v) => serde_json::from_value(v).map_err(|e| format!("{key}: {e}")),
    }
}
fn bytes(o: Map<String, Value>) -> Result<Vec<u8>> {
    serde_json::to_vec(&o).map_err(|e| e.to_string())
}
fn restore<T: serde::Serialize>(value: &T, present: &[String], extra: &[u8]) -> Result<Vec<u8>> {
    let mut out = object(extra)?;
    let value = serde_json::to_value(value).map_err(|e| e.to_string())?;
    if present.is_empty() {
        if let Some(fields) = value.as_object() {
            for (key, v) in fields {
                if !matches!(key.as_str(), "present" | "extra_json" | "data_json") {
                    out.insert(key.clone(), v.clone());
                }
            }
        }
    } else {
        for key in present {
            if let Some(v) = value.get(key) {
                out.insert(key.clone(), v.clone());
            }
        }
    }
    bytes(out)
}
fn core_document(value: Value) -> Result<m::Document> {
    let doc: vchat_core::Document = serde_json::from_value(value).map_err(|e| e.to_string())?;
    doc.validate().map_err(|e| e.to_string())?;
    Ok(document::document_to_proto(&doc))
}
fn core_message(value: Value) -> Result<m::Message> {
    let message: vchat_core::Message = serde_json::from_value(value).map_err(|e| e.to_string())?;
    Ok(document::message_to_proto(&message))
}
fn message_value(v: &m::Message) -> Result<Value> {
    Ok(
        json!({"id":v.id,"source_id":v.source_id,"source_message_id":v.source_message_id,
    "received_at":v.received_at_ms,"sequence":v.sequence,"document":document::document_from_proto(v.document.as_ref().ok_or("消息缺少文档")?)?,
    "source_time":v.source_time,"username":v.username,"status":v.status,"origin_ids":v.origin_ids,"message_type":v.message_type,"avatar_url":v.avatar_url}),
    )
}
pub fn message_input(input: &[u8]) -> Result<m::MessageInput> {
    let mut o = object(input)?;
    let present = o.keys().cloned().collect();
    let doc = o.remove("document").map(core_document).transpose()?;
    Ok(m::MessageInput {
        source_id: take(&mut o, "source_id")?,
        source_message_id: take(&mut o, "source_message_id")?,
        username: take(&mut o, "username")?,
        source_time: take(&mut o, "source_time")?,
        message_type: take(&mut o, "message_type")?,
        avatar_url: take(&mut o, "avatar_url")?,
        origin_ids: take(&mut o, "origin_ids")?,
        document: doc,
        present,
        extra_json: bytes(o)?,
    })
}
pub fn message_input_bytes(v: &m::MessageInput) -> Result<Vec<u8>> {
    let mut value = serde_json::to_value(v).map_err(|e| e.to_string())?;
    if let Some(doc) = &v.document {
        value["document"] =
            serde_json::to_value(document::document_from_proto(doc)?).map_err(|e| e.to_string())?;
    }
    restore(&value, &v.present, &v.extra_json)
}
pub fn frontend(input: &[u8]) -> Result<m::FrontendPayload> {
    let mut o = object(input)?;
    let present = o.keys().cloned().collect();
    let message = o
        .remove("message")
        .filter(|v| !v.is_null())
        .map(core_message)
        .transpose()?;
    Ok(m::FrontendPayload {
        version: take(&mut o, "version")?,
        frontend_id: take(&mut o, "frontend_id")?,
        message_id: take(&mut o, "message_id")?,
        batch_id: take(&mut o, "batch_id")?,
        run_id: take(&mut o, "run_id")?,
        generation: take(&mut o, "generation")?,
        label: take(&mut o, "label")?,
        kind: take(&mut o, "kind")?,
        status: take(&mut o, "status")?,
        capabilities: take(&mut o, "capabilities")?,
        message,
        present,
        extra_json: bytes(o)?,
    })
}
pub fn frontend_bytes(v: &m::FrontendPayload) -> Result<Vec<u8>> {
    let mut value = serde_json::to_value(v).map_err(|e| e.to_string())?;
    if let Some(message) = &v.message {
        value["message"] = message_value(message)?;
    }
    restore(&value, &v.present, &v.extra_json)
}
pub fn web(input: &[u8]) -> Result<m::WebPayload> {
    let mut o = object(input)?;
    let present = o.keys().cloned().collect();
    let data_json = o
        .remove("data")
        .map(|v| serde_json::to_vec(&v).map_err(|e| e.to_string()))
        .transpose()?
        .unwrap_or_default();
    let headers = match o.remove("headers") {
        None => Vec::new(),
        Some(Value::Object(values)) => values
            .into_iter()
            .map(|(name, value)| {
                Ok(m::HeaderPair {
                    name,
                    value: value.as_str().ok_or("header 值必须为文本")?.into(),
                })
            })
            .collect::<Result<Vec<_>>>()?,
        Some(v) => serde_json::from_value(v).map_err(|e| e.to_string())?,
    };
    Ok(m::WebPayload {
        version: take(&mut o, "version")?,
        method: take(&mut o, "method")?,
        path: take(&mut o, "path")?,
        query: take(&mut o, "query")?,
        connection_id: take(&mut o, "connection_id")?,
        preview: take(&mut o, "preview")?,
        headers,
        body: take(&mut o, "body")?,
        routes: take(&mut o, "routes")?,
        data_json,
        present,
        extra_json: bytes(o)?,
    })
}
pub fn web_bytes(v: &m::WebPayload) -> Result<Vec<u8>> {
    let mut value = serde_json::to_value(v).map_err(|e| e.to_string())?;
    value["headers"] = Value::Object(
        v.headers
            .iter()
            .map(|h| (h.name.clone(), Value::String(h.value.clone())))
            .collect(),
    );
    if !v.data_json.is_empty() {
        value["data"] = serde_json::from_slice(&v.data_json).map_err(|e| e.to_string())?;
    }
    restore(&value, &v.present, &v.extra_json)
}
pub fn event(input: &[u8]) -> Result<m::EventPayload> {
    let mut o = object(input)?;
    let message = o.remove("message").map(core_message).transpose()?;
    let data_json = o
        .remove("data")
        .map(|v| serde_json::to_vec(&v).map_err(|e| e.to_string()))
        .transpose()?
        .unwrap_or_default();
    Ok(m::EventPayload {
        version: take(&mut o, "version")?,
        event_id: take(&mut o, "event_id")?,
        kind: take(&mut o, "kind")?,
        message,
        data_json,
    })
}
pub fn event_bytes(v: &m::EventPayload) -> Result<Vec<u8>> {
    let mut value = json!({"version":v.version,"event_id":v.event_id,"kind":v.kind});
    if let Some(message) = &v.message {
        value["message"] = message_value(message)?;
    }
    if !v.data_json.is_empty() {
        value["data"] = serde_json::from_slice(&v.data_json).map_err(|e| e.to_string())?;
    }
    serde_json::to_vec(&value).map_err(|e| e.to_string())
}
pub fn transform(
    input: &[u8],
    fallback: &Option<m::Message>,
    doc: &Option<m::Document>,
) -> Result<m::TransformPayload> {
    let mut o = object(input)?;
    let wrapped = o.contains_key("message");
    let message = if wrapped {
        o.remove("message").map(core_message).transpose()?
    } else if o.contains_key("id") {
        Some(core_message(Value::Object(o.clone()))?)
    } else {
        fallback.clone()
    };
    Ok(m::TransformPayload {
        version: take(&mut o, "version")?,
        message,
        document: doc.clone(),
        wrapped,
    })
}
pub fn transform_bytes(v: &m::TransformPayload) -> Result<Vec<u8>> {
    let message = v.message.as_ref().map(message_value).transpose()?;
    let value = if v.wrapped {
        json!({"version":v.version,"message":message,"document":v.document.as_ref().map(document::document_from_proto).transpose()?})
    } else if let Some(message) = message {
        message
    } else {
        json!({"document":v.document.as_ref().map(document::document_from_proto).transpose()?})
    };
    serde_json::to_vec(&value).map_err(|e| e.to_string())
}
pub fn window(input: &[u8]) -> Result<m::WindowPayload> {
    let mut o = object(input)?;
    let content_json = o
        .remove("content")
        .map(|v| serde_json::to_vec(&v).map_err(|e| e.to_string()))
        .transpose()?
        .unwrap_or_default();
    Ok(m::WindowPayload {
        version: take(&mut o, "version")?,
        window_id: take(&mut o, "window_id")?,
        title: take(&mut o, "title")?,
        content_json,
        properties_json: bytes(o)?,
    })
}
pub fn window_bytes(v: &m::WindowPayload) -> Result<Vec<u8>> {
    let mut o = object(&v.properties_json)?;
    if v.version != 0 {
        o.insert("version".into(), json!(v.version));
    }
    if !v.window_id.is_empty() {
        o.insert("window_id".into(), json!(v.window_id));
    }
    if !v.title.is_empty() {
        o.insert("title".into(), json!(v.title));
    }
    if !v.content_json.is_empty() {
        o.insert(
            "content".into(),
            serde_json::from_slice(&v.content_json).map_err(|e| e.to_string())?,
        );
    }
    bytes(o)
}
