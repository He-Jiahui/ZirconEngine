use std::collections::HashSet;

use serde::Deserialize;
use zircon_runtime::asset::{DataAsset, DataAssetFormat};
use zircon_runtime::core::resource::{ResourceLocator, ResourceLocatorError, ResourceRecord};

use crate::core::editing::skeleton_reference::SkeletonReference;

#[derive(Debug, thiserror::Error)]
pub(crate) enum GltfSkinDataError {
    #[error("skin data must retain its imported JSON format and record locator")]
    PayloadIdentity,
    #[error("invalid imported skin JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("expected gltf_skin metadata, found {kind}")]
    Kind { kind: String },
    #[error("invalid imported skin reference: {0}")]
    Locator(#[from] ResourceLocatorError),
    #[error("skin index or referenced subasset locator disagrees with the imported source")]
    SubassetIdentity,
    #[error("skin metadata refers to a different skeleton resource")]
    SkeletonIdentity,
    #[error("skin declares {declared} joints, contains {actual}, skeleton has {bones}")]
    JointCount {
        declared: usize,
        actual: usize,
        bones: usize,
    },
    #[error("skin references glTF node {node_index} more than once")]
    DuplicateNode { node_index: u32 },
    #[error("skin joint slot {joint_slot} disagrees with the skeleton source bone name")]
    BoneName { joint_slot: usize },
    #[error("inverse-bind metadata has count {actual}, expected {expected}")]
    InverseBindCount { actual: usize, expected: usize },
}

#[derive(Deserialize)]
struct SkinDocument<'a> {
    kind: &'a str,
    skin_index: usize,
    skeleton_asset: &'a str,
    joint_count: usize,
    #[serde(borrow)]
    joints: Vec<JointDocument<'a>>,
    #[serde(borrow)]
    inverse_bind_matrices: Option<&'a str>,
    inverse_bind_matrix_count: usize,
}

#[derive(Deserialize)]
struct JointDocument<'a> {
    node_index: u32,
    node: &'a str,
    bone_name: &'a str,
}

pub(super) struct SkinNode {
    pub(super) node_index: u32,
    pub(super) node: ResourceLocator,
}

pub(super) struct SkinMetadata {
    pub(super) skin_index: usize,
    pub(super) joints: Vec<SkinNode>,
    pub(super) inverse_bind_matrices: Option<ResourceLocator>,
    pub(super) inverse_bind_matrix_count: usize,
}

pub(super) fn read(
    data: &DataAsset,
    record: &ResourceRecord,
    skeleton: &SkeletonReference,
) -> Result<SkinMetadata, GltfSkinDataError> {
    if data.format != DataAssetFormat::Json || data.uri != record.primary_locator {
        return Err(GltfSkinDataError::PayloadIdentity);
    }
    let document = SkinDocument::deserialize(&data.canonical_json)?;
    if document.kind != "gltf_skin" {
        return Err(GltfSkinDataError::Kind {
            kind: document.kind.to_owned(),
        });
    }
    if record.primary_locator.label() != Some(format!("Skin{}", document.skin_index).as_str()) {
        return Err(GltfSkinDataError::SubassetIdentity);
    }
    let skeleton_locator = ResourceLocator::parse(document.skeleton_asset)?;
    if skeleton_locator != skeleton.record().primary_locator {
        return Err(GltfSkinDataError::SkeletonIdentity);
    }
    validate_subasset(
        &skeleton_locator,
        &record.primary_locator,
        &format!("Skin{}/Skeleton", document.skin_index),
    )?;
    if document.joints.is_empty()
        || document.joint_count != document.joints.len()
        || document.joint_count != skeleton.joint_count()
    {
        return Err(GltfSkinDataError::JointCount {
            declared: document.joint_count,
            actual: document.joints.len(),
            bones: skeleton.joint_count(),
        });
    }
    let inverse_bind_matrices = document
        .inverse_bind_matrices
        .map(ResourceLocator::parse)
        .transpose()?;
    let expected_matrices = if let Some(locator) = &inverse_bind_matrices {
        validate_subasset(
            locator,
            &record.primary_locator,
            &format!("Skin{}/InverseBindMatrices", document.skin_index),
        )?;
        document.joint_count
    } else {
        0
    };
    if document.inverse_bind_matrix_count != expected_matrices {
        return Err(GltfSkinDataError::InverseBindCount {
            actual: document.inverse_bind_matrix_count,
            expected: expected_matrices,
        });
    }
    let mut nodes = HashSet::with_capacity(document.joints.len());
    let mut joints = Vec::with_capacity(document.joints.len());
    for (slot, joint) in document.joints.into_iter().enumerate() {
        if !nodes.insert(joint.node_index) {
            return Err(GltfSkinDataError::DuplicateNode {
                node_index: joint.node_index,
            });
        }
        let node = ResourceLocator::parse(joint.node)?;
        validate_subasset(
            &node,
            &record.primary_locator,
            &format!("Node{}", joint.node_index),
        )?;
        let key = skeleton
            .key(slot as u32)
            .expect("validated source joint count");
        if skeleton
            .bone(&key)
            .expect("source key from same reference")
            .name
            != joint.bone_name
        {
            return Err(GltfSkinDataError::BoneName { joint_slot: slot });
        }
        joints.push(SkinNode {
            node_index: joint.node_index,
            node,
        });
    }
    Ok(SkinMetadata {
        skin_index: document.skin_index,
        joints,
        inverse_bind_matrices,
        inverse_bind_matrix_count: document.inverse_bind_matrix_count,
    })
}

fn validate_subasset(
    locator: &ResourceLocator,
    source: &ResourceLocator,
    label: &str,
) -> Result<(), GltfSkinDataError> {
    // Validate the producer's explicit source references. Never derive a registry ResourceId or
    // choose a skin/skeleton from a mesh, display name, hierarchy row or another source document.
    if locator.scheme() != source.scheme()
        || locator.path() != source.path()
        || locator.label() != Some(label)
    {
        return Err(GltfSkinDataError::SubassetIdentity);
    }
    Ok(())
}
