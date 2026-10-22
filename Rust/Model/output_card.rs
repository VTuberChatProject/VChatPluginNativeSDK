//! 插件输出卡片、宿主标准操作与统一地址解析契约。
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use url::Url;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutputCard {
    pub id: String,
    pub title: String,
    pub subtitle: String,
    #[serde(default)]
    pub actions: Vec<OutputAction>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutputAction {
    pub id: String,
    pub label: String,
    pub kind: OutputActionKind,
    #[serde(default)]
    pub target: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputActionKind {
    CopyUrl,
    OpenUrl,
    OpenDialogueWindow,
}

impl OutputActionKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CopyUrl => "copy_url",
            Self::OpenUrl => "open_url",
            Self::OpenDialogueWindow => "open_dialogue_window",
        }
    }

    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "copy_url" => Ok(Self::CopyUrl),
            "open_url" => Ok(Self::OpenUrl),
            "open_dialogue_window" => Ok(Self::OpenDialogueWindow),
            _ => Err(format!("未知输出操作类型: {value}")),
        }
    }
}

/// 宿主与两类插件共享校验；原生窗口权限由调用方按插件身份授予。
pub fn validate_output_cards(
    cards: &[OutputCard],
    allow_dialogue_window: bool,
) -> Result<(), String> {
    let mut ids = HashSet::new();
    for card in cards {
        if card.id.trim().is_empty() || !ids.insert(&card.id) {
            return Err("输出卡片 ID 需要非空且唯一".into());
        }
        if card.title.trim().is_empty() || card.subtitle.trim().is_empty() {
            return Err("输出卡片需要主标题和用途副标题".into());
        }
        if card.actions.len() > 2 {
            return Err("每张输出卡片最多包含两个操作".into());
        }
        let mut actions = HashSet::new();
        for action in &card.actions {
            if action.id.trim().is_empty() || !actions.insert(&action.id) {
                return Err("输出操作 ID 需要在卡片内非空且唯一".into());
            }
            if action.label.trim().is_empty() {
                return Err("输出操作名称不能为空".into());
            }
            match action.kind {
                OutputActionKind::CopyUrl | OutputActionKind::OpenUrl => {
                    validate_url(&action.target)?;
                }
                OutputActionKind::OpenDialogueWindow => {
                    if !allow_dialogue_window || !action.target.is_empty() {
                        return Err("原生对话窗口操作仅向 NanaDialogue 开放且目标应为空".into());
                    }
                }
            }
        }
    }
    Ok(())
}

fn validate_url(value: &str) -> Result<(), String> {
    if value.is_empty()
        || value
            .chars()
            .any(|c| c.is_whitespace() || c.is_control() || c == '\\')
    {
        return Err("输出地址包含无效字符".into());
    }
    if value.starts_with('/') {
        return if value.starts_with("//") {
            Err("输出地址需要使用单个斜杠开头的宿主路径".into())
        } else {
            Ok(())
        };
    }
    let url = Url::parse(value).map_err(|_| "输出地址格式无效".to_string())?;
    if !matches!(url.scheme(), "http" | "https")
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return Err("输出地址需要使用 HTTP 或 HTTPS 且凭据为空".into());
    }
    Ok(())
}

/// 根路径始终归属当前宿主服务；绝对地址保留插件声明的目标。
pub fn resolve_output_cards(
    cards: &[OutputCard],
    hub_url: &str,
) -> Result<Vec<OutputCard>, String> {
    validate_output_cards(cards, true)?;
    let mut resolved = cards.to_vec();
    for card in &mut resolved {
        for action in &mut card.actions {
            if matches!(
                action.kind,
                OutputActionKind::CopyUrl | OutputActionKind::OpenUrl
            ) && action.target.starts_with('/')
            {
                validate_url(hub_url)?;
                let base = Url::parse(hub_url).map_err(|_| "宿主输出地址格式无效".to_string())?;
                action.target = base
                    .join(&action.target)
                    .map_err(|e| e.to_string())?
                    .to_string();
            }
        }
    }
    Ok(resolved)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cards(target: &str) -> Vec<OutputCard> {
        vec![OutputCard {
            id: "live".into(),
            title: "Live".into(),
            subtitle: "实时输出".into(),
            actions: vec![OutputAction {
                id: "open".into(),
                label: "打开".into(),
                kind: OutputActionKind::OpenUrl,
                target: target.into(),
            }],
        }]
    }

    #[test]
    fn addresses_use_current_host_and_keep_external_urls() {
        let source = cards("/Echo-Live/live.html?mode=live#output");
        assert_eq!(
            resolve_output_cards(&source, "http://127.0.0.1:42123").unwrap()[0].actions[0].target,
            "http://127.0.0.1:42123/Echo-Live/live.html?mode=live#output"
        );
        assert_eq!(
            source[0].actions[0].target,
            "/Echo-Live/live.html?mode=live#output"
        );
        let external = cards("https://example.com/output");
        assert_eq!(resolve_output_cards(&external, "").unwrap(), external);
        assert!(resolve_output_cards(&source, "").is_err());
    }

    #[test]
    fn unsafe_or_ambiguous_addresses_are_rejected() {
        for target in [
            "",
            "//example.com/live",
            "file:///tmp/live",
            "javascript:alert(1)",
            "https://user@example.com/live",
            "https://user:secret@example.com/live",
            "relative/live",
            "/\\example.com/live",
            " https://example.com",
            "/live\n",
        ] {
            assert!(
                validate_output_cards(&cards(target), false).is_err(),
                "{target:?}"
            );
        }
    }

    #[test]
    fn action_limits_identity_and_window_permissions_are_checked() {
        let mut value = cards("/live");
        value[0].actions.clear();
        assert!(validate_output_cards(&value, false).is_ok());
        value[0].actions = vec![cards("/live")[0].actions[0].clone(); 3];
        assert!(validate_output_cards(&value, true).is_err());
        let mut value = cards("/live");
        let duplicate = value[0].actions[0].clone();
        value[0].actions.push(duplicate);
        assert!(validate_output_cards(&value, true).is_err());
        let mut value = cards("/live");
        value.push(value[0].clone());
        assert!(validate_output_cards(&value, true).is_err());
        let mut value = cards("/live");
        value[0].actions[0].kind = OutputActionKind::OpenDialogueWindow;
        value[0].actions[0].target.clear();
        assert!(validate_output_cards(&value, true).is_ok());
        assert!(validate_output_cards(&value, false).is_err());
        assert_eq!(
            serde_json::to_value(&value).unwrap()[0]["actions"][0]["kind"],
            "open_dialogue_window"
        );
    }
}
