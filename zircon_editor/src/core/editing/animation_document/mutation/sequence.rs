use zircon_runtime::animation::sample_animation_bool_channel;
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
    let time_seconds = frame_to_seconds(frame, asset.frames_per_second)?;
    let track = find_track_mut(asset, track_path)?.ok_or_else(|| {
        AnimationDocumentMutationError::InvalidTrackPath {
            message: format!("missing animation track {track_path}"),
        }
    })?;
    // Canonical seconds can be closer than EPSILON; distinct source keys must not alias.
    if track
        .channel
        .keys
        .iter()
        .any(|key| key.time_seconds == time_seconds)
    {
        return Ok(false);
    }
    let value = if track
        .channel
        .keys
        .iter()
        .any(|key| matches!(&key.value, AnimationChannelValueAsset::Bool(_)))
    {
        // Insert the value held at this source time, not the channel's final value.
        // The public query validates the complete Bool/Step input without rewriting it.
        let sampled = sample_animation_bool_channel(&track.channel, time_seconds).ok_or_else(|| {
            AnimationDocumentMutationError::InvalidTrackPath {
                message: format!(
                    "cannot add key to animation track {track_path}: invalid or unsupported Boolean channel"
                ),
            }
        })?;
        AnimationChannelValueAsset::Bool(sampled)
    } else {
        track
            .channel
            .keys
            .last()
            .map(|key| key.value.clone())
            .unwrap_or(AnimationChannelValueAsset::Scalar(0.0))
    };
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
    let time_seconds = frame_to_seconds(frame, asset.frames_per_second)?;
    let track = find_track_mut(asset, track_path)?.ok_or_else(|| {
        AnimationDocumentMutationError::InvalidTrackPath {
            message: format!("missing animation track {track_path}"),
        }
    })?;
    let before = track.channel.keys.len();
    // Preserve every other authored time, including invalid intermediate source values.
    track
        .channel
        .keys
        .retain(|key| key.time_seconds != time_seconds);
    Ok(before != track.channel.keys.len())
}

pub(super) fn create_track(
    asset: &mut AnimationSequenceAsset,
    track_path: &AnimationTrackPath,
) -> Result<bool, AnimationDocumentMutationError> {
    if find_track_mut(asset, track_path)?.is_some() {
        return Ok(false);
    }
    let (entity_path, property_path) = split_track_path(track_path)?;
    // TrackPath has no binding identity. Admit one target before changing the source;
    // choosing the first of several valid bindings would make insertion order-dependent.
    let binding_index = {
        let mut candidates = asset
            .bindings
            .iter()
            .enumerate()
            .filter_map(|(index, binding)| (binding.entity_path == entity_path).then_some(index));
        let first = candidates.next();
        if candidates.next().is_some() {
            return Err(AnimationDocumentMutationError::InvalidTrackPath {
                message: format!("ambiguous animation binding for new track {track_path}"),
            });
        }
        first
    };
    let binding = if let Some(index) = binding_index {
        &mut asset.bindings[index]
    } else {
        asset.bindings.push(AnimationSequenceBindingAsset {
            entity_path,
            target_id: None,
            tracks: Vec::new(),
        });
        asset.bindings.last_mut().expect("binding just pushed")
    };
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
    if find_track_mut(asset, track_path)?.is_none() {
        return Ok(false);
    }
    let (entity_path, property_path) = split_track_path(track_path)?;
    let mut changed = false;
    // An empty binding still owns the stable target for later track creation.
    for binding in &mut asset.bindings {
        if binding.entity_path != entity_path {
            continue;
        }
        let before = binding.tracks.len();
        binding
            .tracks
            .retain(|track| track.property_path != property_path);
        changed |= before != binding.tracks.len();
    }
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
    if from_entity == to_entity {
        let Some(track) = find_track_mut(asset, from_track_path)? else {
            return Ok(false);
        };
        // Changing a property keeps the binding's stable target and the track's position.
        track.property_path = to_property;
        return Ok(true);
    }
    // Resolve one real source track before removal can prune bindings or overwrite a match.
    if find_track_mut(asset, from_track_path)?.is_none() {
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
            return !binding.tracks.is_empty();
        }
        // A different empty binding still owns its stable target and source position.
        true
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
    // The source compiler permits multiple bindings for one entity. TrackPath carries no
    // binding identity, so a command must resolve one complete path before any source write.
    let mut matches = asset
        .bindings
        .iter_mut()
        .filter(|binding| binding.entity_path == entity_path)
        .flat_map(|binding| binding.tracks.iter_mut())
        .filter(|track| track.property_path == property_path);
    let track = matches.next();
    if matches.next().is_some() {
        return Err(AnimationDocumentMutationError::InvalidTrackPath {
            message: format!("ambiguous animation track {track_path}"),
        });
    }
    Ok(track)
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

fn frame_to_seconds(
    frame: u32,
    frames_per_second: f32,
) -> Result<f32, AnimationDocumentMutationError> {
    // Match the source compiler's admitted FPS range. A frame command must not silently
    // choose a replacement rate and mutate a different source time.
    if !frames_per_second.is_finite() || frames_per_second <= f32::EPSILON {
        return Err(AnimationDocumentMutationError::InvalidTrackPath {
            message: "cannot convert frame to source seconds: frames_per_second must be finite and greater than EPSILON".to_string(),
        });
    }
    Ok(frame as f32 / frames_per_second)
}

#[cfg(test)]
#[path = "../tests/sequence_rebind.rs"]
mod tests;

#[cfg(test)]
#[path = "../tests/sequence_bool_keys.rs"]
mod bool_key_tests;

#[cfg(test)]
#[path = "../tests/frame_admit.rs"]
mod frame_admission_tests;

#[cfg(test)]
#[path = "../tests/rebind.rs"]
mod binding_isolation_tests;

#[cfg(test)]
#[path = "../tests/create_admit.rs"]
mod creation_admission_tests;
