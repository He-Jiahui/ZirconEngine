use std::collections::{BTreeMap, BTreeSet};

use gltf::animation::util::ReadOutputs;

use super::gltf_labeled_subassets::{gltf_label_reference, gltf_label_uri};
use crate::asset::{
    AnimationChannelAsset, AnimationChannelKeyAsset, AnimationChannelValueAsset,
    AnimationClipAsset, AnimationClipBoneTrackAsset, AnimationInterpolationAsset,
    AnimationSkeletonAsset, AnimationSkeletonBoneAsset, AssetImportError, AssetImportOutcome,
    AssetReference, AssetUri, DataAsset, DataAssetFormat, ImportedAsset, ImportedAssetEntry,
};

pub(crate) fn add_gltf_animation_and_skin_subassets(
    mut outcome: AssetImportOutcome,
    root_uri: &AssetUri,
    document: &gltf::Document,
    buffers: &[gltf::buffer::Data],
) -> Result<AssetImportOutcome, AssetImportError> {
    let skeleton_labels_by_skin = skeleton_labels_by_skin(document);

    for animation in document.animations() {
        let label = format!("Animation{}", animation.index());
        let uri = gltf_label_uri(root_uri, &label);
        let skin_skeleton_label =
            skin_skeleton_label_for_animation(document, &animation, &skeleton_labels_by_skin);
        let skeleton_label = skin_skeleton_label
            .clone()
            .unwrap_or_else(|| format!("{label}/Skeleton"));
        let skeleton_reference = gltf_label_reference(root_uri, &skeleton_label);
        let clip = animation_clip_from_gltf_animation(
            uri.clone(),
            &animation,
            buffers,
            skeleton_reference.clone(),
        )?;
        let entry = ImportedAssetEntry::new(uri, ImportedAsset::AnimationClip(clip))
            .with_dependency(skeleton_reference.locator);
        outcome = with_root_dependency_and_entry(outcome, entry);

        if skin_skeleton_label.is_none() {
            let skeleton_uri = gltf_label_uri(root_uri, &skeleton_label);
            let skeleton = skeleton_asset_from_gltf_animation(document, &label, &animation);
            outcome = with_root_dependency_and_entry(
                outcome,
                ImportedAssetEntry::new(skeleton_uri, ImportedAsset::AnimationSkeleton(skeleton)),
            );
        }
    }

    for skin in document.skins() {
        let label = format!("Skin{}", skin.index());
        let uri = gltf_label_uri(root_uri, &label);
        let skeleton_uri = gltf_label_uri(root_uri, &format!("{label}/Skeleton"));
        let inverse_bind_matrices = inverse_bind_matrices_for_skin(&skin, buffers)?;
        let matrices_uri = inverse_bind_matrices
            .as_ref()
            .map(|_| gltf_label_uri(root_uri, &format!("{label}/InverseBindMatrices")));
        let skeleton = skeleton_asset_from_gltf_skin(document, &label, &skin);

        let mut skin_entry = ImportedAssetEntry::new(
            uri.clone(),
            ImportedAsset::Data(gltf_skin_data_asset(
                root_uri,
                uri,
                &label,
                &skin,
                Some(&skeleton_uri),
                matrices_uri.as_ref(),
                inverse_bind_matrices
                    .as_ref()
                    .map_or(0, |matrices| matrices.len()),
            )),
        );
        for joint in skin.joints() {
            push_dependency_once(
                &mut skin_entry,
                gltf_label_uri(root_uri, &format!("Node{}", joint.index())),
            );
        }
        if let Some(skeleton) = skin.skeleton() {
            push_dependency_once(
                &mut skin_entry,
                gltf_label_uri(root_uri, &format!("Node{}", skeleton.index())),
            );
        }
        push_dependency_once(&mut skin_entry, skeleton_uri.clone());
        if let Some(matrices_uri) = &matrices_uri {
            push_dependency_once(&mut skin_entry, matrices_uri.clone());
        }
        outcome = with_root_dependency_and_entry(outcome, skin_entry);

        outcome = with_root_dependency_and_entry(
            outcome,
            ImportedAssetEntry::new(skeleton_uri, ImportedAsset::AnimationSkeleton(skeleton)),
        );

        if skin.inverse_bind_matrices().is_some() {
            let matrices_label = format!("{label}/InverseBindMatrices");
            let matrices_uri = matrices_uri.expect("matrix uri should exist when accessor exists");
            let inverse_bind_matrices =
                inverse_bind_matrices.expect("matrix payload should exist when accessor exists");
            outcome = with_root_dependency_and_entry(
                outcome,
                ImportedAssetEntry::new(
                    matrices_uri.clone(),
                    ImportedAsset::Data(gltf_inverse_bind_matrices_data_asset(
                        matrices_uri,
                        &matrices_label,
                        inverse_bind_matrices,
                    )),
                ),
            );
        }
    }

    Ok(outcome)
}

