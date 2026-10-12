//! 富文本文档及 Echo-Live 输出适配，保留未知扩展字段。
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use unicode_segmentation::UnicodeSegmentation;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Document {
    pub version: u32,
    pub revision: u64,
    pub paragraphs: Vec<Paragraph>,
}

impl Default for Document {
    fn default() -> Self {
        Self {
            version: 1,
            revision: 0,
            paragraphs: vec![Paragraph::default()],
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Paragraph {
    pub segments: Vec<Segment>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Segment {
    pub text: String,
    pub style: TextStyle,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ruby: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pause: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<Image>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub class: Option<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct TextStyle {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(
        rename = "backgroundColor",
        alias = "background_color",
        skip_serializing_if = "Option::is_none"
    )]
    pub background_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
    #[serde(
        default,
        deserialize_with = "read_weight",
        skip_serializing_if = "Option::is_none"
    )]
    pub weight: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emphasis: Option<bool>,
    #[serde(
        rename = "letterSpacing",
        alias = "letter_spacing",
        skip_serializing_if = "Option::is_none"
    )]
    pub letter_spacing: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stretch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shadow: Option<TextShadow>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// 旧扩展允许数字字重，读入后转换成 EchoLive 命名档位。
fn read_weight<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<String>, D::Error> {
    let value = Option::<Value>::deserialize(deserializer)?;
    Ok(value.and_then(|value| {
        let text = match value {
            Value::String(s) => s,
            Value::Number(n) => n.to_string(),
            _ => return None,
        };
        Some(
            match text.as_str() {
                "100" => "thin",
                "200" => "extra-light",
                "300" => "light",
                "350" => "semi-light",
                "400" => "normal",
                "500" => "medium",
                "600" => "semi-bold",
                "700" => "bold",
                "800" => "extra-bold",
                "900" => "black",
                "950" => "extra-black",
                _ => &text,
            }
            .to_owned(),
        )
    }))
}

/// EchoLive 投影长度保留 CSS 单位，界面编辑统一使用 em。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct TextShadow {
    #[serde(deserialize_with = "read_length")]
    pub x: String,
    #[serde(deserialize_with = "read_length")]
    pub y: String,
    #[serde(deserialize_with = "read_length")]
    pub blur: String,
    pub color: String,
}

/// 导入的零长度可以使用 JSON 数字，保持 EchoLive 原始单位表示。
fn read_length<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    match Value::deserialize(deserializer)? {
        Value::String(value) => Ok(value),
        Value::Number(value) => Ok(value.to_string()),
        _ => Err(serde::de::Error::custom("投影长度需要字符串或数字")),
    }
}

