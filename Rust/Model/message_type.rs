//! 插件消息类型声明与跨入口稳定身份，保留旧消息的兼容分组。
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct MessageTypeKey {
    pub plugin_kind: String,
    pub plugin_id: String,
    pub type_id: String,
}

impl MessageTypeKey {
    /// 主编辑器发送的内置类型。
    pub fn local() -> Self {
        Self {
            plugin_kind: "builtin".into(),
            plugin_id: "host".into(),
            type_id: "local".into(),
        }
    }

    /// 历史快照中缺少类型身份的消息归入独立兼容类型。
    pub fn legacy() -> Self {
        Self {
            plugin_kind: "builtin".into(),
            plugin_id: "history".into(),
            type_id: "unclassified".into(),
        }
    }

    /// 空类型 ID 专供旧插件省略类型时使用，保留插件归属。
    pub fn unclassified(kind: impl Into<String>, id: impl Into<String>) -> Self {
        Self {
            plugin_kind: kind.into(),
            plugin_id: id.into(),
            type_id: String::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageTypeDefinition {
    pub id: String,
    pub label: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageTypeRegistration {
    pub version: u32,
    pub types: Vec<MessageTypeDefinition>,
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum MessageTypeError {
    #[error("unsupported message type registration version {0}")]
    Version(u32),
    #[error("message type ID and label must contain non-whitespace characters")]
    Empty,
    #[error("message type ID must not contain leading or trailing whitespace")]
    UntrimmedId,
    #[error("duplicate message type ID: {0}")]
    Duplicate(String),
}

impl MessageTypeRegistration {
    /// 按完整声明校验，空列表允许插件主动撤销所有已声明类型。
    pub fn validate(&self) -> Result<(), MessageTypeError> {
        if self.version != 1 {
            return Err(MessageTypeError::Version(self.version));
        }
        let mut ids = BTreeSet::new();
        for entry in &self.types {
            if entry.id.trim().is_empty() || entry.label.trim().is_empty() {
                return Err(MessageTypeError::Empty);
            }
            if entry.id.trim() != entry.id {
                return Err(MessageTypeError::UntrimmedId);
            }
            if !ids.insert(&entry.id) {
                return Err(MessageTypeError::Duplicate(entry.id.clone()));
            }
        }
        Ok(())
    }
}