fn skeleton_labels_by_skin(document: &gltf::Document) -> BTreeMap<usize, String> {
    document
        .skins()
        .map(|skin| (skin.index(), format!("Skin{}/Skeleton", skin.index())))
        .collect()
}

fn skin_skeleton_label_for_animation(
    document: &gltf::Document,
    animation: &gltf::Animation<'_>,
    skeleton_labels_by_skin: &BTreeMap<usize, String>,
) -> Option<String> {
    let target_nodes = animation
        .channels()
        .map(|channel| channel.target().node().index())
        .collect::<BTreeSet<_>>();

    document
        .skins()
        .find(|skin| {
            skin.joints()
                .any(|joint| target_nodes.contains(&joint.index()))
        })
        .and_then(|skin| skeleton_labels_by_skin.get(&skin.index()))
        .cloned()
}

fn skeleton_asset_from_gltf_skin(
    document: &gltf::Document,
    label: &str,
    skin: &gltf::Skin<'_>,
) -> AnimationSkeletonAsset {
    let parent_node_indices = parent_node_indices(document);
    let joint_indices = skin
        .joints()
        .enumerate()
        .map(|(bone_index, joint)| (joint.index(), bone_index as u32))
        .collect::<BTreeMap<_, _>>();
    let bones = skin
        .joints()
        .map(|joint| {
            let (local_translation, local_rotation, local_scale) = joint.transform().decomposed();
            let node_index = joint.index();
            let parent_index = parent_node_indices
                .get(&node_index)
                .copied()
                .and_then(|parent_node_index| joint_indices.get(&parent_node_index).copied());
            AnimationSkeletonBoneAsset {
                name: node_bone_name(&joint),
                parent_index,
                local_translation,
                local_rotation,
                local_scale,
            }
        })
        .collect();

    AnimationSkeletonAsset {
        name: skin
            .name()
            .map(str::to_owned)
            .or_else(|| Some(label.to_string())),
        bones,
    }
}

fn skeleton_asset_from_gltf_animation(
    document: &gltf::Document,
    label: &str,
    animation: &gltf::Animation<'_>,
) -> AnimationSkeletonAsset {
    let parent_node_indices = parent_node_indices(document);
    let mut required_nodes = animation
        .channels()
        .map(|channel| channel.target().node().index())
        .collect::<BTreeSet<_>>();
    for mut node_index in required_nodes.clone() {
        while let Some(parent_index) = parent_node_indices.get(&node_index).copied() {
            if !required_nodes.insert(parent_index) {
                break;
            }
            node_index = parent_index;
        }
    }
    let mut nodes = document
        .nodes()
        .filter(|node| required_nodes.contains(&node.index()))
        .collect::<Vec<_>>();
    nodes.sort_by_key(|node| (node_depth(node.index(), &parent_node_indices), node.index()));
    let bone_indices = nodes
        .iter()
        .enumerate()
        .map(|(bone_index, node)| (node.index(), bone_index as u32))
        .collect::<BTreeMap<_, _>>();
    let bones = nodes
        .into_iter()
        .map(|node| {
            let (local_translation, local_rotation, local_scale) = node.transform().decomposed();
            let node_index = node.index();
            let parent_index = parent_node_indices
                .get(&node_index)
                .copied()
                .and_then(|parent_node_index| bone_indices.get(&parent_node_index).copied());
            AnimationSkeletonBoneAsset {
                name: node_bone_name(&node),
                parent_index,
                local_translation,
                local_rotation,
                local_scale,
            }
        })
        .collect();

    AnimationSkeletonAsset {
        name: animation
            .name()
            .map(str::to_owned)
            .or_else(|| Some(label.to_string())),
        bones,
    }
}