impl Default for TextShadow {
    fn default() -> Self {
        Self {
            x: "0em".into(),
            y: "0em".into(),
            blur: "0.25em".into(),
            color: "#000000".into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Image {
    pub source: String,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, thiserror::Error)]
pub enum DocumentError {
    #[error("unsupported document version: {0}")]
    Version(u32),
    #[error("document exceeds the paragraph or segment limit")]
    TooLarge,
    #[error("invalid image dimensions or source")]
    Image,
    #[error("invalid speed, pause, or official font size")]
    TimingOrSize,
    #[error("unsafe rich-text attribute: {0}")]
    UnsafeAttribute(&'static str),
}

impl Document {
    /// 按换行拆分纯文本段落。
    pub fn plain(text: impl AsRef<str>) -> Self {
        Self {
            paragraphs: text
                .as_ref()
                .split('\n')
                .map(|text| Paragraph {
                    segments: vec![Segment {
                        text: text.to_owned(),
                        ..Default::default()
                    }],
                })
                .collect(),
            ..Self::default()
        }
    }

    /// 校验进入插件与输出边界的文档。
    pub fn validate(&self) -> Result<(), DocumentError> {
        if self.version != 1 {
            return Err(DocumentError::Version(self.version));
        }
        if self.paragraphs.len() > 1000
            || self
                .paragraphs
                .iter()
                .map(|p| p.segments.len())
                .sum::<usize>()
                > 10000
            || self.plain_text().len() > 1_000_000
        {
            return Err(DocumentError::TooLarge);
        }
        for segment in self.paragraphs.iter().flat_map(|p| &p.segments) {
            if let Some(image) = &segment.image
                && (image.source.is_empty()
                    || !image.width.is_finite()
                    || !image.height.is_finite()
                    || image.width <= 0.0
                    || image.height <= 0.0)
            {
                return Err(DocumentError::Image);
            }
            if let Some(image) = &segment.image {
                validate_image_source(&image.source)?;
            }
            if segment
                .class
                .as_deref()
                .is_some_and(|value| !safe_attribute(value))
            {
                return Err(DocumentError::UnsafeAttribute("class"));
            }
            if let Some(shadow) = &segment.style.shadow {
                for length in [&shadow.x, &shadow.y, &shadow.blur] {
                    let number = length
                        .trim()
                        .strip_suffix("em")
                        .or_else(|| length.trim().strip_suffix("px"))
                        .unwrap_or(length.trim());
                    if !number.parse::<f32>().is_ok_and(|v| v.is_finite()) {
                        return Err(DocumentError::UnsafeAttribute("shadow length"));
                    }
                }
                let blur = shadow
                    .blur
                    .trim()
                    .trim_end_matches("em")
                    .trim_end_matches("px");
                if blur.parse::<f32>().is_ok_and(|v| v < 0.0)
                    || !safe_css(&shadow.color)
                    || shadow.color.contains([';', '{', '}'])
                {
                    return Err(DocumentError::UnsafeAttribute("shadow"));
                }
            }
            for color in [&segment.style.color, &segment.style.background_color]
                .into_iter()
                .flatten()
            {
                if !safe_css(color) || color.contains([';', '{', '}']) {
                    return Err(DocumentError::UnsafeAttribute("color"));
                }
            }
            validate_extension(
                &serde_json::to_value(&segment.style).expect("样式为有效 JSON"),
                "style",
            )?;
            validate_extension(&Value::Object(segment.extra.clone()), "extra")?;
            if segment.speed == Some(0)
                || segment.speed.is_some_and(|v| v > 60_000)
                || segment.pause.is_some_and(|v| v > 10000)
                || segment.style.size.as_deref().is_some_and(|v| {
                    !["extra-small", "small", "middle", "large", "extra-large"].contains(&v)
                })
            {
                return Err(DocumentError::TimingOrSize);
            }
        }
        Ok(())
    }

    pub fn plain_text(&self) -> String {
        self.paragraphs
            .iter()
            .map(|p| {
                p.segments
                    .iter()
                    .map(|s| s.text.as_str())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// 字素跨样式片段统一计数，图片各占一个单位。
    pub fn visible_graphemes(&self) -> usize {
        self.paragraphs
            .iter()
            .map(|p| {
                let text: String = p.segments.iter().map(|s| s.text.as_str()).collect();
                text.graphemes(true).count()
                    + p.segments.iter().filter(|s| s.image.is_some()).count()
            })
            .sum()
    }

    pub fn split_paragraphs(&self) -> Vec<Self> {
        self.paragraphs
            .iter()
            .cloned()
            .map(|paragraph| Self {
                version: self.version,
                revision: self.revision,
                paragraphs: vec![paragraph],
            })
            .collect()
    }

    /// 一个输出项内用换行保留段落，队列独立掌握后续播放项。
    pub fn echo_data(&self, username: &str) -> Value {
        let mut message = Vec::new();
        for (index, paragraph) in self.paragraphs.iter().enumerate() {
            if index > 0 {
                message.push(json!({ "text": "\n" }));
            }
            for segment in &paragraph.segments {
                let mut value = segment.extra.clone();
                value.insert("text".into(), json!(segment.text));
                value.insert("style".into(), json!(segment.style));
                for (key, val) in [
                    ("ruby", segment.ruby.as_ref().map(|v| json!(escape_html(v)))),
                    ("speed", segment.speed.map(|v| json!(v))),
                    ("pause", segment.pause.map(|v| json!(v))),
                    ("class", segment.class.as_ref().map(|v| json!(v))),
                ] {
                    if let Some(val) = val {
                        value.insert(key.into(), val);
                    }
                }
                if let Some(image) = &segment.image {
                    let data = value.entry("data").or_insert_with(|| json!({}));
                    if let Some(data) = data.as_object_mut() {
                        data.insert(
                            "image".into(),
                            json!({ "url": image.source, "size": {
                                "width": { "value": format!("{}px", image.width) },
                                "height": { "value": format!("{}px", image.height) }
                            }}),
                        );
                    }
                }
                message.push(Value::Object(value));
            }
        }
        json!({ "username": username, "messages": [{ "message": message }] })
    }
}

fn safe_attribute(value: &str) -> bool {
    value.len() <= 65_536
        && !value
            .chars()
            .any(|c| matches!(c, '\'' | '"' | '<' | '>' | '`' | '\\') || c.is_control())
}

fn safe_css(value: &str) -> bool {
    let normalized = value.to_ascii_lowercase();
    safe_attribute(value)
        && !["url(", "expression(", "javascript:", "@import", "/*", "*/"]
            .iter()
            .any(|needle| normalized.contains(needle))
}

/// 图片支持 HTTP、HTTPS、本机绝对路径及受控相对路径。
pub fn validate_image_source(source: &str) -> Result<(), DocumentError> {
    if source.is_empty()
        || source.len() > 16_384
        || source
            .chars()
            .any(|c| matches!(c, '\'' | '"' | '<' | '>' | '`') || c.is_control())
    {
        return Err(DocumentError::UnsafeAttribute("image URL"));
    }
    let lower = source.to_ascii_lowercase();
    let is_windows_path = source.as_bytes().get(1) == Some(&b':')
        && source
            .as_bytes()
            .first()
            .is_some_and(u8::is_ascii_alphabetic);
    if lower.contains(':')
        && !is_windows_path
        && !["http://", "https://", "file:///"]
            .iter()
            .any(|prefix| lower.starts_with(prefix))
    {
        return Err(DocumentError::UnsafeAttribute("image URL scheme"));
    }
    Ok(())
}

fn validate_extension(value: &Value, context: &'static str) -> Result<(), DocumentError> {
    match value {
        Value::Object(fields) => {
            for (key, value) in fields {
                let next = match key.as_str() {
                    "url" | "source" => "url",
                    "style" => "style",
                    "username" | "text" | "typewrite" => "text",
                    _ => context,
                };
                validate_extension(value, next)?;
            }
        }
        Value::Array(values) => {
            for value in values {
                validate_extension(value, context)?;
            }
        }
        Value::String(value) => {
            if context == "url" {
                validate_image_source(value)?;
            } else if context != "text" && !safe_css(value) {
                return Err(DocumentError::UnsafeAttribute(context));
            }
        }
        _ => {}
    }
    Ok(())
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
