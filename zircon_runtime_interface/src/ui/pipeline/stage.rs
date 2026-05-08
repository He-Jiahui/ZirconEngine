use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiPipelineStage {
    InputCollect,
    FocusInteraction,
    ContentMeasure,
    Layout,
    PostLayoutStack,
    HitGrid,
    RenderExtract,
    BatchPrepare,
    PaintSubmit,
    Diagnostics,
}

impl UiPipelineStage {
    pub const ORDER: [Self; 10] = [
        Self::InputCollect,
        Self::FocusInteraction,
        Self::ContentMeasure,
        Self::Layout,
        Self::PostLayoutStack,
        Self::HitGrid,
        Self::RenderExtract,
        Self::BatchPrepare,
        Self::PaintSubmit,
        Self::Diagnostics,
    ];

    pub const fn ordered() -> &'static [Self; 10] {
        &Self::ORDER
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InputCollect => "input_collect",
            Self::FocusInteraction => "focus_interaction",
            Self::ContentMeasure => "content_measure",
            Self::Layout => "layout",
            Self::PostLayoutStack => "post_layout_stack",
            Self::HitGrid => "hit_grid",
            Self::RenderExtract => "render_extract",
            Self::BatchPrepare => "batch_prepare",
            Self::PaintSubmit => "paint_submit",
            Self::Diagnostics => "diagnostics",
        }
    }
}
