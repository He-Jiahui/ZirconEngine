mod snapshot;

pub use snapshot::{
    PrefabAuthoringDiagnostic, PrefabAuthoringDiagnosticKind, PrefabInstanceAuthoringSnapshot,
    PrefabOverrideRow, PrefabSourceAuthoringSnapshot, PrefabSourceComponentConflict,
    PrefabSourceDiagnostic, PrefabSourceDiagnosticKind, PrefabSourceEntityParent,
    PrefabSourceEntityRow, PrefabSourceHierarchyRow, PrefabSourceState,
};

use std::collections::{btree_map::Entry, BTreeMap};

use zircon_runtime::asset::{
    AssetId, AssetKind, AssetReference, AssetRegistryError, AssetRegistryIndex, PrefabAsset,
    PrefabInstanceAsset, PrefabPropertyOverrideAsset, SceneComponentAssetRecord, SceneEntityAsset,
};
use zircon_runtime::core::resource::{
    PrefabMarker, ResourceActiveSnapshotError, ResourceActiveVersionKey, ResourceHandle,
    ResourceManager, ResourceSnapshot,
};

#[derive(Debug)]
pub enum PrefabSourceSnapshotError {
    Reference(AssetRegistryError),
    WrongAssetKind {
        actual: AssetKind,
    },
    NotResident {
        asset_id: AssetId,
    },
    SnapshotRejected {
        asset_id: AssetId,
        error: ResourceActiveSnapshotError,
    },
}

impl std::fmt::Display for PrefabSourceSnapshotError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Reference(error) => write!(formatter, "prefab source reference: {error}"),
            Self::WrongAssetKind { actual } => {
                write!(formatter, "prefab source has asset kind {actual:?}")
            }
            Self::NotResident { asset_id } => {
                write!(
                    formatter,
                    "prefab source {asset_id} has no active resident version"
                )
            }
            Self::SnapshotRejected { asset_id, error } => {
                write!(
                    formatter,
                    "prefab source {asset_id} snapshot rejected: {error:?}"
                )
            }
        }
    }
}

impl std::error::Error for PrefabSourceSnapshotError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Reference(error) => Some(error),
            _ => None,
        }
    }
}

/// One immutable resident read, keeping its native proof and payload together.
///
/// Construction always goes through the owning-project registry and resource
/// manager. This is not another source store, a residency lease, a durable row
/// identity, or permission to publish into a document/view. A retained read may
/// become stale; delayed consumers must revalidate it and their caller context.
#[derive(Debug)]
pub struct PrefabSourceReadSnapshot {
    version_key: ResourceActiveVersionKey,
    source: ResourceSnapshot<PrefabAsset>,
}

/// Resolution inside one immutable source read, never a scene mutation target.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PrefabSourceEntityLookup<'a> {
    Unique(&'a SceneEntityAsset),
    Missing,
    Ambiguous,
}

/// The four persisted definition fields of an opaque component record.
///
/// This borrows native metadata without inventing an instance ID. Equal
/// descriptors may occur more than once in an entity; they are then ambiguous.
/// Neither a descriptor nor a successful read grants a scene mutation permit.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PrefabSourceComponentDescriptor<'a> {
    pub type_id: &'a str,
    pub schema_id: &'a str,
    pub schema_version: u32,
    pub provider_id: &'a str,
}

impl<'a> From<&'a SceneComponentAssetRecord> for PrefabSourceComponentDescriptor<'a> {
    fn from(record: &'a SceneComponentAssetRecord) -> Self {
        Self {
            type_id: &record.type_id,
            schema_id: &record.schema_id,
            schema_version: record.schema_version,
            provider_id: &record.provider_id,
        }
    }
}

/// Exact resolution within one immutable source read, with no first-row fallback.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PrefabSourceComponentLookup<'a> {
    Unique(&'a SceneComponentAssetRecord),
    MissingEntity,
    AmbiguousEntity,
    MissingComponent,
    AmbiguousComponent,
}

impl PrefabSourceReadSnapshot {
    pub fn read(
        source: &AssetReference,
        registry: &AssetRegistryIndex,
        resources: &ResourceManager,
    ) -> Result<Self, PrefabSourceSnapshotError> {
        let (version_key, source) = prefab_source_snapshot(source, registry, resources)?;
        Ok(Self {
            version_key,
            source,
        })
    }

