use std::collections::BTreeMap;

use zircon_runtime::asset::{AnimationSkeletonAsset, AnimationSkeletonBoneAsset};
use zircon_runtime_interface::resource::ResourceLocator;

#[derive(Clone)]
pub(super) struct DerivedSkeletonAsset {
    pub(super) asset: AnimationSkeletonAsset,
    pub(super) joints: BTreeMap<usize, DerivedJoint>,
}

#[derive(Clone)]
pub(super) struct DerivedJoint {
    pub(super) bone_name: String,
    pub(super) local_translation: [f32; 3],
    pub(super) local_rotation: [f32; 4],
    pub(super) local_scale: [f32; 3],
}

pub(super) fn derive_skeleton_asset(
    skin: &gltf::Skin<'_>,
    parent_indices: &BTreeMap<usize, usize>,
    _skeleton_locator: &ResourceLocator,
    fallback_name: &str,
) -> Result<DerivedSkeletonAsset, String> {
    let joints = skin.joints().collect::<Vec<_>>();
    if joints.is_empty() {
        return Err("gltf skin does not define any joints".to_string());
    }

    let joint_lookup = joints
        .iter()
        .enumerate()
        .map(|(index, joint)| (joint.index(), index as u32))
        .collect::<BTreeMap<_, _>>();

    let mut derived_joints = BTreeMap::new();
    let bones = joints
        .into_iter()
        .map(|joint| {
            let (local_translation, local_rotation, local_scale) = joint.transform().decomposed();
            let bone_name = joint_display_name(&joint);
            derived_joints.insert(
                joint.index(),
                DerivedJoint {
                    bone_name: bone_name.clone(),
                    local_translation,
                    local_rotation,
                    local_scale,
                },
            );
            AnimationSkeletonBoneAsset {
                name: bone_name,
                parent_index: parent_indices
                    .get(&joint.index())
                    .and_then(|parent_index| joint_lookup.get(parent_index))
                    .copied(),
                local_translation,
                local_rotation,
                local_scale,
            }
        })
        .collect();

    Ok(DerivedSkeletonAsset {
        asset: AnimationSkeletonAsset {
            name: skin
                .name()
                .map(str::to_string)
                .or_else(|| Some(fallback_name.to_string())),
            bones,
        },
        joints: derived_joints,
    })
}

pub(super) fn node_parent_indices(document: &gltf::Document) -> BTreeMap<usize, usize> {
    let mut parents = BTreeMap::new();
    for node in document.nodes() {
        for child in node.children() {
            parents.insert(child.index(), node.index());
        }
    }
    parents
}

fn joint_display_name(joint: &gltf::Node<'_>) -> String {
    joint
        .name()
        .map(str::to_string)
        .unwrap_or_else(|| format!("joint_{}", joint.index()))
}
