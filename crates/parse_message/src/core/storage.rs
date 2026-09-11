use super::models::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tsify::Tsify;

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct StorageInput {
    pub clipping_directory: String,
    pub messages: Vec<ProcessedMessage>,
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
// Only guards against the same message appearing twice within one batch;
// a file already saved from an earlier run is caught by the host checking
// for an existing file at the (deterministic) target path before writing.
pub fn plan(input: StorageInput) -> StoragePlan {
    let mut ids = HashSet::new();
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
    fn storage_plan_deduplicates_messages_repeated_within_a_batch() {
        let clipping = messages::processed("clip".into(), None, message("2"), "Asia/Tokyo").unwrap();
        let input = StorageInput {
            clipping_directory: "Clips".into(),
            messages: vec![clipping.clone(), clipping],
        };
        let plan = super::plan(input);
        assert_eq!(plan.writes.len(), 1);
        assert_eq!(plan.writes[0].path, "Clips/20260701_003000_2.md");
    }
}
