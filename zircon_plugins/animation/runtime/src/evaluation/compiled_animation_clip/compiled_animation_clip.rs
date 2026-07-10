use std::sync::Arc;

use super::super::{CompiledClipTrack, SkeletonTargetTable};

/// Clip channels with all string target lookups resolved to dense slots.
#[derive(Clone, Debug)]
pub struct CompiledAnimationClip {
    pub(super) target_table: Arc<SkeletonTargetTable>,
    pub(super) tracks: Vec<CompiledClipTrack>,
}

impl CompiledAnimationClip {
    pub fn target_table(&self) -> &SkeletonTargetTable {
        &self.target_table
    }

    pub fn tracks(&self) -> &[CompiledClipTrack] {
        &self.tracks
    }

    pub fn target_index_for_track(&self, track_index: usize) -> Option<usize> {
        let track = self.tracks.get(track_index)?;
        self.target_table.bone_index_for_slot(track.target)
    }
}
