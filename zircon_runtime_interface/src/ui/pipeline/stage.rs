use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiPipelineStage {
    InputCollect,
    Focus,
    WidgetBehavior,
    TextMeasure,
    Layout,
    PostLayout,
    Picking,
    A11yExtract,
    RenderExtract,
    BatchPrepare,
    // Archived diagnostic V1 names stay deserializable for stored reports only.
    // They are intentionally excluded from the Bevy-aligned runtime schedule order.
    FocusInteraction,
    ContentMeasure,
    PostLayoutStack,
    HitGrid,
    PaintSubmit,
    Diagnostics,
}

impl UiPipelineStage {
    pub const ARCHIVED_DIAGNOSTIC_FORMAT_VERSION: u16 = 1;

    pub const ORDER: [Self; 10] = [
        Self::InputCollect,
        Self::Focus,
        Self::WidgetBehavior,
        Self::TextMeasure,
        Self::Layout,
        Self::PostLayout,
        Self::Picking,
        Self::A11yExtract,
        Self::RenderExtract,
        Self::BatchPrepare,
    ];

    pub const ARCHIVED_DIAGNOSTIC_STAGES: [Self; 6] = [
        Self::FocusInteraction,
        Self::ContentMeasure,
        Self::PostLayoutStack,
        Self::HitGrid,
        Self::PaintSubmit,
        Self::Diagnostics,
    ];

    pub const fn ordered() -> &'static [Self; 10] {
        &Self::ORDER
    }

    pub const fn archived_diagnostic_stages() -> &'static [Self; 6] {
        &Self::ARCHIVED_DIAGNOSTIC_STAGES
    }

    pub const fn is_runtime_schedule_stage(self) -> bool {
        match self {
            Self::InputCollect
            | Self::Focus
            | Self::WidgetBehavior
            | Self::TextMeasure
            | Self::Layout
            | Self::PostLayout
            | Self::Picking
            | Self::A11yExtract
            | Self::RenderExtract
            | Self::BatchPrepare => true,
            Self::FocusInteraction
            | Self::ContentMeasure
            | Self::PostLayoutStack
            | Self::HitGrid
            | Self::PaintSubmit
            | Self::Diagnostics => false,
        }
    }

    pub const fn is_archived_diagnostic_stage(self) -> bool {
        match self {
            Self::FocusInteraction
            | Self::ContentMeasure
            | Self::PostLayoutStack
            | Self::HitGrid
            | Self::PaintSubmit
            | Self::Diagnostics => true,
            Self::InputCollect
            | Self::Focus
            | Self::WidgetBehavior
            | Self::TextMeasure
            | Self::Layout
            | Self::PostLayout
            | Self::Picking
            | Self::A11yExtract
            | Self::RenderExtract
            | Self::BatchPrepare => false,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InputCollect => "input_collect",
            Self::Focus => "focus",
            Self::WidgetBehavior => "widget_behavior",
            Self::TextMeasure => "text_measure",
            Self::Layout => "layout",
            Self::PostLayout => "post_layout",
            Self::Picking => "picking",
            Self::A11yExtract => "a11y_extract",
            Self::RenderExtract => "render_extract",
            Self::BatchPrepare => "batch_prepare",
            Self::FocusInteraction => "focus_interaction",
            Self::ContentMeasure => "content_measure",
            Self::PostLayoutStack => "post_layout_stack",
            Self::HitGrid => "hit_grid",
            Self::PaintSubmit => "paint_submit",
            Self::Diagnostics => "diagnostics",
        }
    }
}
