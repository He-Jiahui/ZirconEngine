use zircon_runtime::core::framework::animation::AnimationTrackPath;
use zircon_runtime::core::framework::animation::{
    AnimationChannelAsset, AnimationChannelKeyAsset, AnimationChannelValueAsset,
    AnimationInterpolationAsset, AnimationSequenceAsset, AnimationSequenceBindingAsset,
    AnimationSequenceTrackAsset,
};

use super::super::AnimationDocumentMutationError;

pub(super) fn add_key(
    asset: &mut AnimationSequenceAsset,
    track_path: &AnimationTrackPath,
    frame: u32,
) -> Result<bool, AnimationDocumentMutationError> {
    let time_seconds = frame_to_seconds(frame, asset.frames_per_second);
    let track = find_track_mut(asset, track_path)?.ok_or_else(|| {
        AnimationDocumentMutationError::InvalidTrackPath {
            message: format!("missing animation track {track_path}"),
        }
    })?;
    if track
        .channel
        .keys
        .iter()
        .any(|key| (key.time_seconds - time_seconds).abs() <= f32::EPSILON)
    {
        return Ok(false);
    }
    let value = track
        .channel
        .keys
        .last()
        .map(|key| key.value.clone())
        .unwrap_or(AnimationChannelValueAsset::Scalar(0.0));
    track.channel.keys.push(AnimationChannelKeyAsset {
        time_seconds,
        value,
        in_tangent: None,
        out_tangent: None,
    });
    track
        .channel
        .keys
        .sort_by(|left, right| left.time_seconds.total_cmp(&right.time_seconds));
    Ok(true)
}

pub(super) fn remove_key(
    asset: &mut AnimationSequenceAsset,
    track_path: &AnimationTrackPath,
    frame: u32,
) -> Result<bool, AnimationDocumentMutationError> {
    let time_seconds = frame_to_seconds(frame, asset.frames_per_second);
    let track = find_track_mut(asset, track_path)?.ok_or_else(|| {
        AnimationDocumentMutationError::InvalidTrackPath {
            message: format!("missing animation track {track_path}"),
        }
    })?;
    let before = track.channel.keys.len();
    track
        .channel
        .keys
        .retain(|key| (key.time_seconds - time_seconds).abs() > f32::EPSILON);
    Ok(before != track.channel.keys.len())
}

pub(super) fn create_track(
    asset: &mut AnimationSequenceAsset,
    track_path: &AnimationTrackPath,
) -> Result<bool, AnimationDocumentMutationError> {
    let (entity_path, property_path) = split_track_path(track_path)?;
    let binding = if let Some(index) = asset
        .bindings
        .iter()
        .position(|binding| binding.entity_path == entity_path)
    {
        &mut asset.bindings[index]
    } else {
        asset.bindings.push(AnimationSequenceBindingAsset {
            entity_path,
            target_id: None,
            tracks: Vec::new(),
        });
        asset.bindings.last_mut().expect("binding just pushed")
    };
    if binding
        .tracks
        .iter()
        .any(|track| track.property_path == property_path)
    {
        return Ok(false);
    }
    binding.tracks.push(AnimationSequenceTrackAsset {
        property_path,
        channel: default_channel(),
    });
    Ok(true)
}

pub(super) fn remove_track(
    asset: &mut AnimationSequenceAsset,
    track_path: &AnimationTrackPath,
) -> Result<bool, AnimationDocumentMutationError> {
    let (entity_path, property_path) = split_track_path(track_path)?;
    let mut changed = false;
    asset.bindings.retain_mut(|binding| {
        if binding.entity_path != entity_path {
            return true;
        }
        let before = binding.tracks.len();
        binding
            .tracks
            .retain(|track| track.property_path != property_path);
        changed |= before != binding.tracks.len();
        !binding.tracks.is_empty()
    });
    Ok(changed)
}

pub(super) fn rebind_track(
    asset: &mut AnimationSequenceAsset,
    from_track_path: &AnimationTrackPath,
    to_track_path: &AnimationTrackPath,
) -> Result<bool, AnimationDocumentMutationError> {
    let (from_entity, from_property) = split_track_path(from_track_path)?;
    let (to_entity, to_property) = split_track_path(to_track_path)?;
    if from_entity == to_entity && from_property == to_property {
        return Ok(false);
    }
    if asset.bindings.iter().any(|binding| {
        binding.entity_path == to_entity
            && binding
                .tracks
                .iter()
                .any(|track| track.property_path == to_property)
    }) {
        return Ok(false);
    }
    let mut moved_track = None;
    asset.bindings.retain_mut(|binding| {
        if binding.entity_path != from_entity {
            return true;
        }
        if let Some(index) = binding
            .tracks
            .iter()
            .position(|track| track.property_path == from_property)
        {
            moved_track = Some(binding.tracks.remove(index));
        }
        !binding.tracks.is_empty()
    });
    let Some(mut moved_track) = moved_track else {
        return Ok(false);
    };
    moved_track.property_path = to_property;
    let binding = if let Some(index) = asset
        .bindings
        .iter()
        .position(|binding| binding.entity_path == to_entity)
    {
        &mut asset.bindings[index]
    } else {
        asset.bindings.push(AnimationSequenceBindingAsset {
            entity_path: to_entity,
            target_id: None,
            tracks: Vec::new(),
        });
        asset.bindings.last_mut().expect("binding just pushed")
    };
    binding.tracks.push(moved_track);
    Ok(true)
}

fn find_track_mut<'asset>(
    asset: &'asset mut AnimationSequenceAsset,
    track_path: &AnimationTrackPath,
) -> Result<Option<&'asset mut AnimationSequenceTrackAsset>, AnimationDocumentMutationError> {
    let (entity_path, property_path) = split_track_path(track_path)?;
    Ok(asset
        .bindings
        .iter_mut()
        .find(|binding| binding.entity_path == entity_path)
        .and_then(|binding| {
            binding
                .tracks
                .iter_mut()
                .find(|track| track.property_path == property_path)
        }))
}

fn split_track_path(
    track_path: &AnimationTrackPath,
) -> Result<
    (
        zircon_runtime::core::framework::scene::EntityPath,
        zircon_runtime::core::framework::scene::ComponentPropertyPath,
    ),
    AnimationDocumentMutationError,
> {
    track_path
        .split()
        .map_err(|error| AnimationDocumentMutationError::InvalidTrackPath {
            message: error.to_string(),
        })
}

fn default_channel() -> AnimationChannelAsset {
    AnimationChannelAsset {
        interpolation: AnimationInterpolationAsset::Step,
        keys: vec![AnimationChannelKeyAsset {
            time_seconds: 0.0,
            value: AnimationChannelValueAsset::Scalar(0.0),
            in_tangent: None,
            out_tangent: None,
        }],
    }
}

fn frame_to_seconds(frame: u32, frames_per_second: f32) -> f32 {
    frame as f32 / sanitize_frames_per_second(frames_per_second).max(1.0)
}

fn sanitize_frames_per_second(frames_per_second: f32) -> f32 {
    if frames_per_second.is_finite() && frames_per_second > 0.0 {
        frames_per_second
    } else {
        30.0
    }
}
