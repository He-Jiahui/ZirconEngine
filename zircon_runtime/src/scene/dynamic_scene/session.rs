use std::collections::BTreeSet;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::scene::{LevelSystem, World};

use super::EntityRemap;

mod error;
mod io;
mod manifest;
mod merge;
mod metadata;
mod path_status;
mod reports;
mod retention;
mod slot;
mod slot_id;
mod statistics;

pub use error::RuntimeSessionArchiveError;
pub use manifest::{RuntimeSessionArchiveManifest, RuntimeSessionSlotSummary};
pub use merge::{RuntimeSessionArchiveMergePolicy, RuntimeSessionArchiveMergeReport};
pub use metadata::RuntimeSessionMetadata;
pub use path_status::RuntimeSessionArchivePathStatus;
pub use reports::{RuntimeSessionLevelRestoreReport, RuntimeSessionSlotDiffReport};
pub use retention::{RuntimeSessionArchivePruneReport, RuntimeSessionArchiveRetentionPolicy};
pub use slot::RuntimeSessionSlot;
pub use statistics::RuntimeSessionArchiveStatistics;

use slot_id::{normalize_slot_id, validate_canonical_slot_id};

pub const RUNTIME_SESSION_ARCHIVE_FORMAT_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RuntimeSessionArchive {
    pub format_version: u32,
    #[serde(default)]
    pub slots: Vec<RuntimeSessionSlot>,
}

impl RuntimeSessionArchive {
    pub fn empty() -> Self {
        Self {
            format_version: RUNTIME_SESSION_ARCHIVE_FORMAT_VERSION,
            slots: Vec::new(),
        }
    }

    pub fn from_slots(slots: Vec<RuntimeSessionSlot>) -> Result<Self, RuntimeSessionArchiveError> {
        let mut archive = Self {
            format_version: RUNTIME_SESSION_ARCHIVE_FORMAT_VERSION,
            slots,
        };
        archive.normalize_slot_metadata();
        archive.sort_slots();
        archive.ensure_supported()?;
        Ok(archive)
    }

    pub fn from_world(
        slot_id: impl Into<String>,
        world: &World,
    ) -> Result<Self, RuntimeSessionArchiveError> {
        Self::from_world_with_metadata(slot_id, world, RuntimeSessionMetadata::default())
    }

    pub fn from_world_with_metadata(
        slot_id: impl Into<String>,
        world: &World,
        metadata: RuntimeSessionMetadata,
    ) -> Result<Self, RuntimeSessionArchiveError> {
        Self::from_slots(vec![RuntimeSessionSlot::from_world_with_metadata(
            slot_id, world, metadata,
        )?])
    }

    pub fn from_level(
        slot_id: impl Into<String>,
        level: &LevelSystem,
    ) -> Result<Self, RuntimeSessionArchiveError> {
        Self::from_slots(vec![RuntimeSessionSlot::from_level(slot_id, level)?])
    }

    pub fn from_versioned_json(json: &str) -> Result<Self, RuntimeSessionArchiveError> {
        let mut archive: Self = serde_json::from_str(json)?;
        archive.normalize_slot_metadata();
        archive.ensure_supported()?;
        archive.sort_slots();
        Ok(archive)
    }

    pub fn to_versioned_json_pretty(&self) -> Result<String, RuntimeSessionArchiveError> {
        let mut archive = self.clone();
        archive.normalize_slot_metadata();
        archive.sort_slots();
        archive.ensure_supported()?;
        Ok(serde_json::to_string_pretty(&archive)?)
    }

    pub fn load_from_path(path: impl AsRef<Path>) -> Result<Self, RuntimeSessionArchiveError> {
        io::load_from_path(path)
    }

    pub fn load_or_empty_from_path(
        path: impl AsRef<Path>,
    ) -> Result<Self, RuntimeSessionArchiveError> {
        io::load_or_empty_from_path(path)
    }

