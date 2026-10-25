use std::collections::HashSet;

use zircon_runtime::core::framework::animation::{
    AnimationSkeletonAsset, AnimationSkeletonBoneAsset,
};
use zircon_runtime::core::math::Mat4;
use zircon_runtime::core::resource::{
    AnimationSkeletonMarker, ResourceActiveSnapshotError, ResourceActiveVersionKey, ResourceHandle,
    ResourceManager, ResourceRecord, ResourceSnapshot,
};

use super::hierarchy::SkeletonReferenceHierarchy;
use super::{
    SkeletonBoneReferenceDetails, SkeletonBoneReferenceParent, SkeletonBoneReveal,
    SkeletonReferenceHierarchyError, SkeletonReferenceSpace, SkeletonReferenceTransformError,
};

/// A source joint slot in one installed payload, including the resource owner's ABA identity.
///
/// Display names, filtered row positions and hierarchy paths are never bone identities. This key is
/// process-local; persisted sockets and virtual bones require their own authoring asset identities.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct SkeletonBoneKey {
    version: ResourceActiveVersionKey,
    revision: u64,
    joint_index: u32,
}

impl SkeletonBoneKey {
    pub(crate) fn resource_version(&self) -> &ResourceActiveVersionKey {
        &self.version
    }

    pub(crate) fn revision(&self) -> u64 {
        self.revision
    }

    pub(crate) fn joint_index(&self) -> u32 {
        self.joint_index
    }
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum SkeletonReferenceError {
    #[error("installed skeleton resource cannot be read: {0:?}")]
    Snapshot(ResourceActiveSnapshotError),
    #[error(transparent)]
    Hierarchy(SkeletonReferenceHierarchyError),
}

#[derive(Debug)]
pub(crate) struct SkeletonBoneRow<'a> {
    pub(crate) key: SkeletonBoneKey,
    pub(crate) parent: Option<SkeletonBoneKey>,
    pub(crate) name: &'a str,
    pub(crate) depth: usize,
    pub(crate) has_children: bool,
    pub(crate) expanded: bool,
    pub(crate) matches_query: bool,
}

/// Owns one immutable, atomically paired resource snapshot and its derived tree.
///
/// A retained reference can display a last installed payload during reload. It does not prove
/// current catalog readiness, hold a runtime residency lease, or authorize a pick/edit operation.
/// Event consumers must read the current expected version again through the shared host.
#[derive(Debug)]
pub(crate) struct SkeletonReference {
    snapshot: ResourceSnapshot<AnimationSkeletonAsset>,
    version: ResourceActiveVersionKey,
    hierarchy: SkeletonReferenceHierarchy,
}

impl SkeletonReference {
    pub(crate) fn read(
        resources: &ResourceManager,
        expected: &ResourceActiveVersionKey,
    ) -> Result<Self, SkeletonReferenceError> {
        let snapshot = resources
            .snapshot_expected::<AnimationSkeletonMarker, AnimationSkeletonAsset>(
                ResourceHandle::new(expected.id()),
                expected,
            )
            .map_err(SkeletonReferenceError::Snapshot)?;
        let hierarchy = SkeletonReferenceHierarchy::build(&snapshot.bones)
            .map_err(SkeletonReferenceError::Hierarchy)?;
        Ok(Self {
            snapshot,
            version: expected.clone(),
            hierarchy,
        })
    }

    pub(crate) fn record(&self) -> &ResourceRecord {
        self.snapshot.record()
    }

    pub(crate) fn joint_count(&self) -> usize {
        self.snapshot.bones.len()
    }

    pub(crate) fn key(&self, joint_index: u32) -> Option<SkeletonBoneKey> {
        let index = usize::try_from(joint_index).ok()?;
        self.snapshot.bones.get(index)?;
        Some(self.key_at(index))
    }

    pub(crate) fn bone(&self, key: &SkeletonBoneKey) -> Option<&AnimationSkeletonBoneAsset> {
        if key.version != self.version || key.revision != self.snapshot.revision() {
            return None;
        }
        self.snapshot
            .bones
            .get(usize::try_from(key.joint_index).ok()?)
    }