    pub fn version_key(&self) -> &ResourceActiveVersionKey {
        &self.version_key
    }

    pub fn source_snapshot(&self) -> &ResourceSnapshot<PrefabAsset> {
        &self.source
    }

    /// Borrows lossless rows from this read's payload, including an empty scene.
    /// Failure to obtain a read is never represented as an empty source here.
    pub fn authoring_snapshot(&self) -> PrefabSourceAuthoringSnapshot<'_> {
        PrefabSourceAuthoringSnapshot::new(&self.source)
    }

    /// Borrows instance rows only while its source still matches this native read.
    ///
    /// The instance's own prefab reference is resolved through the supplied owning
    /// project registry, then checked against the original key without refreshing
    /// it. Available describes this resident check, not override-path validity,
    /// a residency lease, document/view agreement, or permission to mutate. Delayed
    /// callers must still revalidate their source and owning context before use.
    pub fn instance_authoring_snapshot<'a>(
        &'a self,
        instance: &'a PrefabInstanceAsset,
        registry: &AssetRegistryIndex,
        resources: &ResourceManager,
    ) -> Result<PrefabInstanceAuthoringSnapshot<'a>, PrefabSourceSnapshotError> {
        prefab_source_snapshot_expected(&instance.prefab, registry, resources, &self.version_key)?;
        Ok(PrefabInstanceAuthoringSnapshot::new(
            instance,
            PrefabSourceState::Available,
        ))
    }

    /// Looks up the exact serialized ID in this payload, ignoring names/order.
    /// Duplicate IDs deliberately return no entity. The caller must still
    /// revalidate this read and its document/view before using a delayed result.
    pub fn entity_by_id(&self, entity_id: u64) -> PrefabSourceEntityLookup<'_> {
        let mut matches = self
            .source
            .scene
            .entities
            .iter()
            .filter(|entity| entity.entity == entity_id);
        match (matches.next(), matches.next()) {
            (None, _) => PrefabSourceEntityLookup::Missing,
            (Some(entity), None) => PrefabSourceEntityLookup::Unique(entity),
            (Some(_), Some(_)) => PrefabSourceEntityLookup::Ambiguous,
        }
    }

    /// Borrows a unique opaque record using the exact entity and native metadata.
    /// Payload/reference slots are preserved without decoding or provider loading.
    /// Callers must revalidate this read and their owning view before publishing;
    /// even an equal descriptor cannot authorize rebinding to a newer source.
    pub fn component_by_descriptor(
        &self,
        entity_id: u64,
        descriptor: PrefabSourceComponentDescriptor<'_>,
    ) -> PrefabSourceComponentLookup<'_> {
        let entity = match self.entity_by_id(entity_id) {
            PrefabSourceEntityLookup::Unique(entity) => entity,
            PrefabSourceEntityLookup::Missing => return PrefabSourceComponentLookup::MissingEntity,
            PrefabSourceEntityLookup::Ambiguous => {
                return PrefabSourceComponentLookup::AmbiguousEntity;
            }
        };
        let mut matches = entity
            .components
            .iter()
            .filter(|record| PrefabSourceComponentDescriptor::from(*record) == descriptor);
        match (matches.next(), matches.next()) {
            (None, _) => PrefabSourceComponentLookup::MissingComponent,
            (Some(record), None) => PrefabSourceComponentLookup::Unique(record),
            (Some(_), Some(_)) => PrefabSourceComponentLookup::AmbiguousComponent,
        }
    }

    /// Re-reads only this native version, preserving the original error kind.
    /// It never refreshes the key to silently adopt a replacement payload.
    pub fn revalidate(
        &self,
        source: &AssetReference,
        registry: &AssetRegistryIndex,
        resources: &ResourceManager,
    ) -> Result<Self, PrefabSourceSnapshotError> {
        let source =
            prefab_source_snapshot_expected(source, registry, resources, &self.version_key)?;
        Ok(Self {
            version_key: self.version_key.clone(),
            source,
        })
    }

    /// Visible native preorder using the caller's expansion state for this source.
    /// Only visible branches query their full entity ID; hidden rows and leaves do
    /// not query or change expansion/selection. Returned positions belong to this
    /// immutable read, never persistent targets. Revalidate its original key and
    /// owning view/document context before publishing these rows.
    pub fn visible_hierarchy_rows<'a>(
        &'a self,
        is_expanded: impl Fn(u64) -> bool + 'a,
    ) -> impl Iterator<Item = PrefabSourceHierarchyRow> + 'a {
        visible_source_hierarchy_rows(
            &self.source,
            self.authoring_snapshot().hierarchy_rows,
            is_expanded,
        )
    }

    /// Matches native rows and retains their valid ancestor paths in source order.
    /// The caller owns matching syntax and filter/expansion/selection state. A
    /// direct match does not include otherwise unmatched descendants. Child flags
    /// describe the retained rows; depths and positions still belong to this read.
    /// Retain/revalidate its original key and owning context before publishing.
    pub fn filtered_hierarchy_rows(
        &self,
        matches: impl Fn(&PrefabSourceEntityRow<'_>) -> bool,
    ) -> Vec<PrefabSourceHierarchyRow> {
        let snapshot = self.authoring_snapshot();
        let mut retained: Vec<_> = snapshot.entity_rows.iter().map(matches).collect();
        let mut has_retained_children = vec![false; retained.len()];
        // Reverse preorder visits children before their valid parents. Cut cycle,
        // missing and ambiguous edges never propagate matches to guessed parents.
        for row in snapshot.hierarchy_rows.iter().rev() {
            if retained[row.source_index] {
                if let PrefabSourceEntityParent::Resolved { source_index } =
                    snapshot.entity_rows[row.source_index].parent
                {
                    retained[source_index] = true;
                    has_retained_children[source_index] = true;
                }
            }
        }
        snapshot
            .hierarchy_rows
            .into_iter()
            .filter_map(|mut row| {
                if !retained[row.source_index] {
                    return None;
                }
                row.has_children = has_retained_children[row.source_index];
                Some(row)
            })
            .collect()
    }

    /// Applies the caller's expansion state to the retained matching paths.
    /// Matching runs once per native row before iteration. Only visible retained
    /// branches query expansion; pruned children do not leave an expansion arrow.
    /// This does not perform search-triggered expansion, selection or focus. The
    /// rows still belong to this read and require its original context/key checks.
    pub fn visible_filtered_hierarchy_rows<'a>(
        &'a self,
        matches: impl Fn(&PrefabSourceEntityRow<'_>) -> bool,
        is_expanded: impl Fn(u64) -> bool + 'a,
    ) -> impl Iterator<Item = PrefabSourceHierarchyRow> + 'a {
        visible_source_hierarchy_rows(
            &self.source,
            self.filtered_hierarchy_rows(matches),
            is_expanded,
        )
    }
}

