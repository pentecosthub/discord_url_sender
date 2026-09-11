use super::models::*;
use regex_lite::Regex;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, sync::LazyLock};
use tsify::Tsify;

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct StorageInput {
    pub clipping_directory: String,
    pub messages: Vec<ProcessedMessage>,
    pub existing_ids: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
pub struct Write {
    pub path: String,
    pub content: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
pub struct StoragePlan {
    pub writes: Vec<Write>,
}
static INDIVIDUAL_ID: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[0-9]{8}_[0-9]{6}_([0-9]+)\.md$").unwrap());
pub fn individual_id(name: &str) -> Option<String> {
    INDIVIDUAL_ID.captures(name).map(|c| c[1].to_string())
}
pub fn plan(input: StorageInput) -> StoragePlan {
    let mut ids: HashSet<_> = input.existing_ids.into_iter().collect();
    let mut writes = Vec::new();
    for message in input.messages {
        if !ids.insert(message.message_id.clone()) {
            continue;
        }
        writes.push(Write {
            path: format!("{}/{}.md", input.clipping_directory, message.file_name),
            content: message.markdown,
        });
    }
    StoragePlan { writes }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::messages;

    fn message(id: &str) -> DiscordMessage {
        DiscordMessage {
            id: id.into(),
            content: "hello".into(),
            timestamp: "2026-06-30T15:30:00Z".into(),
            author: None,
            member: None,
        }
    }

    #[test]
    fn storage_plan_deduplicates_by_message_id() {
        let clipping = messages::processed("clip".into(), message("2"), "Asia/Tokyo").unwrap();
        let input = StorageInput {
            clipping_directory: "Clips".into(),
            messages: vec![clipping.clone(), clipping],
            existing_ids: vec![],
        };
        let plan = super::plan(input.clone());
        assert_eq!(plan.writes.len(), 1);
        assert_eq!(plan.writes[0].path, "Clips/20260701_003000_2.md");

        let plan = super::plan(StorageInput {
            existing_ids: vec!["2".into()],
            ..input
        });
        assert!(plan.writes.is_empty());
    }

    #[test]
    fn individual_ids_reject_lookalike_names_without_parsing_calendar_dates() {
        for (name, expected) in [
            ("20260701_123456_0001.md", "0001"),
            ("99999999_999999_123.md", "123"),
        ] {
            assert_eq!(individual_id(name).as_deref(), Some(expected));
        }
        for name in [
            "2026071_123456_1.md",
            "20260701_12345_1.md",
            "20260701_123456_.md",
            "20260701_123456_1_2.md",
            "20260701_123456_１２.md",
            "２０２６0701_123456_1.md",
            "20260701_123456_1.md\n",
            "20260701_123456_1.MD",
            "notes/20260701_123456_1.md",
        ] {
            assert!(individual_id(name).is_none(), "name: {name:?}");
        }
    }
}
