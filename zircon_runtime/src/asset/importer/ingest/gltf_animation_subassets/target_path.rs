use std::collections::BTreeMap;

use crate::asset::AssetImportError;
use crate::core::framework::animation::AnimationSkeletonAsset;

// 动画 clip 的节点通道使用骨架层级路径作稳定 target ID；
// 导入时拒绝重名路径、重复节点映射、越界节点和父级环，避免运行时绑定歧义。
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

// 路径由骨架 parent_index 决定而非 glTF 节点遍历顺序；显式栈覆盖父骨骼后置的文档。
fn skeleton_bone_target_paths(
    animation_index: usize,
    skeleton: &AnimationSkeletonAsset,
) -> Result<Vec<String>, AssetImportError> {
    const UNVISITED: u8 = 0;
    const VISITING: u8 = 1;
    const INDEXED: u8 = 2;

    let mut states = vec![UNVISITED; skeleton.bones.len()];
    let mut target_ids = vec![None; skeleton.bones.len()];
    let mut chain = Vec::new();
    for start in 0..skeleton.bones.len() {
        if states[start] == INDEXED {
            continue;
        }
        chain.clear();
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
        while let Some(bone_index) = chain.pop() {
            if !parent_target.is_empty() {
                parent_target.push('/');
            }
            parent_target.push_str(&skeleton.bones[bone_index].name);
            states[bone_index] = INDEXED;
            if chain.is_empty() {
                target_ids[bone_index] = Some(parent_target);
                break;
            }
            target_ids[bone_index] = Some(parent_target.clone());
        }
    }
    Ok(target_ids
        .into_iter()
        .map(|target_id| target_id.expect("every skeleton bone target path is indexed"))
        .collect())
}

#[cfg(test)]
#[path = "tests/target_path.rs"]
mod tests;
