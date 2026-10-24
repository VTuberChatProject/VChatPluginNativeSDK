//! 调用期结构化视图；每个字段在 C ABI 上有固定类型和布局。
use crate::abi::*;
use crate::{memory::*, proto};
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Document {
    pub header: Header,
    pub version: u32,
    pub revision: u64,
    pub paragraphs: Slice<Paragraph>,
}
impl Value for proto::Document {
    type Ffi = Document;
    fn view(&self, a: &mut Arena) -> Result<Document, String> {
        if !self.json.is_empty() {
            return Err("ABI 2 文档必须使用结构化段落".into());
        }
        Ok(Document {
            header: Header::of::<Document>(),
            version: self.version,
            revision: self.revision,
            paragraphs: self.paragraphs.view(a)?,
        })
    }
    unsafe fn read_ptr(p: *const Document, b: &mut Budget) -> Result<Self, String> {
        let h = unsafe { checked(p.cast::<Header>())? };
        header::<Document>(*h)?;
        unsafe { Self::read(*checked(p)?, b) }
    }
    unsafe fn read(v: Document, b: &mut Budget) -> Result<Self, String> {
        header::<Document>(v.header)?;
        Ok(Self {
            version: v.version,
            revision: v.revision,
            json: Vec::new(),
            paragraphs: unsafe { Vec::<proto::Paragraph>::read(v.paragraphs, b)? },
        })
    }
}
ffi_struct!(Paragraph { segments: Vec<proto::Segment> });
ffi_struct!(Segment { text: String, style: Option<proto::TextStyle>, ruby: Option<String>, speed: Option<u32>, pause: Option<u32>, image: Option<proto::Image>, class: Option<String>, extra_json: Vec<u8> });
ffi_struct!(TextStyle { bold: bool, italic: bool, underline: bool, strikethrough: bool, color: Option<String>, background_color: Option<String>, size: Option<String>, extra_json: Vec<u8> });
ffi_struct!(Image {
    source: String,
    width: f32,
    height: f32
});
ffi_struct!(Message { id: String, source_id: String, source_message_id: Option<String>, received_at_ms: i64, sequence: u64, document: Option<proto::Document>, source_time: Option<i64>, username: String, status: String, origin_ids: Vec<String>, message_type: Option<proto::MessageType>, avatar_url: Option<String> });
ffi_struct!(MessageType {
    plugin_kind: String,
    plugin_id: String,
    type_id: String
});
ffi_struct!(Selection {
    start: u64,
    end: u64
});
ffi_struct!(PluginError {
    code: String,
    message: String
});
ffi_struct!(DocumentEdit { expected_revision: u64, replacement: Option<proto::Document> });
ffi_struct!(PluginResponse { payload_json: Vec<u8>, error: Option<proto::PluginError>, edit: Option<proto::DocumentEdit>, ui: Option<proto::UiContribution>, status: u32, content_type: String, no_store: bool, body: Vec<u8> });
ffi_struct!(UiCommand {
    id: String,
    label: String,
    icon: String,
    tooltip: String,
    enabled: bool,
    selected: bool
});
ffi_struct!(UiField { id: String, label: String, kind: String, value_json: String, options: Vec<String>, minimum: f64, maximum: f64, schema_json: String });
ffi_struct!(OutputAction {
    id: String,
    label: String,
    kind: String,
    target: String
});
ffi_struct!(OutputCard { id: String, title: String, subtitle: String, actions: Vec<proto::OutputAction> });
ffi_struct!(UiContribution { commands: Vec<proto::UiCommand>, fields: Vec<proto::UiField>, output_cards: Vec<proto::OutputCard> });
ffi_struct!(StateSnapshot { plugin_id: String, format_version: u32, data: Vec<u8> });
ffi_struct!(MessageInput { source_id: String, source_message_id: Option<String>, username: String, source_time: Option<i64>, message_type: Option<String>, avatar_url: Option<String>, origin_ids: Vec<String>, document: Option<proto::Document>, present: Vec<String>, extra_json: Vec<u8> });
ffi_struct!(MessageTypeDefinition {
    id: String,
    label: String
});
ffi_struct!(MessageTypes { version: u32, types: Vec<proto::MessageTypeDefinition> });
ffi_struct!(RuntimePaths { package_root: String, resources_root: String, data_root: String, assets_root: Option<String> });
ffi_struct!(FrontendPayload { version: u32, frontend_id: String, message_id: String, batch_id: String, run_id: String, generation: Option<u64>, label: String, kind: String, status: String, capabilities: Vec<String>, message: Option<proto::Message>, present: Vec<String>, extra_json: Vec<u8> });
ffi_struct!(HeaderPair {
    name: String,
    value: String
});
ffi_struct!(WebRoute {
    kind: String,
    method: String,
    path: String,
    operation: String
});
ffi_struct!(WebPayload { version: u32, method: String, path: String, query: String, connection_id: String, preview: bool, headers: Vec<proto::HeaderPair>, body: Vec<u8>, routes: Vec<proto::WebRoute>, data_json: Vec<u8>, present: Vec<String>, extra_json: Vec<u8> });
ffi_struct!(EventPayload { version: u32, event_id: String, kind: String, message: Option<proto::Message>, data_json: Vec<u8> });
ffi_struct!(LogPayload { message: String, level: String, context_json: Vec<u8> });
ffi_struct!(Subscription { events: Vec<String> });
ffi_struct!(WindowPayload { version: u32, window_id: String, title: String, content_json: Vec<u8>, properties_json: Vec<u8> });
ffi_struct!(TransformPayload { version: u32, message: Option<proto::Message>, document: Option<proto::Document>, wrapped: bool });