fn parent_node_indices(document: &gltf::Document) -> BTreeMap<usize, usize> {
    let mut parents = BTreeMap::new();
    for node in document.nodes() {
        for child in node.children() {
            parents.insert(child.index(), node.index());
        }
    }
    parents
}

fn node_depth(node_index: usize, parent_node_indices: &BTreeMap<usize, usize>) -> usize {
    let mut depth = 0;
    let mut current = node_index;
    while let Some(parent) = parent_node_indices.get(&current) {
        depth += 1;
        current = *parent;
    }
    depth
}

fn animation_clip_from_gltf_animation(
    uri: AssetUri,
    animation: &gltf::Animation<'_>,
    buffers: &[gltf::buffer::Data],
    skeleton: AssetReference,
) -> Result<AnimationClipAsset, AssetImportError> {
    let mut tracks = BTreeMap::<usize, GltfTrackBuilder>::new();
    let mut duration_seconds = 0.0_f32;

    for channel in animation.channels() {
        let node = channel.target().node();
        let node_index = node.index();
        let track = tracks
            .entry(node_index)
            .or_insert_with(|| GltfTrackBuilder::new(&node));
        let channel_asset = channel_asset_from_gltf_channel(&channel, buffers)?;
        duration_seconds = duration_seconds.max(last_channel_time(&channel_asset));
        match channel.target().property() {
            gltf::animation::Property::Translation => {
                track.translation = Some(channel_asset);
            }
            gltf::animation::Property::Rotation => {
                track.rotation = Some(channel_asset);
            }
            gltf::animation::Property::Scale => {
                track.scale = Some(channel_asset);
            }
            gltf::animation::Property::MorphTargetWeights => {}
        }
    }

    Ok(AnimationClipAsset {
        name: animation
            .name()
            .map(str::to_owned)
            .or_else(|| uri.label().map(str::to_owned)),
        skeleton,
        duration_seconds,
        tracks: tracks
            .into_values()
            .map(GltfTrackBuilder::into_clip_track)
            .collect(),
        event_tracks: Vec::new(),
    })
}

fn channel_asset_from_gltf_channel(
    channel: &gltf::animation::Channel<'_>,
    buffers: &[gltf::buffer::Data],
) -> Result<AnimationChannelAsset, AssetImportError> {
    let reader = channel.reader(|buffer| Some(&buffers[buffer.index()].0));
    let times = reader
        .read_inputs()
        .ok_or_else(|| {
            AssetImportError::Parse(format!(
                "gltf Animation{} channel targeting Node{} missing sampler input",
                channel.animation().index(),
                channel.target().node().index()
            ))
        })?
        .collect::<Vec<_>>();
    let interpolation = animation_interpolation_from_gltf(channel.sampler().interpolation());
    let outputs = reader.read_outputs().ok_or_else(|| {
        AssetImportError::Parse(format!(
            "gltf Animation{} channel targeting Node{} missing sampler output",
            channel.animation().index(),
            channel.target().node().index()
        ))
    })?;
    match outputs {
        ReadOutputs::Translations(values) => {
            let values = values
                .map(AnimationChannelValueAsset::Vec3)
                .collect::<Vec<_>>();
            channel_from_times_and_values(channel, interpolation, times, values)
        }
        ReadOutputs::Rotations(values) => {
            let values = values
                .into_f32()
                .map(AnimationChannelValueAsset::Quaternion)
                .collect::<Vec<_>>();
            channel_from_times_and_values(channel, interpolation, times, values)
        }
        ReadOutputs::Scales(values) => {
            let values = values
                .map(AnimationChannelValueAsset::Vec3)
                .collect::<Vec<_>>();
            channel_from_times_and_values(channel, interpolation, times, values)
        }
        ReadOutputs::MorphTargetWeights(_) => Err(AssetImportError::Parse(format!(
            "gltf Animation{} channel targeting Node{} morph target weights are not supported by AnimationClip bone tracks",
            channel.animation().index(),
            channel.target().node().index()
        ))),
    }
}

