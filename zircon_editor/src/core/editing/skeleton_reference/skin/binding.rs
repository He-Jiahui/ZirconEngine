use std::collections::HashMap;

use zircon_runtime::asset::DataAsset;
use zircon_runtime::core::resource::{
    DataMarker, ResourceActiveSnapshotError, ResourceActiveVersionKey, ResourceHandle,
    ResourceLocator, ResourceManager, ResourceRecord, ResourceSnapshot,
};

use super::metadata;
use super::GltfSkinDataError;
use crate::core::editing::skeleton_reference::{SkeletonBoneKey, SkeletonReference};

#[derive(Debug, thiserror::Error)]
pub(crate) enum GltfSkinReferenceError {
    #[error("installed skin data cannot be read: {0:?}")]
    Snapshot(ResourceActiveSnapshotError),
    #[error(transparent)]
    Metadata(#[from] GltfSkinDataError),
}

/// One original skin joint slot, not a display name or a hierarchy traversal position.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GltfSkinJoint {
    pub(crate) key: SkeletonBoneKey,
    pub(crate) node_index: u32,
    pub(crate) node: ResourceLocator,
}

/// Retains one installed Skin Data snapshot and borrows its explicitly associated skeleton.
///
/// The two resources are not captured as one publication transaction. This mapping neither loads
/// the node/mesh/inverse-bind assets nor proves current readiness, a residency lease, an evaluated
/// pose, a preview generation, or permission to pick/edit. The shared host admits those separately.
#[derive(Debug)]
pub(crate) struct GltfSkinReference<'a> {
    snapshot: ResourceSnapshot<DataAsset>,
    version: ResourceActiveVersionKey,
    skeleton: &'a SkeletonReference,
    skin_index: usize,
    joints: Vec<GltfSkinJoint>,
    joint_by_node: HashMap<u32, usize>,
    inverse_bind_matrices: Option<ResourceLocator>,
    inverse_bind_matrix_count: usize,
}

impl<'a> GltfSkinReference<'a> {
    pub(crate) fn read(
        resources: &ResourceManager,
        expected_skin: &ResourceActiveVersionKey,
        skeleton: &'a SkeletonReference,
    ) -> Result<Self, GltfSkinReferenceError> {
        let snapshot = resources
            .snapshot_expected::<DataMarker, DataAsset>(
                ResourceHandle::new(expected_skin.id()),
                expected_skin,
            )
            .map_err(GltfSkinReferenceError::Snapshot)?;
        let metadata = metadata::read(&snapshot, snapshot.record(), skeleton)?;
        let mut joints = Vec::with_capacity(metadata.joints.len());
        let mut joint_by_node = HashMap::with_capacity(metadata.joints.len());
        for (slot, joint) in metadata.joints.into_iter().enumerate() {
            // Metadata validation checked the complete slot count against this same immutable
            // reference. Hierarchy construction has already bounded the skeleton length to u32.
            let key = skeleton
                .key(slot as u32)
                .expect("validated source joint slot");
            joint_by_node.insert(joint.node_index, slot);
            joints.push(GltfSkinJoint {
                key,
                node_index: joint.node_index,
                node: joint.node,
            });
        }
        Ok(Self {
            snapshot,
            version: expected_skin.clone(),
            skeleton,
            skin_index: metadata.skin_index,
            joints,
            joint_by_node,
            inverse_bind_matrices: metadata.inverse_bind_matrices,
            inverse_bind_matrix_count: metadata.inverse_bind_matrix_count,
        })
    }

    pub(crate) fn record(&self) -> &ResourceRecord {
        self.snapshot.record()
    }

    pub(crate) fn resource_version(&self) -> &ResourceActiveVersionKey {
        &self.version
    }

    pub(crate) fn skin_index(&self) -> usize {
        self.skin_index
    }

    pub(crate) fn joints(&self) -> &[GltfSkinJoint] {
        &self.joints
    }

    pub(crate) fn key_for_node(&self, node_index: u32) -> Option<&SkeletonBoneKey> {
        let slot = *self.joint_by_node.get(&node_index)?;
        self.joints.get(slot).map(|joint| &joint.key)
    }

    pub(crate) fn joint_for_key(&self, key: &SkeletonBoneKey) -> Option<&GltfSkinJoint> {
        self.skeleton.bone(key)?;
        self.joints.get(key.joint_index() as usize)
    }

    pub(crate) fn inverse_bind_matrices(&self) -> Option<&ResourceLocator> {
        self.inverse_bind_matrices.as_ref()
    }

    pub(crate) fn inverse_bind_matrix_count(&self) -> usize {
        self.inverse_bind_matrix_count
    }
}
