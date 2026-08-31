use zircon_runtime::core::framework::animation::AnimationTrackPath;

use super::AnimationEditorSession;
use super::support::{clamp_timeline_span, frame_to_seconds};

impl AnimationEditorSession {
    /// Source replay can invalidate a UI-only selection. Keep the transient cursor detached from
    /// document history while refusing to project a span for a track no longer in the source.
    pub(crate) fn reconcile_source_change(&mut self) {
        let selected_track_path = self
            .sequence
            .as_ref()
            .and_then(|sequence| sequence.selected_span.as_ref())
            .map(|(track_path, _, _)| track_path.clone());
        let Some(selected_track_path) = selected_track_path else {
            return;
        };
        let is_present = self
            .document()
            .read()
            .asset()
            .as_sequence()
            .is_some_and(|asset| asset.track_paths().contains(&selected_track_path));
        if !is_present {
            self.clear_selected_timeline_track_if(&selected_track_path);
        }
    }

    pub(crate) fn clear_selected_timeline_track_if(&mut self, track_path: &AnimationTrackPath) {
        let Ok(sequence) = self.sequence_state_mut() else {
            return;
        };
        if matches!(
            sequence.selected_span.as_ref(),
            Some((selected_track_path, _, _)) if selected_track_path == track_path
        ) {
            sequence.selected_span = None;
        }
    }

    pub(crate) fn rebind_selected_timeline_track_if(
        &mut self,
        from_track_path: &AnimationTrackPath,
        to_track_path: &AnimationTrackPath,
    ) {
        let Ok(sequence) = self.sequence_state_mut() else {
            return;
        };
        if let Some((selected_track_path, start_frame, end_frame)) = sequence.selected_span.clone()
            && selected_track_path == *from_track_path
        {
            sequence.selected_span = Some((to_track_path.clone(), start_frame, end_frame));
        }
    }

    pub fn scrub_timeline(&mut self, frame: u32) -> Result<bool, String> {
        let sequence = self.sequence_state_mut()?;
        let next = frame.clamp(sequence.timeline_start_frame, sequence.timeline_end_frame);
        let changed = sequence.current_frame != next;
        sequence.current_frame = next;
        Ok(changed)
    }

    pub fn set_timeline_range(&mut self, start_frame: u32, end_frame: u32) -> Result<bool, String> {
        let sequence = self.sequence_state_mut()?;
        let (next_start, next_end) = if start_frame <= end_frame {
            (start_frame, end_frame)
        } else {
            (end_frame, start_frame)
        };
        let changed =
            sequence.timeline_start_frame != next_start || sequence.timeline_end_frame != next_end;
        sequence.timeline_start_frame = next_start;
        sequence.timeline_end_frame = next_end;
        sequence.current_frame = sequence.current_frame.clamp(next_start, next_end);
        if let Some((track_path, selected_start, selected_end)) = sequence.selected_span.clone() {
            let (selected_start, selected_end) =
                clamp_timeline_span(selected_start, selected_end, next_start, next_end);
            sequence.selected_span = Some((track_path, selected_start, selected_end));
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
        let has_track = self
            .document()
            .read()
            .asset()
            .as_sequence()
            .is_some_and(|asset| {
                asset.bindings.iter().any(|binding| {
                    binding.entity_path == entity_path
                        && binding
                            .tracks
                            .iter()
                            .any(|track| track.property_path == property_path)
                })
            });
        if !has_track {
            return Ok(false);
        }
        let sequence = self.sequence_state_mut()?;
        let (start_frame, end_frame) = if start_frame <= end_frame {
            (start_frame, end_frame)
        } else {
            (end_frame, start_frame)
        };
        let (start_frame, end_frame) = clamp_timeline_span(
            start_frame,
            end_frame,
            sequence.timeline_start_frame,
            sequence.timeline_end_frame,
        );
        let next = Some((track_path.clone(), start_frame, end_frame));
        let changed = sequence.selected_span != next;
        sequence.selected_span = next;
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
        let sequence = self.sequence_state_mut()?;
        let changed = sequence.playing != playing
            || sequence.looping != looping
            || (sequence.speed - speed).abs() > f32::EPSILON;
        sequence.playing = playing;
        sequence.looping = looping;
        sequence.speed = speed;
        Ok(changed)
    }
}
