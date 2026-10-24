//! 插件拥有的业务数据；Protobuf 编码仅在宿主协议 feature 中启用。
/// 核心文档为结构化字段；JSON 保存向前兼容扩展。
#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq)]
#[cfg_attr(feature = "host-protocol", derive(::prost::Message))]
#[cfg_attr(not(feature = "host-protocol"), derive(Debug, Default))]
pub struct Document {
    #[cfg_attr(feature = "host-protocol", prost(uint32, tag = "1"))]
    pub version: u32,
    #[cfg_attr(feature = "host-protocol", prost(uint64, tag = "2"))]
    pub revision: u64,
    #[cfg_attr(feature = "host-protocol", prost(bytes = "vec", tag = "3"))]
    pub json: Vec<u8>,
    #[cfg_attr(feature = "host-protocol", prost(message, repeated, tag = "4"))]
    pub paragraphs: Vec<Paragraph>,
}
#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq)]
#[cfg_attr(feature = "host-protocol", derive(::prost::Message))]
#[cfg_attr(not(feature = "host-protocol"), derive(Debug, Default))]
pub struct Paragraph {
    #[cfg_attr(feature = "host-protocol", prost(message, repeated, tag = "1"))]
    pub segments: Vec<Segment>,
}
#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq)]
#[cfg_attr(feature = "host-protocol", derive(::prost::Message))]
#[cfg_attr(not(feature = "host-protocol"), derive(Debug, Default))]
pub struct Segment {
    #[cfg_attr(feature = "host-protocol", prost(string, tag = "1"))]
    pub text: String,
    #[cfg_attr(feature = "host-protocol", prost(message, optional, tag = "2"))]
    pub style: Option<TextStyle>,
    #[cfg_attr(feature = "host-protocol", prost(string, optional, tag = "3"))]
    pub ruby: Option<String>,
    #[cfg_attr(feature = "host-protocol", prost(uint32, optional, tag = "4"))]
    pub speed: Option<u32>,
    #[cfg_attr(feature = "host-protocol", prost(uint32, optional, tag = "5"))]
    pub pause: Option<u32>,
    #[cfg_attr(feature = "host-protocol", prost(message, optional, tag = "6"))]
    pub image: Option<Image>,
    #[cfg_attr(feature = "host-protocol", prost(string, optional, tag = "7"))]
    pub class: Option<String>,
    #[cfg_attr(feature = "host-protocol", prost(bytes = "vec", tag = "8"))]
    pub extra_json: Vec<u8>,
}
#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "host-protocol", derive(::prost::Message))]
#[cfg_attr(not(feature = "host-protocol"), derive(Debug, Default))]
pub struct TextStyle {
    #[cfg_attr(feature = "host-protocol", prost(bool, tag = "1"))]
    pub bold: bool,
    #[cfg_attr(feature = "host-protocol", prost(bool, tag = "2"))]
    pub italic: bool,
    #[cfg_attr(feature = "host-protocol", prost(bool, tag = "3"))]
    pub underline: bool,
    #[cfg_attr(feature = "host-protocol", prost(bool, tag = "4"))]
    pub strikethrough: bool,
    #[cfg_attr(feature = "host-protocol", prost(string, optional, tag = "5"))]
    pub color: Option<String>,
    #[cfg_attr(feature = "host-protocol", prost(string, optional, tag = "6"))]
    pub background_color: Option<String>,
    #[cfg_attr(feature = "host-protocol", prost(string, optional, tag = "7"))]
    pub size: Option<String>,
    #[cfg_attr(feature = "host-protocol", prost(bytes = "vec", tag = "8"))]
    pub extra_json: Vec<u8>,
}
#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq)]
#[cfg_attr(feature = "host-protocol", derive(::prost::Message))]
#[cfg_attr(not(feature = "host-protocol"), derive(Debug, Default))]
pub struct Image {
    #[cfg_attr(feature = "host-protocol", prost(string, tag = "1"))]
    pub source: String,
    #[cfg_attr(feature = "host-protocol", prost(float, tag = "2"))]
    pub width: f32,
    #[cfg_attr(feature = "host-protocol", prost(float, tag = "3"))]
    pub height: f32,
}
#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq)]
#[cfg_attr(feature = "host-protocol", derive(::prost::Message))]
#[cfg_attr(not(feature = "host-protocol"), derive(Debug, Default))]
pub struct Message {
    #[cfg_attr(feature = "host-protocol", prost(string, tag = "1"))]
    pub id: String,
    #[cfg_attr(feature = "host-protocol", prost(string, tag = "2"))]
    pub source_id: String,
    #[cfg_attr(feature = "host-protocol", prost(string, optional, tag = "3"))]
    pub source_message_id: Option<String>,
    #[cfg_attr(feature = "host-protocol", prost(int64, tag = "4"))]
    pub received_at_ms: i64,
    #[cfg_attr(feature = "host-protocol", prost(uint64, tag = "5"))]
    pub sequence: u64,
    #[cfg_attr(feature = "host-protocol", prost(message, optional, tag = "6"))]
    pub document: Option<Document>,
    #[cfg_attr(feature = "host-protocol", prost(int64, optional, tag = "7"))]
    pub source_time: Option<i64>,
    #[cfg_attr(feature = "host-protocol", prost(string, tag = "8"))]
    pub username: String,
    #[cfg_attr(feature = "host-protocol", prost(string, tag = "9"))]
    pub status: String,
    #[cfg_attr(feature = "host-protocol", prost(string, repeated, tag = "10"))]
    pub origin_ids: Vec<String>,
    #[cfg_attr(feature = "host-protocol", prost(message, optional, tag = "11"))]
    pub message_type: Option<MessageType>,
    #[cfg_attr(feature = "host-protocol", prost(string, optional, tag = "12"))]
    pub avatar_url: Option<String>,
}
/// 类型归属由宿主鉴权后生成，插件提交时仅使用插件内 type_id。
#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "host-protocol", derive(::prost::Message))]
#[cfg_attr(not(feature = "host-protocol"), derive(Debug, Default))]
pub struct MessageType {
    #[cfg_attr(feature = "host-protocol", prost(string, tag = "1"))]
    pub plugin_kind: String,
    #[cfg_attr(feature = "host-protocol", prost(string, tag = "2"))]
    pub plugin_id: String,
    #[cfg_attr(feature = "host-protocol", prost(string, tag = "3"))]
    pub type_id: String,
}
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "host-protocol", derive(::prost::Message))]
#[cfg_attr(not(feature = "host-protocol"), derive(Debug, Default))]
pub struct Selection {
    #[cfg_attr(feature = "host-protocol", prost(uint64, tag = "1"))]
    pub start: u64,
    #[cfg_attr(feature = "host-protocol", prost(uint64, tag = "2"))]
    pub end: u64,
}
#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq)]
#[cfg_attr(feature = "host-protocol", derive(::prost::Message))]
#[cfg_attr(not(feature = "host-protocol"), derive(Debug, Default))]
pub struct PluginRequest {
    #[cfg_attr(feature = "host-protocol", prost(uint32, tag = "1"))]
    /// 业务模型版本，当前为 1；ABI 版本由函数表声明。
    pub protocol_version: u32,
    #[cfg_attr(feature = "host-protocol", prost(string, tag = "2"))]
    pub operation: String,
    #[cfg_attr(feature = "host-protocol", prost(bytes = "vec", tag = "3"))]
    pub payload_json: Vec<u8>,
    #[cfg_attr(feature = "host-protocol", prost(message, optional, tag = "4"))]
    pub document: Option<Document>,
    #[cfg_attr(feature = "host-protocol", prost(message, optional, tag = "5"))]
    pub selection: Option<Selection>,
    #[cfg_attr(feature = "host-protocol", prost(string, tag = "6"))]
    pub event_id: String,
    #[cfg_attr(feature = "host-protocol", prost(message, optional, tag = "7"))]
    pub message: Option<Message>,
}
#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "host-protocol", derive(::prost::Message))]
#[cfg_attr(not(feature = "host-protocol"), derive(Debug, Default))]
pub struct PluginError {
    #[cfg_attr(feature = "host-protocol", prost(string, tag = "1"))]
    pub code: String,
    #[cfg_attr(feature = "host-protocol", prost(string, tag = "2"))]
    pub message: String,
}
#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq)]
#[cfg_attr(feature = "host-protocol", derive(::prost::Message))]
#[cfg_attr(not(feature = "host-protocol"), derive(Debug, Default))]
pub struct DocumentEdit {
    #[cfg_attr(feature = "host-protocol", prost(uint64, tag = "1"))]
    pub expected_revision: u64,
    #[cfg_attr(feature = "host-protocol", prost(message, optional, tag = "2"))]
    pub replacement: Option<Document>,
}
#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq)]
#[cfg_attr(feature = "host-protocol", derive(::prost::Message))]
#[cfg_attr(not(feature = "host-protocol"), derive(Debug, Default))]
pub struct PluginResponse {
    #[cfg_attr(feature = "host-protocol", prost(bytes = "vec", tag = "1"))]
    pub payload_json: Vec<u8>,
    #[cfg_attr(feature = "host-protocol", prost(message, optional, tag = "2"))]
    pub error: Option<PluginError>,
    #[cfg_attr(feature = "host-protocol", prost(message, optional, tag = "3"))]
    pub edit: Option<DocumentEdit>,
    #[cfg_attr(feature = "host-protocol", prost(message, optional, tag = "4"))]
    pub ui: Option<UiContribution>,
    /// HTTP route response fields; status 0 uses 200 and empty content_type uses octet-stream.
    #[cfg_attr(feature = "host-protocol", prost(uint32, tag = "5"))]
    pub status: u32,
    #[cfg_attr(feature = "host-protocol", prost(string, tag = "6"))]
    pub content_type: String,
    #[cfg_attr(feature = "host-protocol", prost(bool, tag = "7"))]
    pub no_store: bool,
    #[cfg_attr(feature = "host-protocol", prost(bytes = "vec", tag = "8"))]
    pub body: Vec<u8>,
}
#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "host-protocol", derive(::prost::Message))]
#[cfg_attr(not(feature = "host-protocol"), derive(Debug, Default))]
pub struct UiCommand {
    #[cfg_attr(feature = "host-protocol", prost(string, tag = "1"))]
    pub id: String,
    #[cfg_attr(feature = "host-protocol", prost(string, tag = "2"))]
    pub label: String,
    #[cfg_attr(feature = "host-protocol", prost(string, tag = "3"))]
    pub icon: String,
    #[cfg_attr(feature = "host-protocol", prost(string, tag = "4"))]
    pub tooltip: String,
    #[cfg_attr(feature = "host-protocol", prost(bool, tag = "5"))]
    pub enabled: bool,
    #[cfg_attr(feature = "host-protocol", prost(bool, tag = "6"))]
    pub selected: bool,
}
#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq)]
#[cfg_attr(feature = "host-protocol", derive(::prost::Message))]
#[cfg_attr(not(feature = "host-protocol"), derive(Debug, Default))]
pub struct UiField {
    #[cfg_attr(feature = "host-protocol", prost(string, tag = "1"))]
    pub id: String,
    #[cfg_attr(feature = "host-protocol", prost(string, tag = "2"))]
    pub label: String,
    #[cfg_attr(feature = "host-protocol", prost(string, tag = "3"))]
    pub kind: String,
    #[cfg_attr(feature = "host-protocol", prost(string, tag = "4"))]
    pub value_json: String,
    #[cfg_attr(feature = "host-protocol", prost(string, repeated, tag = "5"))]
    pub options: Vec<String>,
    #[cfg_attr(feature = "host-protocol", prost(double, tag = "6"))]
    pub minimum: f64,
    #[cfg_attr(feature = "host-protocol", prost(double, tag = "7"))]
    pub maximum: f64,
    /// 复杂原生表单的版本化 JSON 定义；空值沿用已有字段控件。
    #[cfg_attr(feature = "host-protocol", prost(string, tag = "8"))]
    #[serde(default)]
    pub schema_json: String,
}
/// 输出卡片由插件声明，宿主解释三类标准操作；每张卡片最多两个操作。
#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "host-protocol", derive(::prost::Message))]
#[cfg_attr(not(feature = "host-protocol"), derive(Debug, Default))]
pub struct OutputAction {
    #[cfg_attr(feature = "host-protocol", prost(string, tag = "1"))]
    pub id: String,
    #[cfg_attr(feature = "host-protocol", prost(string, tag = "2"))]
    pub label: String,
    #[cfg_attr(feature = "host-protocol", prost(string, tag = "3"))]
    pub kind: String,
    #[cfg_attr(feature = "host-protocol", prost(string, tag = "4"))]
    #[serde(default)]
    pub target: String,
}
#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq)]
#[cfg_attr(feature = "host-protocol", derive(::prost::Message))]
#[cfg_attr(not(feature = "host-protocol"), derive(Debug, Default))]
pub struct OutputCard {
    #[cfg_attr(feature = "host-protocol", prost(string, tag = "1"))]
    pub id: String,
    #[cfg_attr(feature = "host-protocol", prost(string, tag = "2"))]
    pub title: String,
    #[cfg_attr(feature = "host-protocol", prost(string, tag = "3"))]
    pub subtitle: String,
    #[cfg_attr(feature = "host-protocol", prost(message, repeated, tag = "4"))]
    #[serde(default)]
    pub actions: Vec<OutputAction>,
}
#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq)]
#[cfg_attr(feature = "host-protocol", derive(::prost::Message))]
#[cfg_attr(not(feature = "host-protocol"), derive(Debug, Default))]
pub struct UiContribution {
    #[cfg_attr(feature = "host-protocol", prost(message, repeated, tag = "1"))]
    pub commands: Vec<UiCommand>,
    #[cfg_attr(feature = "host-protocol", prost(message, repeated, tag = "2"))]
    pub fields: Vec<UiField>,
    #[cfg_attr(feature = "host-protocol", prost(message, repeated, tag = "3"))]
    #[serde(default)]
    pub output_cards: Vec<OutputCard>,
}
#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "host-protocol", derive(::prost::Message))]
#[cfg_attr(not(feature = "host-protocol"), derive(Debug, Default))]
pub struct StateSnapshot {
    #[cfg_attr(feature = "host-protocol", prost(string, tag = "1"))]
    pub plugin_id: String,
    #[cfg_attr(feature = "host-protocol", prost(uint32, tag = "2"))]
    pub format_version: u32,
    #[cfg_attr(feature = "host-protocol", prost(bytes = "vec", tag = "3"))]
    pub data: Vec<u8>,
}
#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq)]
#[cfg_attr(feature = "host-protocol", derive(::prost::Message))]
#[cfg_attr(not(feature = "host-protocol"), derive(Debug, Default))]
pub struct HostRequest {
    #[cfg_attr(feature = "host-protocol", prost(string, tag = "1"))]
    pub service: String,
    #[cfg_attr(feature = "host-protocol", prost(bytes = "vec", tag = "2"))]
    pub payload_json: Vec<u8>,
    #[cfg_attr(feature = "host-protocol", prost(message, optional, tag = "3"))]
    pub ui: Option<UiContribution>,
    #[cfg_attr(feature = "host-protocol", prost(string, tag = "4"))]
    pub event_id: String,
}

pub use crate::service_models::*;
