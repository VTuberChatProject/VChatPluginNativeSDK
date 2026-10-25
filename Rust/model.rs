//! 插件拥有的文档、消息及输出卡片模型，与宿主通过稳定 ABI/JSON 交换。
#[path = "Model/document.rs"]
mod document;
#[path = "Model/message.rs"]
mod message;
#[path = "Model/message_type.rs"]
mod message_type;
#[path = "Model/output_card.rs"]
mod output_card;

pub use document::*;
pub use message::*;
pub use message_type::*;
pub use output_card::*;
