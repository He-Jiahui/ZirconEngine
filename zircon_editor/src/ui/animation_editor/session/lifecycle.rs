use std::path::Path;

use zircon_runtime::core::framework::animation::{
    AnimationGraphAsset, AnimationSequenceAsset, AnimationStateMachineAsset,
};

use super::support::duration_frames;
use super::{
    AnimationEditorDocument, AnimationEditorSession, AnimationEditorSessionError,
    AnimationSequenceDocument,
};

impl AnimationEditorSession {
    pub fn from_path(path: &Path) -> Result<Self, AnimationEditorSessionError> {
        let bytes =
            std::fs::read(path).map_err(|error| AnimationEditorSessionError(error.to_string()))?;
        let asset_path = path.to_string_lossy().into_owned();
        let lowered = asset_path.to_ascii_lowercase();
        if lowered.ends_with(".sequence.zranim") {
            let asset = AnimationSequenceAsset::from_bytes(&bytes)
                .map_err(|error| AnimationEditorSessionError(error.to_string()))?;
            let timeline_end_frame =
                duration_frames(asset.duration_seconds, asset.frames_per_second);
            return Ok(Self {
                asset_path,
                document: AnimationEditorDocument::Sequence(AnimationSequenceDocument {
                    asset,
                    current_frame: 0,
                    timeline_start_frame: 0,
                    timeline_end_frame,
                    selected_span: None,
                    playing: false,
                    looping: false,
                    speed: 1.0,
                }),
                dirty: false,
            });
        }
        if lowered.ends_with(".graph.zranim") {
            let asset = AnimationGraphAsset::from_bytes(&bytes)
                .map_err(|error| AnimationEditorSessionError(error.to_string()))?;
            return Ok(Self {
                asset_path,
                document: AnimationEditorDocument::Graph(asset),
                dirty: false,
            });
        }
        if lowered.ends_with(".state_machine.zranim") {
            let asset = AnimationStateMachineAsset::from_bytes(&bytes)
                .map_err(|error| AnimationEditorSessionError(error.to_string()))?;
            return Ok(Self {
                asset_path,
                document: AnimationEditorDocument::StateMachine(asset),
                dirty: false,
            });
        }
        Err(AnimationEditorSessionError(format!(
            "unsupported animation editor asset {}",
            path.display()
        )))
    }

    pub fn asset_path(&self) -> &str {
        &self.asset_path
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn save(&mut self) -> Result<(), AnimationEditorSessionError> {
        let bytes = self.document_bytes()?;
        std::fs::write(&self.asset_path, bytes)
            .map_err(|error| AnimationEditorSessionError(error.to_string()))?;
        self.dirty = false;
        Ok(())
    }
}
