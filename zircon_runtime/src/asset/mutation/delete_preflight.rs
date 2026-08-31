use crate::asset::registry::{AssetRegistryEntry, AssetRegistryIndex};
use crate::asset::{AssetKind, AssetUri, AssetUuid};
use std::collections::HashSet;

/// Immutable asset metadata projected from the authoritative runtime registry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssetMutationAsset {
    uuid: AssetUuid,
    locator: AssetUri,
    kind: AssetKind,
}

impl AssetMutationAsset {
    pub fn uuid(&self) -> AssetUuid {
        self.uuid
    }

    pub fn locator(&self) -> &AssetUri {
        &self.locator
    }

    pub fn kind(&self) -> AssetKind {
        self.kind
    }
}

impl From<&AssetRegistryEntry> for AssetMutationAsset {
    fn from(entry: &AssetRegistryEntry) -> Self {
        Self {
            uuid: entry.uuid(),
            locator: entry.path().clone(),
            kind: entry.type_marker(),
        }
    }
}

/// Topology result for a source-asset delete transaction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssetMutationDeleteDisposition {
    Ready,
    MissingAsset,
    UnsupportedSubasset,
    BlockedByReferencers,
}

/// Runtime registry preflight for deleting an authored source asset.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssetMutationDeletePreflight {
    disposition: AssetMutationDeleteDisposition,
    target: Option<AssetMutationAsset>,
    referencers: Vec<AssetMutationAsset>,
}

impl AssetMutationDeletePreflight {
    /// Evaluates immutable registry topology without touching filesystem or generation state.
    ///
    /// A source mutation transaction must repeat this preflight after acquiring its commit
    /// generation, because this view cannot reserve a source path or referencer set.
    pub fn evaluate(registry: &AssetRegistryIndex, target_uuid: AssetUuid) -> Self {
        let Some(target_entry) = registry.entry_by_uuid(target_uuid) else {
            return Self {
                disposition: AssetMutationDeleteDisposition::MissingAsset,
                target: None,
                referencers: Vec::new(),
            };
        };
        let target = AssetMutationAsset::from(target_entry);
        if target.locator().label().is_some() {
            return Self {
                disposition: AssetMutationDeleteDisposition::UnsupportedSubasset,
                target: Some(target),
                referencers: Vec::new(),
            };
        }

        let source_entries = registry.source_entries(target.locator());
        let source_uuids = source_entries
            .iter()
            .map(|entry| entry.uuid())
            .collect::<HashSet<_>>();
        let mut referencer_uuids = HashSet::new();
        for source_entry in source_entries {
            referencer_uuids.extend(
                registry
                    .get_referencers_by_uuid(source_entry.uuid())
                    .into_iter()
                    .filter(|uuid| !source_uuids.contains(uuid)),
            );
        }
        let mut referencers = referencer_uuids
            .into_iter()
            .filter_map(|uuid| registry.entry_by_uuid(uuid))
            .map(AssetMutationAsset::from)
            .collect::<Vec<_>>();
        referencers.sort_by(|left, right| {
            left.locator
                .cmp(&right.locator)
                .then_with(|| left.uuid.to_string().cmp(&right.uuid.to_string()))
        });
        let disposition = if referencers.is_empty() {
            AssetMutationDeleteDisposition::Ready
        } else {
            AssetMutationDeleteDisposition::BlockedByReferencers
        };
        Self {
            disposition,
            target: Some(target),
            referencers,
        }
    }

    pub fn disposition(&self) -> AssetMutationDeleteDisposition {
        self.disposition
    }

    pub fn target(&self) -> Option<&AssetMutationAsset> {
        self.target.as_ref()
    }

    pub fn referencers(&self) -> &[AssetMutationAsset] {
        &self.referencers
    }
}
