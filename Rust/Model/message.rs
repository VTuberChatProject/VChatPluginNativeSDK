//! 消息标识、来源策略及生命周期状态。
use crate::model::{Document, MessageTypeKey};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageStatus {
    #[default]
    Received,
    Processing,
    Queued,
    Transmitted,
    Playing,
    Completed,
    Failed,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Message {
    pub id: Uuid,
    pub source_id: String,
    pub source_message_id: Option<String>,
    pub source_time: Option<i64>,
    pub received_at: i64,
    pub sequence: u64,
    pub username: String,
    /// 插件提供的头像链接，旧消息省略时采用姓名字头像。
    #[serde(default)]
    pub avatar_url: Option<String>,
    pub document: Document,
    pub status: MessageStatus,
    #[serde(default)]
    pub origin_ids: Vec<String>,
    #[serde(default)]
    pub message_type: Option<MessageTypeKey>,
}

impl Message {
    /// 所有入口使用宿主接收时间和序号建立总顺序。
    pub fn new(
        source_id: impl Into<String>,
        username: impl Into<String>,
        document: Document,
        received_at: i64,
        sequence: u64,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            source_id: source_id.into(),
            source_message_id: None,
            source_time: None,
            received_at,
            sequence,
            username: username.into(),
            avatar_url: None,
            document,
            status: MessageStatus::Received,
            origin_ids: Vec::new(),
            message_type: None,
        }
    }

    /// 旧消息缺少字段时仍有稳定身份，供暂停策略统一匹配。
    pub fn type_key(&self) -> MessageTypeKey {
        self.message_type
            .clone()
            .unwrap_or_else(MessageTypeKey::legacy)
    }

    /// 本地输入使用内置类型，历史主播消息按来源兼容识别。
    pub fn is_self(&self) -> bool {
        self.message_type
            .as_ref()
            .map_or(self.source_id == "host", |key| {
                *key == MessageTypeKey::local()
            })
    }

    pub fn resend(&self, received_at: i64, sequence: u64) -> Self {
        Self {
            id: Uuid::new_v4(),
            source_message_id: None,
            received_at,
            sequence,
            status: MessageStatus::Received,
            ..self.clone()
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SourcePolicy {
    pub auto_forward: bool,
}

impl Default for SourcePolicy {
    fn default() -> Self {
        Self { auto_forward: true }
    }
}