    pub fn load_manifest_from_path(
        path: impl AsRef<Path>,
    ) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
        io::load_manifest_from_path(path)
    }

    pub fn single_slot_archive_from_path(
        path: impl AsRef<Path>,
        slot_id: &str,
    ) -> Result<Self, RuntimeSessionArchiveError> {
        io::single_slot_archive_from_path(path, slot_id)
    }

    pub fn slot_summary_from_path(
        path: impl AsRef<Path>,
        slot_id: &str,
    ) -> Result<Option<RuntimeSessionSlotSummary>, RuntimeSessionArchiveError> {
        io::slot_summary_from_path(path, slot_id)
    }

    pub fn contains_slot_from_path(
        path: impl AsRef<Path>,
        slot_id: &str,
    ) -> Result<bool, RuntimeSessionArchiveError> {
        io::contains_slot_from_path(path, slot_id)
    }

    pub fn slot_ids_from_path(
        path: impl AsRef<Path>,
    ) -> Result<Vec<String>, RuntimeSessionArchiveError> {
        io::slot_ids_from_path(path)
    }

    pub fn slots_with_tag_from_path(
        path: impl AsRef<Path>,
        tag: &str,
    ) -> Result<Vec<RuntimeSessionSlotSummary>, RuntimeSessionArchiveError> {
        io::slots_with_tag_from_path(path, tag)
    }

    pub fn slots_matching_display_name_from_path(
        path: impl AsRef<Path>,
        query: &str,
    ) -> Result<Vec<RuntimeSessionSlotSummary>, RuntimeSessionArchiveError> {
        io::slots_matching_display_name_from_path(path, query)
    }

    pub fn latest_updated_slot_id_from_path(
        path: impl AsRef<Path>,
    ) -> Result<Option<String>, RuntimeSessionArchiveError> {
        io::latest_updated_slot_id_from_path(path)
    }

    pub fn oldest_updated_slot_id_from_path(
        path: impl AsRef<Path>,
    ) -> Result<Option<String>, RuntimeSessionArchiveError> {
        io::oldest_updated_slot_id_from_path(path)
    }

    pub fn latest_updated_slot_id_with_tag_from_path(
        path: impl AsRef<Path>,
        tag: &str,
    ) -> Result<Option<String>, RuntimeSessionArchiveError> {
        io::latest_updated_slot_id_with_tag_from_path(path, tag)
    }

    pub fn oldest_updated_slot_id_with_tag_from_path(
        path: impl AsRef<Path>,
        tag: &str,
    ) -> Result<Option<String>, RuntimeSessionArchiveError> {
        io::oldest_updated_slot_id_with_tag_from_path(path, tag)
    }

    pub fn statistics_from_path(
        path: impl AsRef<Path>,
    ) -> Result<RuntimeSessionArchiveStatistics, RuntimeSessionArchiveError> {
        io::statistics_from_path(path)
    }

    pub fn inspect_path(path: impl AsRef<Path>) -> RuntimeSessionArchivePathStatus {
        io::inspect_path(path)
    }

    pub fn save_to_path(&self, path: impl AsRef<Path>) -> Result<(), RuntimeSessionArchiveError> {
        io::save_to_path(self, path)
    }

    pub fn save_to_path_atomically(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<(), RuntimeSessionArchiveError> {
        io::save_to_path_atomically(self, path)
    }

    pub fn save_single_slot_archive_from_path_atomically(
        source_path: impl AsRef<Path>,
        slot_id: &str,
        target_path: impl AsRef<Path>,
    ) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
        io::save_single_slot_archive_from_path_atomically(source_path, slot_id, target_path)
    }

    pub fn save_single_slot_archive_to_path_atomically(
        &self,
        slot_id: &str,
        target_path: impl AsRef<Path>,
    ) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
        let archive = self.single_slot_archive(slot_id)?;
        archive.save_to_path_atomically(target_path)?;
        archive.manifest()
    }

    pub fn capture_world_slot_to_path_atomically(
        path: impl AsRef<Path>,
        slot_id: impl Into<String>,
        world: &World,
        metadata: RuntimeSessionMetadata,
    ) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
        io::capture_world_slot_to_path_atomically(path, slot_id, world, metadata)
    }

    pub fn capture_level_slot_to_path_atomically(
        path: impl AsRef<Path>,
        slot_id: impl Into<String>,
        level: &LevelSystem,
    ) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
        io::capture_level_slot_to_path_atomically(path, slot_id, level)
    }

    pub fn restore_slot_from_path_to_empty_world(
        path: impl AsRef<Path>,
        slot_id: &str,
    ) -> Result<World, RuntimeSessionArchiveError> {
        io::restore_slot_from_path_to_empty_world(path, slot_id)
    }

    pub fn restore_slot_from_path_into_level(
        path: impl AsRef<Path>,
        slot_id: &str,
        level: &LevelSystem,
    ) -> Result<RuntimeSessionLevelRestoreReport, RuntimeSessionArchiveError> {
        io::restore_slot_from_path_into_level(path, slot_id, level)
    }

    pub fn apply_slot_from_path_to_world(
        path: impl AsRef<Path>,
        slot_id: &str,
        world: &mut World,
    ) -> Result<EntityRemap, RuntimeSessionArchiveError> {
        io::apply_slot_from_path_to_world(path, slot_id, world)
    }

    pub fn apply_slot_from_path_to_level(
        path: impl AsRef<Path>,
        slot_id: &str,
        level: &LevelSystem,
    ) -> Result<EntityRemap, RuntimeSessionArchiveError> {
        io::apply_slot_from_path_to_level(path, slot_id, level)
    }

    pub fn diff_slot_from_path_with_world(
        path: impl AsRef<Path>,
        slot_id: &str,
        world: &World,
    ) -> Result<RuntimeSessionSlotDiffReport, RuntimeSessionArchiveError> {
        io::diff_slot_from_path_with_world(path, slot_id, world)
    }

    pub fn diff_slot_from_path_with_level(
        path: impl AsRef<Path>,
        slot_id: &str,
        level: &LevelSystem,
    ) -> Result<RuntimeSessionSlotDiffReport, RuntimeSessionArchiveError> {
        io::diff_slot_from_path_with_level(path, slot_id, level)
    }

    pub fn rename_slot_at_path_atomically(
        path: impl AsRef<Path>,
        old_slot_id: &str,
        new_slot_id: impl Into<String>,
    ) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
        io::rename_slot_at_path_atomically(path, old_slot_id, new_slot_id)
    }

    pub fn update_slot_metadata_at_path_atomically(
        path: impl AsRef<Path>,
        slot_id: &str,
        metadata: RuntimeSessionMetadata,
    ) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
        io::update_slot_metadata_at_path_atomically(path, slot_id, metadata)
    }

    pub fn touch_slot_at_path_atomically(
        path: impl AsRef<Path>,
        slot_id: &str,
        updated_at_unix_millis: u64,
    ) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
        io::touch_slot_at_path_atomically(path, slot_id, updated_at_unix_millis)
    }

    pub fn remove_slot_at_path_atomically(
        path: impl AsRef<Path>,
        slot_id: &str,
    ) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
        io::remove_slot_at_path_atomically(path, slot_id)
    }

    pub fn copy_slot_at_path_atomically(
        path: impl AsRef<Path>,
        source_slot_id: &str,
        new_slot_id: impl Into<String>,
    ) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
        io::copy_slot_at_path_atomically(path, source_slot_id, new_slot_id)
    }

    pub fn copy_slot_with_metadata_at_path_atomically(
        path: impl AsRef<Path>,
        source_slot_id: &str,
        new_slot_id: impl Into<String>,
        metadata: RuntimeSessionMetadata,
    ) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
        io::copy_slot_with_metadata_at_path_atomically(path, source_slot_id, new_slot_id, metadata)
    }

    pub fn import_slot_from_archive_at_path_atomically(
        path: impl AsRef<Path>,
        incoming: &RuntimeSessionArchive,
        source_slot_id: &str,
        new_slot_id: impl Into<String>,
    ) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
        io::import_slot_from_archive_at_path_atomically(path, incoming, source_slot_id, new_slot_id)
    }

    pub fn import_slot_from_archive_path_at_path_atomically(
        path: impl AsRef<Path>,
        source_path: impl AsRef<Path>,
        source_slot_id: &str,
        new_slot_id: impl Into<String>,
    ) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
        io::import_slot_from_archive_path_at_path_atomically(
            path,
            source_path,
            source_slot_id,
            new_slot_id,
        )
    }

    pub fn import_slot_from_archive_with_metadata_at_path_atomically(
        path: impl AsRef<Path>,
        incoming: &RuntimeSessionArchive,
        source_slot_id: &str,
        new_slot_id: impl Into<String>,
        metadata: RuntimeSessionMetadata,
    ) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
        io::import_slot_from_archive_with_metadata_at_path_atomically(
            path,
            incoming,
            source_slot_id,
            new_slot_id,
            metadata,
        )
    }

    pub fn import_slot_from_archive_path_with_metadata_at_path_atomically(
        path: impl AsRef<Path>,
        source_path: impl AsRef<Path>,
        source_slot_id: &str,
        new_slot_id: impl Into<String>,
        metadata: RuntimeSessionMetadata,
    ) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
        io::import_slot_from_archive_path_with_metadata_at_path_atomically(
            path,
            source_path,
            source_slot_id,
            new_slot_id,
            metadata,
        )
    }

    pub fn merge_archive_at_path_atomically(
        path: impl AsRef<Path>,
        incoming: &RuntimeSessionArchive,
        policy: RuntimeSessionArchiveMergePolicy,
    ) -> Result<RuntimeSessionArchiveMergeReport, RuntimeSessionArchiveError> {
        io::merge_archive_at_path_atomically(path, incoming, policy)
    }

    pub fn preview_merge_archive_at_path(
        path: impl AsRef<Path>,
        incoming: &RuntimeSessionArchive,
        policy: RuntimeSessionArchiveMergePolicy,
    ) -> Result<RuntimeSessionArchiveMergeReport, RuntimeSessionArchiveError> {
        io::preview_merge_archive_at_path(path, incoming, policy)
    }

    pub fn merge_archive_from_path_at_path_atomically(
        path: impl AsRef<Path>,
        source_path: impl AsRef<Path>,
        policy: RuntimeSessionArchiveMergePolicy,
    ) -> Result<RuntimeSessionArchiveMergeReport, RuntimeSessionArchiveError> {
        io::merge_archive_from_path_at_path_atomically(path, source_path, policy)
    }

    pub fn preview_merge_archive_from_path_at_path(
        path: impl AsRef<Path>,
        source_path: impl AsRef<Path>,
        policy: RuntimeSessionArchiveMergePolicy,
    ) -> Result<RuntimeSessionArchiveMergeReport, RuntimeSessionArchiveError> {
        io::preview_merge_archive_from_path_at_path(path, source_path, policy)
    }

    pub fn prune_slots_at_path_atomically(
        path: impl AsRef<Path>,
        policy: RuntimeSessionArchiveRetentionPolicy,
    ) -> Result<RuntimeSessionArchivePruneReport, RuntimeSessionArchiveError> {
        io::prune_slots_at_path_atomically(path, policy)
    }

    pub fn prune_slots_with_tag_at_path_atomically(
        path: impl AsRef<Path>,
        tag: &str,
        policy: RuntimeSessionArchiveRetentionPolicy,
    ) -> Result<RuntimeSessionArchivePruneReport, RuntimeSessionArchiveError> {
        io::prune_slots_with_tag_at_path_atomically(path, tag, policy)
    }

    pub fn preview_prune_slots_from_path(
        path: impl AsRef<Path>,
        policy: RuntimeSessionArchiveRetentionPolicy,
    ) -> Result<RuntimeSessionArchivePruneReport, RuntimeSessionArchiveError> {
        io::preview_prune_slots_from_path(path, policy)
    }

    pub fn preview_prune_slots_with_tag_from_path(
        path: impl AsRef<Path>,
        tag: &str,
        policy: RuntimeSessionArchiveRetentionPolicy,
    ) -> Result<RuntimeSessionArchivePruneReport, RuntimeSessionArchiveError> {
        io::preview_prune_slots_with_tag_from_path(path, tag, policy)
    }

    pub fn capture_world_slot(
        &mut self,
        slot_id: impl Into<String>,
        world: &World,
        metadata: RuntimeSessionMetadata,
    ) -> Result<(), RuntimeSessionArchiveError> {
        self.upsert_slot(RuntimeSessionSlot::from_world_with_metadata(
            slot_id, world, metadata,
        )?)
    }

    pub fn capture_level_slot(
        &mut self,
        slot_id: impl Into<String>,
        level: &LevelSystem,
    ) -> Result<(), RuntimeSessionArchiveError> {
        self.upsert_slot(RuntimeSessionSlot::from_level(slot_id, level)?)
    }

    pub fn push_slot(
        &mut self,
        mut slot: RuntimeSessionSlot,
    ) -> Result<(), RuntimeSessionArchiveError> {
        validate_canonical_slot_id(&slot.slot_id)?;
        slot.metadata.normalize();
        slot.scene.ensure_supported()?;
        if self.slot(&slot.slot_id).is_some() {
            return Err(RuntimeSessionArchiveError::DuplicateSlotId {
                slot_id: slot.slot_id,
            });
        }
        self.slots.push(slot);
        self.sort_slots();
        Ok(())
    }

    pub fn upsert_slot(
        &mut self,
        mut slot: RuntimeSessionSlot,
    ) -> Result<(), RuntimeSessionArchiveError> {
        validate_canonical_slot_id(&slot.slot_id)?;
        slot.metadata.normalize();
        slot.scene.ensure_supported()?;
        if let Some(existing) = self
            .slots
            .iter_mut()
            .find(|existing| existing.slot_id == slot.slot_id)
        {
            *existing = slot;
        } else {
            self.slots.push(slot);
        }
        self.sort_slots();
        Ok(())
    }

    pub fn remove_slot(&mut self, slot_id: &str) -> Option<RuntimeSessionSlot> {
        let index = self.slots.iter().position(|slot| slot.slot_id == slot_id)?;
        Some(self.slots.remove(index))
    }

    pub fn copy_slot(
        &mut self,
        source_slot_id: &str,
        new_slot_id: impl Into<String>,
    ) -> Result<(), RuntimeSessionArchiveError> {
        let metadata = self.require_slot(source_slot_id)?.metadata.clone();
        self.copy_slot_with_metadata(source_slot_id, new_slot_id, metadata)
    }

    pub fn copy_slot_with_metadata(
        &mut self,
        source_slot_id: &str,
        new_slot_id: impl Into<String>,
        metadata: RuntimeSessionMetadata,
    ) -> Result<(), RuntimeSessionArchiveError> {
        let mut slot = self.require_slot(source_slot_id)?.clone();
        slot.slot_id = normalize_slot_id(new_slot_id.into())?;
        slot.metadata = metadata.normalized();
        self.push_slot(slot)
    }

    pub fn single_slot_archive(&self, slot_id: &str) -> Result<Self, RuntimeSessionArchiveError> {
        self.ensure_supported()?;
        Self::from_slots(vec![self.require_slot(slot_id)?.clone()])
    }

    pub fn import_slot_from_archive(
        &mut self,
        incoming: &RuntimeSessionArchive,
        source_slot_id: &str,
        new_slot_id: impl Into<String>,
    ) -> Result<(), RuntimeSessionArchiveError> {
        let metadata = incoming.require_slot(source_slot_id)?.metadata.clone();
        self.import_slot_from_archive_with_metadata(incoming, source_slot_id, new_slot_id, metadata)
    }

    pub fn import_slot_from_archive_with_metadata(
        &mut self,
        incoming: &RuntimeSessionArchive,
        source_slot_id: &str,
        new_slot_id: impl Into<String>,
        metadata: RuntimeSessionMetadata,
    ) -> Result<(), RuntimeSessionArchiveError> {
        self.ensure_supported()?;
        incoming.ensure_supported()?;
        let mut slot = incoming.require_slot(source_slot_id)?.clone();
        slot.slot_id = normalize_slot_id(new_slot_id.into())?;
        slot.metadata = metadata.normalized();
        self.push_slot(slot)
    }

    pub fn rename_slot(
        &mut self,
        old_slot_id: &str,
        new_slot_id: impl Into<String>,
    ) -> Result<(), RuntimeSessionArchiveError> {
        let new_slot_id = normalize_slot_id(new_slot_id.into())?;
        let slot_index = self
            .slots
            .iter()
            .position(|slot| slot.slot_id == old_slot_id)
            .ok_or_else(|| RuntimeSessionArchiveError::MissingSlot {
                slot_id: old_slot_id.to_string(),
            })?;

        if self.slots[slot_index].slot_id == new_slot_id {
            return Ok(());
        }
        if self
            .slots
            .iter()
            .enumerate()
            .any(|(index, slot)| index != slot_index && slot.slot_id == new_slot_id)
        {
            return Err(RuntimeSessionArchiveError::DuplicateSlotId {
                slot_id: new_slot_id,
            });
        }

        self.slots[slot_index].slot_id = new_slot_id;
        self.sort_slots();
        Ok(())
    }

    pub fn update_slot_metadata(
        &mut self,
        slot_id: &str,
        metadata: RuntimeSessionMetadata,
    ) -> Result<(), RuntimeSessionArchiveError> {
        let slot =
            self.slot_mut(slot_id)
                .ok_or_else(|| RuntimeSessionArchiveError::MissingSlot {
                    slot_id: slot_id.to_string(),
                })?;
        slot.metadata = metadata.normalized();
        Ok(())
    }

    pub fn touch_slot(
        &mut self,
        slot_id: &str,
        updated_at_unix_millis: u64,
    ) -> Result<(), RuntimeSessionArchiveError> {
        let slot =
            self.slot_mut(slot_id)
                .ok_or_else(|| RuntimeSessionArchiveError::MissingSlot {
                    slot_id: slot_id.to_string(),
                })?;
        slot.metadata.updated_at_unix_millis = Some(updated_at_unix_millis);
        Ok(())
    }

    pub fn merge_archive(
        &mut self,
        incoming: &RuntimeSessionArchive,
        policy: RuntimeSessionArchiveMergePolicy,
    ) -> Result<RuntimeSessionArchiveMergeReport, RuntimeSessionArchiveError> {
        let report = self.preview_merge_archive(incoming, policy)?;

        for slot in &incoming.slots {
            let slot_id = slot.slot_id.clone();
            if self.contains_slot(&slot_id) {
                match policy {
                    RuntimeSessionArchiveMergePolicy::RejectConflicts => unreachable!(
                        "reject-conflicts policy scans duplicate slot ids before mutating"
                    ),
                    RuntimeSessionArchiveMergePolicy::KeepExisting => {
                        report.skipped_slot_ids.push(slot_id);
                    }
                    RuntimeSessionArchiveMergePolicy::ReplaceExisting => {
                        self.upsert_slot(slot.clone())?;
                        report.replaced_slot_ids.push(slot_id);
                    }
                }
            } else {
                self.push_slot(slot.clone())?;
                report.inserted_slot_ids.push(slot_id);
            }
        }
        Ok(report)
    }

    pub fn preview_merge_archive(
        &self,
        incoming: &RuntimeSessionArchive,
        policy: RuntimeSessionArchiveMergePolicy,
    ) -> Result<RuntimeSessionArchiveMergeReport, RuntimeSessionArchiveError> {
        merge::preview_merge_archive(self, incoming, policy)
    }

    pub fn prune_slots(
        &mut self,
        policy: RuntimeSessionArchiveRetentionPolicy,
    ) -> Result<RuntimeSessionArchivePruneReport, RuntimeSessionArchiveError> {
        retention::prune_slots(self, policy)
    }

    pub fn preview_prune_slots(
        &self,
        policy: RuntimeSessionArchiveRetentionPolicy,
    ) -> Result<RuntimeSessionArchivePruneReport, RuntimeSessionArchiveError> {
        retention::preview_prune_slots(self, policy)
    }

    pub fn prune_slots_with_tag(
        &mut self,
        tag: &str,
        policy: RuntimeSessionArchiveRetentionPolicy,
    ) -> Result<RuntimeSessionArchivePruneReport, RuntimeSessionArchiveError> {
        retention::prune_slots_with_tag(self, tag, policy)
    }

    pub fn preview_prune_slots_with_tag(
        &self,
        tag: &str,
        policy: RuntimeSessionArchiveRetentionPolicy,
    ) -> Result<RuntimeSessionArchivePruneReport, RuntimeSessionArchiveError> {
        retention::preview_prune_slots_with_tag(self, tag, policy)
    }

    pub fn slot_count(&self) -> usize {
        self.slots.len()
    }

    pub fn is_empty(&self) -> bool {
        self.slots.is_empty()
    }

    pub fn contains_slot(&self, slot_id: &str) -> bool {
        self.slot(slot_id).is_some()
    }

    pub fn slot(&self, slot_id: &str) -> Option<&RuntimeSessionSlot> {
        self.slots.iter().find(|slot| slot.slot_id == slot_id)
    }

    pub fn slots(&self) -> &[RuntimeSessionSlot] {
        &self.slots
    }

    pub fn slot_ids(&self) -> impl Iterator<Item = &str> {
        self.slots.iter().map(|slot| slot.slot_id.as_str())
    }

    pub fn manifest(&self) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
        self.ensure_supported()?;
        let mut slots = self
            .slots
            .iter()
            .map(RuntimeSessionSlot::summary)
            .collect::<Vec<_>>();
        slots.sort_by(|left, right| left.slot_id.cmp(&right.slot_id));
        Ok(RuntimeSessionArchiveManifest {
            format_version: self.format_version,
            slots,
        })
    }

    pub fn latest_updated_slot_id(&self) -> Result<Option<String>, RuntimeSessionArchiveError> {
        Ok(self
            .manifest()?
            .latest_updated_slot()
            .map(|slot| slot.slot_id.clone()))
    }

    pub fn oldest_updated_slot_id(&self) -> Result<Option<String>, RuntimeSessionArchiveError> {
        Ok(self
            .manifest()?
            .oldest_updated_slot()
            .map(|slot| slot.slot_id.clone()))
    }

    pub fn latest_updated_slot_id_with_tag(
        &self,
        tag: &str,
    ) -> Result<Option<String>, RuntimeSessionArchiveError> {
        Ok(self
            .manifest()?
            .latest_updated_slot_with_tag(tag)
            .map(|slot| slot.slot_id.clone()))
    }

    pub fn oldest_updated_slot_id_with_tag(
        &self,
        tag: &str,
    ) -> Result<Option<String>, RuntimeSessionArchiveError> {
        Ok(self
            .manifest()?
            .oldest_updated_slot_with_tag(tag)
            .map(|slot| slot.slot_id.clone()))
    }

    pub fn statistics(
        &self,
    ) -> Result<RuntimeSessionArchiveStatistics, RuntimeSessionArchiveError> {
        self.ensure_supported()?;
        let mut statistics = RuntimeSessionArchiveStatistics {
            format_version: self.format_version,
            slot_count: self.slots.len(),
            ..Default::default()
        };

        for slot in &self.slots {
            let entity_count = slot.scene.entities.len();
            let resource_count = slot.scene.resources.len();
            statistics.total_entity_count += entity_count;
            statistics.total_resource_count += resource_count;
            statistics.max_slot_entity_count = statistics.max_slot_entity_count.max(entity_count);
            statistics.max_slot_resource_count =
                statistics.max_slot_resource_count.max(resource_count);

            if let Some(updated_at) = slot.metadata.updated_at_unix_millis {
                statistics.earliest_updated_at_unix_millis = Some(
                    statistics
                        .earliest_updated_at_unix_millis
                        .map_or(updated_at, |current| current.min(updated_at)),
                );
                statistics.latest_updated_at_unix_millis = Some(
                    statistics
                        .latest_updated_at_unix_millis
                        .map_or(updated_at, |current| current.max(updated_at)),
                );
            } else {
                statistics.untimed_slot_count += 1;
            }
        }

        Ok(statistics)
    }

    pub fn apply_slot(
        &self,
        slot_id: &str,
        world: &mut World,
    ) -> Result<EntityRemap, RuntimeSessionArchiveError> {
        self.require_slot(slot_id)?.apply_to_world(world)
    }

    pub fn apply_slot_to_level(
        &self,
        slot_id: &str,
        level: &LevelSystem,
    ) -> Result<EntityRemap, RuntimeSessionArchiveError> {
        self.require_slot(slot_id)?.apply_to_level(level)
    }

    pub fn restore_slot_to_empty_world(
        &self,
        slot_id: &str,
    ) -> Result<World, RuntimeSessionArchiveError> {
        self.require_slot(slot_id)?.restore_to_empty_world()
    }

    pub fn restore_slot_into_level(
        &self,
        slot_id: &str,
        level: &LevelSystem,
    ) -> Result<RuntimeSessionLevelRestoreReport, RuntimeSessionArchiveError> {
        self.require_slot(slot_id)?.restore_into_level(level)
    }

    pub fn diff_slot_with_world(
        &self,
        slot_id: &str,
        world: &World,
    ) -> Result<RuntimeSessionSlotDiffReport, RuntimeSessionArchiveError> {
        self.require_slot(slot_id)?.diff_world(world)
    }

    pub fn diff_slot_with_level(
        &self,
        slot_id: &str,
        level: &LevelSystem,
    ) -> Result<RuntimeSessionSlotDiffReport, RuntimeSessionArchiveError> {
        self.require_slot(slot_id)?.diff_level(level)
    }

    pub fn ensure_supported(&self) -> Result<(), RuntimeSessionArchiveError> {
        if self.format_version != RUNTIME_SESSION_ARCHIVE_FORMAT_VERSION {
            return Err(RuntimeSessionArchiveError::UnsupportedFormatVersion {
                expected: RUNTIME_SESSION_ARCHIVE_FORMAT_VERSION,
                actual: self.format_version,
            });
        }

        let mut seen = BTreeSet::new();
        for slot in &self.slots {
            validate_canonical_slot_id(&slot.slot_id)?;
            slot.scene.ensure_supported()?;
            if !seen.insert(slot.slot_id.as_str()) {
                return Err(RuntimeSessionArchiveError::DuplicateSlotId {
                    slot_id: slot.slot_id.clone(),
                });
            }
        }
        Ok(())
    }

    fn require_slot(
        &self,
        slot_id: &str,
    ) -> Result<&RuntimeSessionSlot, RuntimeSessionArchiveError> {
        self.slot(slot_id)
            .ok_or_else(|| RuntimeSessionArchiveError::MissingSlot {
                slot_id: slot_id.to_string(),
            })
    }

    // Keep mutable slot access private so callers cannot bypass id sorting,
    // duplicate checks, or metadata normalization.
    fn slot_mut(&mut self, slot_id: &str) -> Option<&mut RuntimeSessionSlot> {
        self.slots.iter_mut().find(|slot| slot.slot_id == slot_id)
    }

    fn sort_slots(&mut self) {
        self.slots
            .sort_by(|left, right| left.slot_id.cmp(&right.slot_id));
    }

    fn normalize_slot_metadata(&mut self) {
        for slot in &mut self.slots {
            slot.metadata.normalize();
        }
    }
}

impl Default for RuntimeSessionArchive {
    fn default() -> Self {
        Self::empty()
    }
}
