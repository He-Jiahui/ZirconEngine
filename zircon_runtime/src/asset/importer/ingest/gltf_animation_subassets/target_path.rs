use std::collections::BTreeMap;

use crate::asset::AssetImportError;
use crate::core::framework::animation::AnimationSkeletonAsset;

pub(super) fn skeleton_target_ids_by_node(
    animation_index: usize,
    node_count: usize,
    skeleton: &AnimationSkeletonAsset,
    bone_node_indices: &[usize],
) -> Result<Vec<Option<String>>, AssetImportError> {
    if skeleton.bones.len() != bone_node_indices.len() {
        return Err(AssetImportError::Parse(format!(
            "gltf Animation{animation_index} skeleton bone/node mapping length mismatch"
        )));
    }

    let mut target_ids_by_node = vec![None; node_count];
    let mut bone_index_by_target = BTreeMap::new();
    let target_ids = skeleton_bone_target_paths(animation_index, skeleton)?;
    for (bone_index, node_index) in bone_node_indices.iter().copied().enumerate() {
        let target_id = target_ids[bone_index].clone();
        if let Some(first_bone_index) = bone_index_by_target.insert(target_id.clone(), bone_index) {
            return Err(AssetImportError::Parse(format!(
                "gltf Animation{animation_index} skeleton bones {first_bone_index} and {bone_index} share target path '{target_id}'"
            )));
        }
        let target = target_ids_by_node.get_mut(node_index).ok_or_else(|| {
            AssetImportError::Parse(format!(
                "gltf Animation{animation_index} skeleton bone {bone_index} references missing Node{node_index}"
            ))
        })?;
        if target.replace(target_id).is_some() {
            return Err(AssetImportError::Parse(format!(
                "gltf Animation{animation_index} maps Node{node_index} to more than one skeleton bone"
            )));
        }
    }
    Ok(target_ids_by_node)
}

fn skeleton_bone_target_paths(
    animation_index: usize,
    skeleton: &AnimationSkeletonAsset,
) -> Result<Vec<String>, AssetImportError> {
    const UNVISITED: u8 = 0;
    const VISITING: u8 = 1;
    const INDEXED: u8 = 2;

    let mut states = vec![UNVISITED; skeleton.bones.len()];
    let mut target_ids = vec![None; skeleton.bones.len()];
    for start in 0..skeleton.bones.len() {
        if states[start] == INDEXED {
            continue;
        }
        let mut chain = Vec::new();
        let mut current = start;
        let mut parent_target = loop {
            match states[current] {
                UNVISITED => {
                    let bone = &skeleton.bones[current];
                    if bone.name.trim().is_empty()
                        || bone.name != bone.name.trim()
                        || bone.name.contains('/')
                    {
                        return Err(AssetImportError::Parse(format!(
                            "gltf Animation{animation_index} skeleton bone {current} has non-canonical name '{}'",
                            bone.name
                        )));
                    }
                    states[current] = VISITING;
                    chain.push(current);
                    let Some(parent_index) = bone.parent_index.map(|parent| parent as usize) else {
                        break String::new();
                    };
                    if parent_index >= skeleton.bones.len() {
                        return Err(AssetImportError::Parse(format!(
                            "gltf Animation{animation_index} skeleton bone {current} has invalid parent index {parent_index}"
                        )));
                    }
                    current = parent_index;
                }
                VISITING => {
                    return Err(AssetImportError::Parse(format!(
                        "gltf Animation{animation_index} skeleton contains a parent cycle at bone {current}"
                    )));
                }
                INDEXED => break target_ids[current].clone().expect("indexed target path"),
                _ => unreachable!("skeleton target path state is internal"),
            }
        };
        for bone_index in chain.into_iter().rev() {
            if !parent_target.is_empty() {
                parent_target.push('/');
            }
            parent_target.push_str(&skeleton.bones[bone_index].name);
            target_ids[bone_index] = Some(parent_target.clone());
            states[bone_index] = INDEXED;
        }
    }
    Ok(target_ids
        .into_iter()
        .map(|target_id| target_id.expect("every skeleton bone target path is indexed"))
        .collect())
}

#[cfg(test)]
mod tests {
    use crate::core::framework::animation::{AnimationSkeletonAsset, AnimationSkeletonBoneAsset};

    use super::skeleton_target_ids_by_node;

    #[test]
    fn target_paths_follow_skeleton_parents_even_when_parent_bones_come_later() {
        let skeleton = AnimationSkeletonAsset {
            name: Some("skin".to_string()),
            bones: vec![bone("Node5:Hand", Some(1)), bone("Node3:Root", None)],
        };

        let targets = skeleton_target_ids_by_node(7, 8, &skeleton, &[5, 3]).unwrap();

        assert_eq!(targets[3].as_deref(), Some("Node3:Root"));
        assert_eq!(targets[5].as_deref(), Some("Node3:Root/Node5:Hand"));
        assert!(targets[0].is_none());
    }

    #[test]
    fn target_paths_reject_cycles_and_nodes_outside_the_document() {
        let cycle = AnimationSkeletonAsset {
            name: Some("cycle".to_string()),
            bones: vec![bone("A", Some(1)), bone("B", Some(0))],
        };
        let error = skeleton_target_ids_by_node(8, 2, &cycle, &[0, 1]).unwrap_err();
        assert!(error.to_string().contains("parent cycle at bone"));

        let missing_node = AnimationSkeletonAsset {
            name: Some("missing-node".to_string()),
            bones: vec![bone("Root", None)],
        };
        let error = skeleton_target_ids_by_node(9, 1, &missing_node, &[3]).unwrap_err();
        assert!(error.to_string().contains("references missing Node3"));
    }

    fn bone(name: &str, parent_index: Option<u32>) -> AnimationSkeletonBoneAsset {
        AnimationSkeletonBoneAsset {
            name: name.to_string(),
            parent_index,
            local_translation: [0.0; 3],
            local_rotation: [0.0, 0.0, 0.0, 1.0],
            local_scale: [1.0; 3],
        }
    }
}
