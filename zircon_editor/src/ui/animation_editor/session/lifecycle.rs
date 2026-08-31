#[cfg(test)]
use std::path::Path;

#[cfg(test)]
use zircon_runtime::asset::AssetUri;

#[cfg(test)]
use crate::core::editing::animation_document::{
    AnimationAuthoringAsset, AnimationAuthoringDocument, AnimationAuthoringDocumentKind,
    AnimationAuthoringDocumentReadHandle,
};
#[cfg(test)]
use crate::core::editor_message::DocumentId;

use super::{AnimationEditorSession, AnimationEditorSessionError};

impl AnimationEditorSession {
    #[cfg(test)]
    pub(super) fn from_path(path: &Path) -> Result<Self, AnimationEditorSessionError> {
        let asset_path = path.to_string_lossy();
        let lowered = asset_path.to_ascii_lowercase();
        let document_kind = if lowered.ends_with(".sequence.zranim") {
            AnimationEditorDocumentKind::Sequence
        } else if lowered.ends_with(".graph.zranim") {
            AnimationEditorDocumentKind::Graph
        } else if lowered.ends_with(".state_machine.zranim") {
            AnimationEditorDocumentKind::StateMachine
        } else {
            return Err(AnimationEditorSessionError::new(format!(
                "unsupported animation editor asset {}",
                path.display()
            )));
        };
        let bytes = std::fs::read(path)
            .map_err(|error| AnimationEditorSessionError::new(error.to_string()))?;
        let asset = AnimationAuthoringAsset::from_bytes(document_kind, &bytes)
            .map_err(AnimationEditorSessionError::from_animation_asset_error)?;
        let document = AnimationAuthoringDocument::new(
            DocumentId::new(1),
            AssetUri::parse("res://tests/animation.zranim")
                .expect("test animation locator must be valid"),
            asset,
        );
        Ok(Self::new(
            path.to_string_lossy().into_owned(),
            AnimationAuthoringDocumentReadHandle::detached_for_test(document),
        ))
    }

    pub fn asset_path(&self) -> &str {
        &self.asset_path
    }
}
