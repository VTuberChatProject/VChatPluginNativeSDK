//! 标准宿主服务与插件操作的强类型载荷，JSON 仅保存显式扩展。
use crate::proto::{Document, Message};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MessageInput {
    pub source_id: String,
    pub source_message_id: Option<String>,
    pub username: String,
    pub source_time: Option<i64>,
    pub message_type: Option<String>,
    pub avatar_url: Option<String>,
    pub origin_ids: Vec<String>,
    pub document: Option<Document>,
    pub present: Vec<String>,
    pub extra_json: Vec<u8>,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MessageTypeDefinition {
    pub id: String,
    pub label: String,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MessageTypes {
    pub version: u32,
    pub types: Vec<MessageTypeDefinition>,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct RuntimePaths {
    pub package_root: String,
    pub resources_root: String,
    pub data_root: String,
    pub assets_root: Option<String>,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct FrontendPayload {
    pub version: u32,
    pub frontend_id: String,
    pub message_id: String,
    pub batch_id: String,
    pub run_id: String,
    pub generation: Option<u64>,
    pub label: String,
    pub kind: String,
    pub status: String,
    pub capabilities: Vec<String>,
    pub message: Option<Message>,
    pub present: Vec<String>,
    pub extra_json: Vec<u8>,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct HeaderPair {
    pub name: String,
    pub value: String,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct WebRoute {
    pub kind: String,
    pub method: String,
    pub path: String,
    pub operation: String,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct WebPayload {
    pub version: u32,
    pub method: String,
    pub path: String,
    pub query: String,
    pub connection_id: String,
    pub preview: bool,
    pub headers: Vec<HeaderPair>,
    pub body: Vec<u8>,
    pub routes: Vec<WebRoute>,
    pub data_json: Vec<u8>,
    pub present: Vec<String>,
    pub extra_json: Vec<u8>,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct EventPayload {
    pub version: u32,
    pub event_id: String,
    pub kind: String,
    pub message: Option<Message>,
    pub data_json: Vec<u8>,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct LogPayload {
    pub message: String,
    pub level: String,
    pub context_json: Vec<u8>,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Subscription {
    pub events: Vec<String>,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct WindowPayload {
    pub version: u32,
    pub window_id: String,
    pub title: String,
    pub content_json: Vec<u8>,
    pub properties_json: Vec<u8>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct TransformPayload {
    pub version: u32,
    pub message: Option<Message>,
    pub document: Option<Document>,
    pub wrapped: bool,
}
