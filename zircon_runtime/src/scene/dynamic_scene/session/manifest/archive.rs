use std::sync::Arc;

use serde::{Deserialize, Serialize};

use super::order::compare_slot_summary_update_order;
use super::summary::RuntimeSessionSlotSummary;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeSessionArchiveManifest {
    pub format_version: u32,
    #[serde(default)]
    pub slots: Arc<Vec<RuntimeSessionSlotSummary>>,
}

impl RuntimeSessionArchiveManifest {
    pub fn slot_count(&self) -> usize {
        self.slots.len()
    }

    pub fn is_empty(&self) -> bool {
        self.slots.is_empty()
    }

    pub fn slot(&self, slot_id: &str) -> Option<&RuntimeSessionSlotSummary> {
        self.slots.iter().find(|slot| slot.slot_id == slot_id)
    }

    pub fn slot_ids(&self) -> impl Iterator<Item = &str> {
        self.slots.iter().map(|slot| slot.slot_id.as_str())
    }

    pub fn latest_updated_slot(&self) -> Option<&RuntimeSessionSlotSummary> {
        self.slots
            .iter()
            .max_by(|left, right| compare_slot_summary_update_order(*left, *right))
    }

    pub fn oldest_updated_slot(&self) -> Option<&RuntimeSessionSlotSummary> {
        self.slots
            .iter()
            .min_by(|left, right| compare_slot_summary_update_order(*left, *right))
    }

    pub fn slots_with_tag<'a>(
        &'a self,
        tag: &'a str,
    ) -> impl Iterator<Item = &'a RuntimeSessionSlotSummary> + 'a {
        let tag = tag.trim();
        self.slots
            .iter()
            .filter(move |slot| slot.metadata.tags.iter().any(|candidate| candidate == tag))
    }

    pub fn latest_updated_slot_with_tag(&self, tag: &str) -> Option<&RuntimeSessionSlotSummary> {
        let tag = tag.trim();
        if tag.is_empty() {
            return None;
        }
        self.slots
            .iter()
            .filter(|slot| slot.metadata.tags.iter().any(|candidate| candidate == tag))
            .max_by(|left, right| compare_slot_summary_update_order(*left, *right))
    }

    pub fn oldest_updated_slot_with_tag(&self, tag: &str) -> Option<&RuntimeSessionSlotSummary> {
        let tag = tag.trim();
        if tag.is_empty() {
            return None;
        }
        self.slots
            .iter()
            .filter(|slot| slot.metadata.tags.iter().any(|candidate| candidate == tag))
            .min_by(|left, right| compare_slot_summary_update_order(*left, *right))
    }

    pub fn slots_matching_display_name<'a>(
        &'a self,
        query: &'a str,
    ) -> impl Iterator<Item = &'a RuntimeSessionSlotSummary> + 'a {
        let query = query.trim();
        self.slots.iter().filter(move |slot| {
            !query.is_empty()
                && slot
                    .metadata
                    .display_name
                    .as_deref()
                    .is_some_and(|display_name| display_name.contains(query))
        })
    }
}
