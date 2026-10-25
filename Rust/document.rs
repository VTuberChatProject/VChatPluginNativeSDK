//! 将共享富文本文档转换为结构化 Protobuf，并保留扩展字段。
use crate::model::{Document, Image, Paragraph, Segment, TextStyle};
use crate::proto;

pub fn document_to_proto(value: &Document) -> proto::Document {
    proto::Document {
        version: value.version,
        revision: value.revision,
        json: Vec::new(),
        paragraphs: value
            .paragraphs
            .iter()
            .map(|p| proto::Paragraph {
                segments: p
                    .segments
                    .iter()
                    .map(|s| proto::Segment {
                        text: s.text.clone(),
                        style: Some(proto::TextStyle {
                            bold: s.style.bold,
                            italic: s.style.italic,
                            underline: s.style.underline,
                            strikethrough: s.style.strikethrough,
                            color: s.style.color.clone(),
                            background_color: s.style.background_color.clone(),
                            size: s.style.size.clone(),
                            extra_json: {
                                // 五项新属性走既有扩展通道，旧插件保留相同线协议。
                                let mut extras = s.style.extra.clone();
                                let all = serde_json::to_value(&s.style).unwrap();
                                for key in
                                    ["weight", "emphasis", "letterSpacing", "stretch", "shadow"]
                                {
                                    if let Some(value) = all.get(key) {
                                        extras.insert(key.into(), value.clone());
                                    }
                                }
                                serde_json::to_vec(&extras).unwrap()
                            },
                        }),
                        ruby: s.ruby.clone(),
                        speed: s.speed,
                        pause: s.pause,
                        image: s.image.as_ref().map(|i| proto::Image {
                            source: i.source.clone(),
                            width: i.width,
                            height: i.height,
                        }),
                        class: s.class.clone(),
                        extra_json: serde_json::to_vec(&s.extra).unwrap(),
                    })
                    .collect(),
            })
            .collect(),
    }
}
fn extras(bytes: &[u8]) -> Result<serde_json::Map<String, serde_json::Value>, String> {
    if bytes.is_empty() {
        return Ok(Default::default());
    }
    serde_json::from_slice(bytes).map_err(|e| e.to_string())
}
/// 接收时校验结构化字段；旧版 JSON-only 文档仍可导入。
pub fn document_from_proto(value: &proto::Document) -> Result<Document, String> {
    if value.paragraphs.is_empty() && !value.json.is_empty() {
        let doc: Document = serde_json::from_slice(&value.json).map_err(|e| e.to_string())?;
        doc.validate().map_err(|e| e.to_string())?;
        return Ok(doc);
    }
    let paragraphs = value
        .paragraphs
        .iter()
        .map(|p| {
            let segments = p
                .segments
                .iter()
                .map(|s| {
                    let style = if let Some(style) = &s.style {
                        let mut fields = extras(&style.extra_json)?;
                        for (key, value) in [
                            ("bold", serde_json::json!(style.bold)),
                            ("italic", serde_json::json!(style.italic)),
                            ("underline", serde_json::json!(style.underline)),
                            ("strikethrough", serde_json::json!(style.strikethrough)),
                            ("color", serde_json::json!(style.color)),
                            ("backgroundColor", serde_json::json!(style.background_color)),
                            ("size", serde_json::json!(style.size)),
                        ] {
                            fields.insert(key.into(), value);
                        }
                        serde_json::from_value::<TextStyle>(serde_json::Value::Object(fields))
                            .map_err(|e| e.to_string())?
                    } else {
                        TextStyle::default()
                    };
                    Ok(Segment {
                        text: s.text.clone(),
                        style,
                        ruby: s.ruby.clone(),
                        speed: s.speed,
                        pause: s.pause,
                        image: s.image.as_ref().map(|i| Image {
                            source: i.source.clone(),
                            width: i.width,
                            height: i.height,
                        }),
                        class: s.class.clone(),
                        extra: extras(&s.extra_json)?,
                    })
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(Paragraph { segments })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let doc = Document {
        version: value.version,
        revision: value.revision,
        paragraphs,
    };
    doc.validate().map_err(|e| e.to_string())?;
    Ok(doc)
}

/// 消息元数据与结构化文档共同进入 Protobuf，JSON 保留扩展入口。
pub fn message_to_proto(value: &crate::model::Message) -> proto::Message {
    proto::Message {
        id: value.id.to_string(),
        source_id: value.source_id.clone(),
        source_message_id: value.source_message_id.clone(),
        received_at_ms: value.received_at,
        sequence: value.sequence,
        document: Some(document_to_proto(&value.document)),
        source_time: value.source_time,
        username: value.username.clone(),
        avatar_url: value.avatar_url.clone(),
        status: serde_json::to_value(value.status)
            .unwrap()
            .as_str()
            .unwrap()
            .into(),
        origin_ids: value.origin_ids.clone(),
        message_type: value.message_type.as_ref().map(|kind| proto::MessageType {
            plugin_kind: kind.plugin_kind.clone(),
            plugin_id: kind.plugin_id.clone(),
            type_id: kind.type_id.clone(),
        }),
    }
}

#[cfg(all(test, feature = "host-protocol"))]
mod tests {
    use super::*;
    use prost::Message;
    #[test]
    fn structured_document_roundtrip_retains_extended_fields() {
        let mut doc = Document::plain("你好 👨‍👩‍👧‍👦\n下一段");
        let s = &mut doc.paragraphs[0].segments[0];
        s.style.bold = true;
        s.style.weight = Some("semi-bold".into());
        s.style.emphasis = Some(true);
        s.style.letter_spacing = Some("half-width".into());
        s.style.stretch = Some("condensed".into());
        s.style.shadow = Some(Default::default());
        s.style.color = Some("#aa1234".into());
        s.style
            .extra
            .insert("filter".into(), serde_json::json!("shadow"));
        s.ruby = Some("ni hao".into());
        s.speed = Some(30);
        s.pause = Some(4);
        s.image = Some(Image {
            source: "/media/a.png".into(),
            width: 32.0,
            height: 48.0,
        });
        s.extra
            .insert("custom".into(), serde_json::json!({"nested":[1,2]}));
        let wire = document_to_proto(&doc).encode_to_vec();
        let decoded = proto::Document::decode(wire.as_slice()).unwrap();
        assert_eq!(document_from_proto(&decoded).unwrap(), doc);
    }

    #[test]
    fn message_type_identity_is_optional_and_survives_protobuf() {
        let mut message =
            crate::model::Message::new("source", "author", Document::plain("消息"), 1, 1);
        let old = message_to_proto(&message).encode_to_vec();
        assert_eq!(
            proto::Message::decode(old.as_slice()).unwrap().message_type,
            None
        );
        message.message_type = Some(crate::model::MessageTypeKey {
            plugin_kind: "internal".into(),
            plugin_id: "example".into(),
            type_id: "remote".into(),
        });
        message.avatar_url = Some("https://example.com/avatar.png".into());
        let wire = message_to_proto(&message).encode_to_vec();
        let kind = proto::Message::decode(wire.as_slice())
            .unwrap()
            .message_type
            .unwrap();
        assert_eq!(
            proto::Message::decode(wire.as_slice()).unwrap().avatar_url,
            message.avatar_url
        );
        assert_eq!(
            (
                kind.plugin_kind.as_str(),
                kind.plugin_id.as_str(),
                kind.type_id.as_str()
            ),
            ("internal", "example", "remote")
        );
    }
}