fn visible_source_hierarchy_rows<'a>(
    source: &'a PrefabAsset,
    hierarchy: Vec<PrefabSourceHierarchyRow>,
    is_expanded: impl Fn(u64) -> bool + 'a,
) -> impl Iterator<Item = PrefabSourceHierarchyRow> + 'a {
    let mut collapsed_at = None;
    hierarchy.into_iter().filter(move |row| {
        if collapsed_at.is_some_and(|depth| row.depth > depth) {
            return false;
        }
        collapsed_at = None;
        if row.has_children && !is_expanded(source.scene.entities[row.source_index].entity) {
            collapsed_at = Some(row.depth);
        }
        true
    })
}

/// Reads an already-resident source using the caller's owning-project registry.
///
/// The registry and resources must belong to the same project lifecycle. This does
/// not load a source, register a producer, or grant a document mutation permit.
/// The returned native key detects resource replacement, including remove/recreate;
/// delayed consumers must revalidate it and their owning document/view before use.
/// An unavailable resident payload is distinct from an unresolved asset reference.
pub fn prefab_source_snapshot(
    source: &AssetReference,
    registry: &AssetRegistryIndex,
    resources: &ResourceManager,
) -> Result<(ResourceActiveVersionKey, ResourceSnapshot<PrefabAsset>), PrefabSourceSnapshotError> {
    let asset_id = resolve_prefab_source_id(source, registry)?;
    let version_key = resources
        .projection_snapshot()
        .active()
        .key(asset_id)
        .ok_or(PrefabSourceSnapshotError::NotResident { asset_id })?;
    let asset = prefab_source_snapshot_expected(source, registry, resources, &version_key)?;
    Ok((version_key, asset))
}

