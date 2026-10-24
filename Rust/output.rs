//! 内置插件 Protobuf 输出声明与宿主共享模型的转换。
use crate::proto;
use vchat_core::{OutputAction, OutputActionKind, OutputCard};

/// 保留插件的声明顺序，并让未知标准操作在注册边界返回错误。
pub fn cards_from_proto(cards: &[proto::OutputCard]) -> Result<Vec<OutputCard>, String> {
    cards
        .iter()
        .map(|card| {
            Ok(OutputCard {
                id: card.id.clone(),
                title: card.title.clone(),
                subtitle: card.subtitle.clone(),
                actions: card
                    .actions
                    .iter()
                    .map(|action| {
                        Ok(OutputAction {
                            id: action.id.clone(),
                            label: action.label.clone(),
                            kind: OutputActionKind::parse(&action.kind)?,
                            target: action.target.clone(),
                        })
                    })
                    .collect::<Result<_, String>>()?,
            })
        })
        .collect()
}

/// 插件可直接使用共享模型构造声明，再映射至跨语言协议。
pub fn cards_to_proto(cards: &[OutputCard]) -> Vec<proto::OutputCard> {
    cards
        .iter()
        .map(|card| proto::OutputCard {
            id: card.id.clone(),
            title: card.title.clone(),
            subtitle: card.subtitle.clone(),
            actions: card
                .actions
                .iter()
                .map(|action| proto::OutputAction {
                    id: action.id.clone(),
                    label: action.label.clone(),
                    kind: action.kind.as_str().into(),
                    target: action.target.clone(),
                })
                .collect(),
        })
        .collect()
}

#[cfg(all(test, feature = "host-protocol"))]
mod tests {
    use super::*;
    use prost::Message;

    #[test]
    fn old_contribution_defaults_to_empty_cards() {
        let old = proto::UiContribution::decode(&[][..]).unwrap();
        assert!(old.output_cards.is_empty());
        let json: proto::UiContribution =
            serde_json::from_value(serde_json::json!({"commands":[],"fields":[]})).unwrap();
        assert!(json.output_cards.is_empty());
    }

    #[test]
    fn cards_round_trip_and_unknown_actions_fail() {
        let cards = vec![OutputCard {
            id: "live".into(),
            title: "Live".into(),
            subtitle: "实时输出".into(),
            actions: vec![OutputAction {
                id: "open".into(),
                label: "浏览器打开".into(),
                kind: OutputActionKind::OpenUrl,
                target: "/Echo-Live/live.html".into(),
            }],
        }];
        let mut proto = cards_to_proto(&cards);
        assert_eq!(cards_from_proto(&proto).unwrap(), cards);
        proto[0].actions[0].kind = "unknown".into();
        assert!(cards_from_proto(&proto).is_err());
    }
}