    /// Resolve selection against this snapshot, never against visible row positions or names.
    pub(crate) fn details(
        &self,
        key: &SkeletonBoneKey,
    ) -> Option<SkeletonBoneReferenceDetails<'_>> {
        let bone = self.bone(key)?;
        let index = usize::try_from(key.joint_index).ok()?;
        let parent = self.hierarchy.parents[index].map(|parent| SkeletonBoneReferenceParent {
            key: self.key_at(parent),
            name: &self.snapshot.bones[parent].name,
        });
        Some(SkeletonBoneReferenceDetails {
            key: key.clone(),
            name: &bone.name,
            parent,
            reference_local_translation: &bone.local_translation,
            reference_local_rotation: &bone.local_rotation,
            reference_local_scale: &bone.local_scale,
        })
    }

    /// A reference matrix in this snapshot's local or skeleton-root space.
    ///
    /// The root result preserves shear instead of decomposing it back into TRS. Neither space
    /// includes an evaluated pose or preview component/actor transform; in particular, the root
    /// result is not UE's Mesh Relative transform. Old keys remain invalid for a new snapshot.
    pub(crate) fn reference_matrix(
        &self,
        key: &SkeletonBoneKey,
        space: SkeletonReferenceSpace,
    ) -> Result<Option<Mat4>, SkeletonReferenceTransformError> {
        if self.bone(key).is_none() {
            return Ok(None);
        }
        // bone() checked this source slot against the same immutable version and revision.
        let index = key.joint_index as usize;
        super::space::reference_matrix(&self.snapshot.bones, &self.hierarchy, index, space)
            .map(Some)
    }

    /// Plan a reveal against this snapshot and the current plain-name filter.
    ///
    /// Included ancestors are ordered root first; the caller applies them only when its existing
    /// expand-on-selection preference permits. Filtered-out targets stay explicit instead of
    /// clearing the query or selecting a nearby visible row. This does not change selection,
    /// focus or scrolling, and a retained reference is not authority for current preview input.
    pub(crate) fn reveal_for_name_query(
        &self,
        key: &SkeletonBoneKey,
        query: &str,
    ) -> Option<SkeletonBoneReveal> {
        self.bone(key)?;
        let index = key.joint_index as usize;
        let (_, included) =
            super::reveal::name_query_visibility(&self.snapshot.bones, &self.hierarchy, query);
        if !included[index] {
            return Some(SkeletonBoneReveal::FilteredOut { key: key.clone() });
        }
        let ancestors = super::reveal::ancestor_indices(&self.hierarchy, index)
            .into_iter()
            .map(|index| self.key_at(index))
            .collect();
        Some(SkeletonBoneReveal::Included {
            key: key.clone(),
            ancestors,
        })
    }

    /// Plain case-insensitive name substring search with matching ancestors retained.
    ///
    /// Filtering reveals matching branches without changing the caller's expansion state. A
    /// hierarchy traversal orders rows; each row still carries its original source joint slot.
    /// UE expression filters and flatten/hide-parent preferences are separate, unimplemented modes.
    pub(crate) fn rows_for_name_query(
        &self,
        query: &str,
        expanded: &HashSet<SkeletonBoneKey>,
    ) -> Vec<SkeletonBoneRow<'_>> {
        let filtering = !query.is_empty();
        let (matches, visible) =
            super::reveal::name_query_visibility(&self.snapshot.bones, &self.hierarchy, query);

        let mut reachable = vec![false; self.snapshot.bones.len()];
        let mut rows = Vec::new();
        for &index in &self.hierarchy.preorder {
            let parent = self.hierarchy.parents[index];
            reachable[index] = parent.is_none_or(|parent| {
                reachable[parent] && (filtering || expanded.contains(&self.key_at(parent)))
            });
            if !visible[index] || !reachable[index] {
                continue;
            }
            let key = self.key_at(index);
            let has_children = self.hierarchy.children[index]
                .iter()
                .any(|&child| visible[child]);
            rows.push(SkeletonBoneRow {
                parent: parent.map(|parent| self.key_at(parent)),
                name: &self.snapshot.bones[index].name,
                depth: self.hierarchy.depths[index],
                has_children,
                expanded: has_children && (filtering || expanded.contains(&key)),
                matches_query: matches[index],
                key,
            });
        }
        rows
    }

    fn key_at(&self, index: usize) -> SkeletonBoneKey {
        SkeletonBoneKey {
            version: self.version.clone(),
            revision: self.snapshot.revision(),
            // Hierarchy construction already checked the source slot count fits in u32.
            joint_index: index as u32,
        }
    }
}