fn channel_from_times_and_values(
    channel: &gltf::animation::Channel<'_>,
    interpolation: AnimationInterpolationAsset,
    times: Vec<f32>,
    values: Vec<AnimationChannelValueAsset>,
) -> Result<AnimationChannelAsset, AssetImportError> {
    let keys = match channel.sampler().interpolation() {
        gltf::animation::Interpolation::CubicSpline => {
            if values.len() != times.len() * 3 {
                return Err(channel_value_count_error(
                    channel,
                    times.len(),
                    values.len(),
                ));
            }
            times
                .into_iter()
                .enumerate()
                .map(|(index, time_seconds)| AnimationChannelKeyAsset {
                    time_seconds,
                    in_tangent: values.get(index * 3).cloned(),
                    value: values
                        .get(index * 3 + 1)
                        .expect("validated cubic key value")
                        .clone(),
                    out_tangent: values.get(index * 3 + 2).cloned(),
                })
                .collect()
        }
        _ => {
            if values.len() != times.len() {
                return Err(channel_value_count_error(
                    channel,
                    times.len(),
                    values.len(),
                ));
            }
            times
                .into_iter()
                .zip(values)
                .map(|(time_seconds, value)| AnimationChannelKeyAsset {
                    time_seconds,
                    value,
                    in_tangent: None,
                    out_tangent: None,
                })
                .collect()
        }
    };
    Ok(AnimationChannelAsset {
        interpolation,
        keys,
    })
}

fn channel_value_count_error(
    channel: &gltf::animation::Channel<'_>,
    time_count: usize,
    value_count: usize,
) -> AssetImportError {
    AssetImportError::Parse(format!(
        "gltf Animation{} channel targeting Node{} has {time_count} sampler input keys but {value_count} output values",
        channel.animation().index(),
        channel.target().node().index()
    ))
}

fn animation_interpolation_from_gltf(
    interpolation: gltf::animation::Interpolation,
) -> AnimationInterpolationAsset {
    match interpolation {
        gltf::animation::Interpolation::Step => AnimationInterpolationAsset::Step,
        gltf::animation::Interpolation::Linear => AnimationInterpolationAsset::Linear,
        gltf::animation::Interpolation::CubicSpline => AnimationInterpolationAsset::Hermite,
    }
}

fn last_channel_time(channel: &AnimationChannelAsset) -> f32 {
    channel
        .keys
        .last()
        .map(|key| key.time_seconds)
        .unwrap_or(0.0)
}

fn inverse_bind_matrices_for_skin(
    skin: &gltf::Skin<'_>,
    buffers: &[gltf::buffer::Data],
) -> Result<Option<Vec<[[f32; 4]; 4]>>, AssetImportError> {
    let Some(accessor) = skin.inverse_bind_matrices() else {
        return Ok(None);
    };
    let matrices = skin
        .reader(|buffer| Some(&buffers[buffer.index()].0))
        .read_inverse_bind_matrices()
        .ok_or_else(|| {
            AssetImportError::Parse(format!(
                "gltf Skin{} inverseBindMatrices accessor {} could not be read",
                skin.index(),
                accessor.index()
            ))
        })?
        .collect();
    Ok(Some(matrices))
}

#[derive(Clone, Debug)]
struct GltfTrackBuilder {
    bone_name: String,
    target_id: String,
    bind_translation: [f32; 3],
    bind_rotation: [f32; 4],
    bind_scale: [f32; 3],
    translation: Option<AnimationChannelAsset>,
    rotation: Option<AnimationChannelAsset>,
    scale: Option<AnimationChannelAsset>,
}

impl GltfTrackBuilder {
    fn new(node: &gltf::Node<'_>) -> Self {
        let (bind_translation, bind_rotation, bind_scale) = node.transform().decomposed();
        let bone_name = node_bone_name(node);
        Self {
            target_id: bone_name.clone(),
            bone_name,
            bind_translation,
            bind_rotation,
            bind_scale,
            translation: None,
            rotation: None,
            scale: None,
        }
    }

