use zircon_runtime::core::framework::animation::AnimationTrackPath;
use zircon_runtime::core::framework::animation::{
    AnimationChannelAsset, AnimationChannelKeyAsset, AnimationChannelValueAsset,
    AnimationInterpolationAsset, AnimationSequenceBindingAsset, AnimationSequenceTrackAsset,
};

use super::AnimationEditorSession;
use super::support::{clamp_timeline_span, frame_to_seconds};

impl AnimationEditorSession {
    pub fn add_key(&mut self, track_path: &AnimationTrackPath, frame: u32) -> Result<bool, String> {
        let time_seconds = frame_to_seconds(frame, self.sequence_frames_per_second());
        let track = self
            .sequence_track_mut(track_path)?
            .ok_or_else(|| format!("missing animation track {track_path}"))?;
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
        self.dirty = true;
        Ok(true)
    }

    pub fn remove_key(
        &mut self,
        track_path: &AnimationTrackPath,
        frame: u32,
    ) -> Result<bool, String> {
        let time_seconds = frame_to_seconds(frame, self.sequence_frames_per_second());
        let track = self
            .sequence_track_mut(track_path)?
            .ok_or_else(|| format!("missing animation track {track_path}"))?;
        let before = track.channel.keys.len();
        track
            .channel
            .keys
            .retain(|key| (key.time_seconds - time_seconds).abs() > f32::EPSILON);
        let changed = before != track.channel.keys.len();
        self.dirty |= changed;
        Ok(changed)
    }

    pub fn create_track(&mut self, track_path: &AnimationTrackPath) -> Result<bool, String> {
        let (entity_path, property_path) = track_path.split().map_err(|error| error.to_string())?;
        let document = self.sequence_document_mut()?;
        let binding_index = document
            .asset
            .bindings
            .iter()
            .position(|binding| binding.entity_path == entity_path);
        let binding = if let Some(binding_index) = binding_index {
            &mut document.asset.bindings[binding_index]
        } else {
            document.asset.bindings.push(AnimationSequenceBindingAsset {
                entity_path,
                target_id: None,
                tracks: Vec::new(),
            });
            document
                .asset
                .bindings
                .last_mut()
                .expect("binding just pushed")
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
        self.dirty = true;
        Ok(true)
    }

    pub fn remove_track(&mut self, track_path: &AnimationTrackPath) -> Result<bool, String> {
        let (entity_path, property_path) = track_path.split().map_err(|error| error.to_string())?;
        let document = self.sequence_document_mut()?;
        let mut changed = false;
        document.asset.bindings.retain_mut(|binding| {
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
        if changed
            && matches!(
                document.selected_span.as_ref(),
                Some((selected_track_path, _, _)) if selected_track_path == track_path
            )
        {
            document.selected_span = None;
        }
        self.dirty |= changed;
        Ok(changed)
    }

    pub fn rebind_track(
        &mut self,
        from_track_path: &AnimationTrackPath,
        to_track_path: &AnimationTrackPath,
    ) -> Result<bool, String> {
        let (from_entity, from_property) =
            from_track_path.split().map_err(|error| error.to_string())?;
        let (to_entity, to_property) = to_track_path.split().map_err(|error| error.to_string())?;
        if from_entity == to_entity && from_property == to_property {
            return Ok(false);
        }
        let document = self.sequence_document_mut()?;
        if document.asset.bindings.iter().any(|binding| {
            binding.entity_path == to_entity
                && binding
                    .tracks
                    .iter()
                    .any(|track| track.property_path == to_property)
        }) {
            return Ok(false);
        }
        let mut moved_track = None;
        document.asset.bindings.retain_mut(|binding| {
            if binding.entity_path != from_entity {
                return true;
            }
            if let Some(track_index) = binding
                .tracks
                .iter()
                .position(|track| track.property_path == from_property)
            {
                moved_track = Some(binding.tracks.remove(track_index));
            }
            !binding.tracks.is_empty()
        });
        let Some(mut moved_track) = moved_track else {
            return Ok(false);
        };
        moved_track.property_path = to_property;
        let binding_index = document
            .asset
            .bindings
            .iter()
            .position(|binding| binding.entity_path == to_entity);
        let binding = if let Some(binding_index) = binding_index {
            &mut document.asset.bindings[binding_index]
        } else {
            document.asset.bindings.push(AnimationSequenceBindingAsset {
                entity_path: to_entity,
                target_id: None,
                tracks: Vec::new(),
            });
            document
                .asset
                .bindings
                .last_mut()
                .expect("binding just pushed")
        };
        binding.tracks.push(moved_track);
        if let Some((selected_track_path, start_frame, end_frame)) = document.selected_span.clone()
        {
            if selected_track_path == *from_track_path {
                document.selected_span = Some((to_track_path.clone(), start_frame, end_frame));
            }
        }
        self.dirty = true;
        Ok(true)
    }

    pub fn scrub_timeline(&mut self, frame: u32) -> Result<bool, String> {
        let document = self.sequence_document_mut()?;
        let next = frame.clamp(document.timeline_start_frame, document.timeline_end_frame);
        let changed = document.current_frame != next;
        document.current_frame = next;
        Ok(changed)
    }

    pub fn set_timeline_range(&mut self, start_frame: u32, end_frame: u32) -> Result<bool, String> {
        let document = self.sequence_document_mut()?;
        let (next_start, next_end) = if start_frame <= end_frame {
            (start_frame, end_frame)
        } else {
            (end_frame, start_frame)
        };
        let changed =
            document.timeline_start_frame != next_start || document.timeline_end_frame != next_end;
        document.timeline_start_frame = next_start;
        document.timeline_end_frame = next_end;
        document.current_frame = document.current_frame.clamp(next_start, next_end);
        if let Some((track_path, selected_start, selected_end)) = document.selected_span.clone() {
            let (selected_start, selected_end) =
                clamp_timeline_span(selected_start, selected_end, next_start, next_end);
            document.selected_span = Some((track_path, selected_start, selected_end));
        }
        Ok(changed)
    }

    pub fn select_timeline_span(
        &mut self,
        track_path: &AnimationTrackPath,
        start_frame: u32,
        end_frame: u32,
    ) -> Result<bool, String> {
        let (entity_path, property_path) = track_path.split().map_err(|error| error.to_string())?;
        let document = self.sequence_document_mut()?;
        let has_track = document.asset.bindings.iter().any(|binding| {
            binding.entity_path == entity_path
                && binding
                    .tracks
                    .iter()
                    .any(|track| track.property_path == property_path)
        });
        if !has_track {
            return Ok(false);
        }
        let (start_frame, end_frame) = if start_frame <= end_frame {
            (start_frame, end_frame)
        } else {
            (end_frame, start_frame)
        };
        let (start_frame, end_frame) = clamp_timeline_span(
            start_frame,
            end_frame,
            document.timeline_start_frame,
            document.timeline_end_frame,
        );
        let next = Some((track_path.clone(), start_frame, end_frame));
        let changed = document.selected_span != next;
        document.selected_span = next;
        Ok(changed)
    }

    pub fn set_playback(
        &mut self,
        playing: bool,
        looping: bool,
        speed: f32,
    ) -> Result<bool, String> {
        if !speed.is_finite() {
            return Ok(false);
        }
        let document = self.sequence_document_mut()?;
        let changed = document.playing != playing
            || document.looping != looping
            || (document.speed - speed).abs() > f32::EPSILON;
        document.playing = playing;
        document.looping = looping;
        document.speed = speed;
        Ok(changed)
    }
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
