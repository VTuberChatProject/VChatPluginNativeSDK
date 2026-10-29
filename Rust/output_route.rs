//! 按输出卡片登记的版本化业务载荷；C ABI 2 通过现有扩展字段承载。
use serde::{Deserialize, Serialize};

pub const OUTPUT_CARD_CAPABILITY: &str = "output_cards_v1";
pub const OUTPUT_REGISTRATION_VERSION: u32 = 2;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutputEndpoint {
    pub card_id: String,
    /// completed 等待打印完成，accepted 表示接收或归档成功。
    pub receipt_mode: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutputRegistration {
    pub version: u32,
    pub outputs: Vec<OutputEndpoint>,
}

impl OutputRegistration {
    pub fn new(outputs: Vec<OutputEndpoint>) -> Self {
        Self {
            version: OUTPUT_REGISTRATION_VERSION,
            outputs,
        }
    }
}

/// 身份编码与宿主共享 JSON 元组规则，允许名称包含分隔符。
pub fn frontend_id(kind: &str, plugin_id: &str, card_id: &str) -> String {
    format!(
        "card:{}",
        serde_json::to_string(&(kind, plugin_id, card_id)).expect("string tuple")
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn registration_and_endpoint_identity_are_stable() {
        let registration = OutputRegistration::new(vec![OutputEndpoint {
            card_id: "live".into(),
            receipt_mode: "completed".into(),
        }]);
        assert_eq!(serde_json::to_value(registration).unwrap()["version"], 2);
        assert_eq!(
            frontend_id("internal", "plugin:a", "card:b"),
            "card:[\"internal\",\"plugin:a\",\"card:b\"]"
        );
    }
}
