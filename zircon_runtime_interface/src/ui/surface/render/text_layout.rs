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

/// 源文本或视觉文本中的半开 UTF-8 字节范围；调用方应使用字符边界再进行切片。
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
    identity: Arc<dyn RuntimeArtifactIdentity>,
}

trait RuntimeArtifactIdentity: Any + Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn equals(&self, other: &dyn RuntimeArtifactIdentity) -> bool;
}

struct TypedRuntimeArtifactIdentity<T>(T);

impl<T> RuntimeArtifactIdentity for TypedRuntimeArtifactIdentity<T>
where
    T: Any + Send + Sync + PartialEq,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn equals(&self, other: &dyn RuntimeArtifactIdentity) -> bool {
        other
            .as_any()
            .downcast_ref::<Self>()
            .is_some_and(|other| self.0 == other.0)
    }
}

impl UiRichTextArtifactHandle {
    /// Erases a process-local runtime artifact while retaining the owner's semantic identity.
    ///
    /// The identity must change whenever the artifact can produce different layout, rendering,
    /// or rebuild behavior. Allocation identity and the erased payload's `TypeId` are not a
    /// sufficient substitute for that contract.
    pub fn from_runtime_artifact_with_identity<T, I>(artifact: Arc<T>, identity: I) -> Self
    where
        T: Any + Send + Sync,
        I: Any + Send + Sync + PartialEq,
    {
        Self {
            artifact,
            identity: Arc::new(TypedRuntimeArtifactIdentity(identity)),
        }
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
        self.artifact.as_ref().type_id() == other.artifact.as_ref().type_id()
            && self.identity.equals(other.identity.as_ref())
    }
}

#[cfg(test)]
#[path = "tests/text_layout.rs"]
mod tests;

/// 文本布局所有者发布给绘制、命中和 IME 几何计算的共同结果。
/// 序列化只传可重建的布局数据；进程内 rich_text_artifact 由 Runtime 持有并按身份判断缓存等价。
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
    /// Absolute natural content geometry. The main-axis extent agrees with
    /// `measured_width`; alignment changes its origin but never its extent.
    pub frame: UiFrame,
    /// Absolute paragraph or rich-cell slot used to place and select this
    /// physical line. Glyph, caret, selection, and IME geometry use `frame`.
    pub placement_frame: UiFrame,
    pub source_range: UiTextRange,
    pub visual_range: UiTextRange,
    pub measured_width: f32,
    /// One visual advance per grapheme cluster in `text`, populated by the
    /// runtime text layout owner before shaped/render DTO projection.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub glyph_advances: Vec<f32>,
    /// Cross-axis offset relative to `frame`: y for horizontal text and x for `VerticalRl`.
    pub baseline: f32,
    pub direction: UiTextDirection,
    pub runs: Vec<UiResolvedTextRun>,
    pub ellipsized: bool,
}

impl UiResolvedTextLine {
    /// Content-level hit candidacy is intentionally narrower than the full
    /// placement slot, so aligned empty space cannot activate a rich run.
    pub fn hit_frame(&self) -> UiFrame {
        self.frame
    }

    pub fn translate(&mut self, delta_x: f32, delta_y: f32) {
        self.frame.x += delta_x;
        self.frame.y += delta_y;
        self.placement_frame.x += delta_x;
        self.placement_frame.y += delta_y;
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiResolvedTextRun {
    pub kind: UiTextRunKind,
    pub text: String,
    pub source_range: UiTextRange,
    pub visual_range: UiTextRange,
    pub direction: UiTextDirection,
}