/// Re-reads a resident source only if its original native version is still active.
///
/// Unlike a fresh query, this never adopts a replacement version after a delayed
/// selection/read. The reference UUID and catalog kind are checked again, then
/// the resource manager checks the supplied key and payload under its read lock.
/// Removal, replacement and a key from another resource manager are rejected.
///
/// The caller must still validate its owning project/document/view and selection
/// before publishing these rows. This key is temporary proof, not a persistent
/// tree item identity or a document mutation permit.
pub fn prefab_source_snapshot_expected(
    source: &AssetReference,
    registry: &AssetRegistryIndex,
    resources: &ResourceManager,
    expected: &ResourceActiveVersionKey,
) -> Result<ResourceSnapshot<PrefabAsset>, PrefabSourceSnapshotError> {
    let asset_id = resolve_prefab_source_id(source, registry)?;
    resources
        .snapshot_expected::<PrefabMarker, PrefabAsset>(ResourceHandle::new(asset_id), expected)
        .map_err(|error| PrefabSourceSnapshotError::SnapshotRejected { asset_id, error })
}

fn resolve_prefab_source_id(
    source: &AssetReference,
    registry: &AssetRegistryIndex,
) -> Result<AssetId, PrefabSourceSnapshotError> {
    if let Some(entry) = registry.entry_by_uuid(source.uuid) {
        if entry.type_marker() != AssetKind::Prefab {
            return Err(PrefabSourceSnapshotError::WrongAssetKind {
                actual: entry.type_marker(),
            });
        }
    }
    // UUID remains authoritative after relocation; do not fall back to the URL.
    registry
        .resolve_asset_id_for_reference(source.uuid, &source.locator)
        .map_err(PrefabSourceSnapshotError::Reference)
}

enum EffectiveOverrideBucket<'a> {
    One(&'a PrefabPropertyOverrideAsset),
    Many(Vec<&'a PrefabPropertyOverrideAsset>),
}

pub fn effective_prefab_overrides(
    instance: &PrefabInstanceAsset,
) -> Vec<PrefabPropertyOverrideAsset> {
    // Keep every value for a path instead of letting `BTreeMap::insert` silently
    // discard an earlier conflict. Validation rejects duplicate paths before a
    // mutation is admitted, while this read/query surface remains lossless for
    // diagnostics and deterministic replay.
    let mut overrides: BTreeMap<(&str, &str), EffectiveOverrideBucket<'_>> = BTreeMap::new();
    for override_value in &instance.overrides {
        let key = (
            override_value.entity_path.as_str(),
            override_value.property_path.as_str(),
        );
        match overrides.entry(key) {
            Entry::Vacant(entry) => {
                entry.insert(EffectiveOverrideBucket::One(override_value));
            }
            Entry::Occupied(mut entry) => {
                let bucket = entry.get_mut();
                let previous =
                    std::mem::replace(bucket, EffectiveOverrideBucket::One(override_value));
                *bucket = match previous {
                    EffectiveOverrideBucket::One(first) => {
                        EffectiveOverrideBucket::Many(vec![first, override_value])
                    }
                    EffectiveOverrideBucket::Many(mut values) => {
                        values.push(override_value);
                        EffectiveOverrideBucket::Many(values)
                    }
                };
            }
        }
    }
    let mut effective = Vec::with_capacity(instance.overrides.len());
    for bucket in overrides.into_values() {
        match bucket {
            EffectiveOverrideBucket::One(value) => effective.push(value.clone()),
            EffectiveOverrideBucket::Many(values) => {
                effective.extend(values.into_iter().map(Clone::clone));
            }
        }
    }
    effective
}

pub fn validate_prefab_instance(
    instance: &PrefabInstanceAsset,
    source_prefab_available: bool,
) -> Vec<String> {
    let source_state = if source_prefab_available {
        PrefabSourceState::Available
    } else {
        PrefabSourceState::Missing
    };
    let mut diagnostics = snapshot::prefab_instance_diagnostics(instance, source_state)
        .into_iter()
        .map(|diagnostic| diagnostic.message)
        .collect::<Vec<_>>();
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}
