use std::any::Any;

use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::Arc;

use super::{
    UiEditableTextState, UiTextAlign, UiTextDirection, UiTextOverflow, UiTextRunKind, UiTextWrap,
    UiTextWritingMode,
};
use crate::ui::layout::UiFrame;
use crate::ui::style::UiRgbaColor;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiTextRange {
    pub start: usize,
    pub end: usize,
}

/// Opaque runtime-owned text artifact.
///
/// The type-erased allocation follows rich-text and shaped-glyph sidecars
/// across extract, rendering, resources, and input without exposing runtime
/// implementation types through the public UI contract or serializing
/// process-local state.
#[derive(Clone)]
pub struct UiRichTextArtifactHandle {
    artifact: Arc<dyn Any + Send + Sync>,
}

impl UiRichTextArtifactHandle {
    pub fn from_runtime_artifact<T>(artifact: Arc<T>) -> Self
    where
        T: Any + Send + Sync,
    {
        Self { artifact }
    }

    pub fn downcast_runtime_artifact<T>(&self) -> Option<Arc<T>>
    where
        T: Any + Send + Sync,
    {
        Arc::downcast(Arc::clone(&self.artifact)).ok()
    }
}

impl fmt::Debug for UiRichTextArtifactHandle {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("UiRichTextArtifactHandle")
            .finish_non_exhaustive()
    }
}

impl PartialEq for UiRichTextArtifactHandle {
    fn eq(&self, other: &Self) -> bool {
        // Artifacts are process-local caches, so allocation identity must not make two
        // otherwise identical resolved layouts compare differently.
        self.artifact.as_ref().type_id() == other.artifact.as_ref().type_id()
    }
}

impl Eq for UiRichTextArtifactHandle {}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::UiRichTextArtifactHandle;

    #[test]
    fn artifact_equality_ignores_process_local_allocation_identity() {
        let first = UiRichTextArtifactHandle::from_runtime_artifact(Arc::new(1_u32));
        let second = UiRichTextArtifactHandle::from_runtime_artifact(Arc::new(2_u32));
        let different_kind = UiRichTextArtifactHandle::from_runtime_artifact(Arc::new(1_u64));

        assert_eq!(first, second);
        assert_ne!(first, different_kind);
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiResolvedTextLayout {
    pub text_align: UiTextAlign,
    pub wrap: UiTextWrap,
    pub direction: UiTextDirection,
    #[serde(default)]
    pub writing_mode: UiTextWritingMode,
    pub overflow: UiTextOverflow,
    pub font_size: f32,
    pub line_height: f32,
    pub measured_width: f32,
    pub measured_height: f32,
    pub source_range: UiTextRange,
    pub lines: Vec<UiResolvedTextLine>,
    /// Resolved non-glyph boxes such as rich-table cell backgrounds and borders.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub boxes: Vec<UiResolvedTextBox>,
    pub overflow_clipped: bool,
    pub editable: Option<UiEditableTextState>,
    /// Process-local compiled rich-text or shaped-glyph lifetime. Never
    /// persisted or sent over runtime/editor serialization boundaries.
    #[serde(skip)]
    pub rich_text_artifact: Option<UiRichTextArtifactHandle>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiResolvedTextBox {
    pub range: UiTextRange,
    pub frame: UiFrame,
    pub background_color: Option<UiRgbaColor>,
    pub border_color: Option<UiRgbaColor>,
    pub border_width: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiResolvedTextLine {
    pub text: String,
    pub frame: UiFrame,
    pub source_range: UiTextRange,
    pub visual_range: UiTextRange,
    pub measured_width: f32,
    /// One visual advance per grapheme cluster in `text`, populated by the
    /// runtime text layout owner before shaped/render DTO projection.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub glyph_advances: Vec<f32>,
    pub baseline: f32,
    pub direction: UiTextDirection,
    pub runs: Vec<UiResolvedTextRun>,
    pub ellipsized: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiResolvedTextRun {
    pub kind: UiTextRunKind,
    pub text: String,
    pub source_range: UiTextRange,
    pub visual_range: UiTextRange,
    pub direction: UiTextDirection,
}