    fn into_clip_track(self) -> AnimationClipBoneTrackAsset {
        AnimationClipBoneTrackAsset {
            bone_name: self.bone_name,
            target_id: Some(self.target_id),
            translation: self
                .translation
                .unwrap_or_else(|| single_vec3_channel(self.bind_translation)),
            rotation: self
                .rotation
                .unwrap_or_else(|| single_quaternion_channel(self.bind_rotation)),
            scale: self
                .scale
                .unwrap_or_else(|| single_vec3_channel(self.bind_scale)),
        }
    }
}

fn single_vec3_channel(value: [f32; 3]) -> AnimationChannelAsset {
    single_key_channel(AnimationChannelValueAsset::Vec3(value))
}

fn single_quaternion_channel(value: [f32; 4]) -> AnimationChannelAsset {
    single_key_channel(AnimationChannelValueAsset::Quaternion(value))
}

fn single_key_channel(value: AnimationChannelValueAsset) -> AnimationChannelAsset {
    AnimationChannelAsset {
        interpolation: AnimationInterpolationAsset::Step,
        keys: vec![AnimationChannelKeyAsset {
            time_seconds: 0.0,
            value,
            in_tangent: None,
            out_tangent: None,
        }],
    }
}

fn node_bone_name(node: &gltf::Node<'_>) -> String {
    node.name()
        .map(|name| format!("Node{}:{name}", node.index()))
        .unwrap_or_else(|| format!("Node{}", node.index()))
}

fn gltf_skin_data_asset(
    root_uri: &AssetUri,
    uri: AssetUri,
    label: &str,
    skin: &gltf::Skin<'_>,
    skeleton_asset_uri: Option<&AssetUri>,
    inverse_bind_matrices_uri: Option<&AssetUri>,
    inverse_bind_matrix_count: usize,
) -> DataAsset {
    let joints = skin
        .joints()
        .map(|joint| {
            serde_json::json!({
                "node_index": joint.index(),
                "node": gltf_label_uri(root_uri, &format!("Node{}", joint.index())).to_string(),
                "name": joint.name(),
                "bone_name": node_bone_name(&joint),
            })
        })
        .collect::<Vec<_>>();
    let joint_count = joints.len();
    let skeleton = skin.skeleton().map(|node| {
        serde_json::json!({
            "node_index": node.index(),
            "node": gltf_label_uri(root_uri, &format!("Node{}", node.index())).to_string(),
            "name": node.name(),
            "bone_name": node_bone_name(&node),
        })
    });
    let canonical_json = serde_json::json!({
        "kind": "gltf_skin",
        "label": label,
        "skin_index": skin.index(),
        "name": skin.name(),
        "skeleton": skeleton,
        "skeleton_asset": skeleton_asset_uri.map(ToString::to_string),
        "joints": joints,
        "joint_count": joint_count,
        "inverse_bind_matrices": inverse_bind_matrices_uri.map(ToString::to_string),
        "inverse_bind_matrix_count": inverse_bind_matrix_count,
    });
    json_data_asset(uri, canonical_json)
}

fn gltf_inverse_bind_matrices_data_asset(
    uri: AssetUri,
    label: &str,
    inverse_bind_matrices: Vec<[[f32; 4]; 4]>,
) -> DataAsset {
    json_data_asset(
        uri,
        serde_json::json!({
            "kind": "gltf_inverse_bind_matrices",
            "label": label,
            "matrix_count": inverse_bind_matrices.len(),
            "matrices": inverse_bind_matrices,
        }),
    )
}

fn json_data_asset(uri: AssetUri, canonical_json: serde_json::Value) -> DataAsset {
    DataAsset {
        uri,
        format: DataAssetFormat::Json,
        text: serde_json::to_string_pretty(&canonical_json)
            .expect("generated gltf data JSON should serialize"),
        canonical_json,
    }
}

fn with_root_dependency_and_entry(
    outcome: AssetImportOutcome,
    entry: ImportedAssetEntry,
) -> AssetImportOutcome {
    outcome
        .with_dependency(entry.locator.clone())
        .with_entry(entry)
}

fn push_dependency_once(entry: &mut ImportedAssetEntry, locator: AssetUri) {
    if !entry.dependencies.contains(&locator) {
        entry.dependencies.push(locator);
    }
}
