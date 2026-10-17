use serde::{Deserialize, Serialize};

/// 报告与调度投影使用的阶段标识，同时保留旧版诊断名称。
///
/// 枚举值不执行阶段；当前顺序由 [`Self::ordered`] 给出，归档组独立列出。
/// Serde 保留所有变体的蛇形名称，读取旧诊断名称不表示恢复旧调度顺序。
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
    /// 旧诊断命名组的版本标识；常量本身不执行版本迁移或输入校验。
    pub const ARCHIVED_DIAGNOSTIC_FORMAT_VERSION: u16 = 1;

    /// 报告完整性检查与调度投影使用的固定顺序，归档名称不在此表中。
    /// 存在某阶段的报告行不等于该阶段实际执行过。
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

    /// 借用当前阶段顺序的静态表，不分配或创建阶段实例。
    pub const fn ordered() -> &'static [Self; 10] {
        &Self::ORDER
    }

    /// 借用仅为历史诊断兼容保留的静态名称组，不属于当前必需阶段。
    pub const fn archived_diagnostic_stages() -> &'static [Self; 6] {
        &Self::ARCHIVED_DIAGNOSTIC_STAGES
    }

    /// 为当前阶段提供与规范顺序对应的小下标，供帧报告查询和存在表使用。
    /// 归档名称返回 `None`，由查询方选择回退路径。
    pub(crate) const fn runtime_index(self) -> Option<usize> {
        match self {
            Self::InputCollect => Some(0),
            Self::Focus => Some(1),
            Self::WidgetBehavior => Some(2),
            Self::TextMeasure => Some(3),
            Self::Layout => Some(4),
            Self::PostLayout => Some(5),
            Self::Picking => Some(6),
            Self::A11yExtract => Some(7),
            Self::RenderExtract => Some(8),
            Self::BatchPrepare => Some(9),
            Self::FocusInteraction
            | Self::ContentMeasure
            | Self::PostLayoutStack
            | Self::HitGrid
            | Self::PaintSubmit
            | Self::Diagnostics => None,
        }
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

    /// 返回不分配内存的静态诊断名称，与本枚举的蛇形 Serde 标签一致。
    /// 调用方可将该借用用于展示或记录，不需要绑定某个报告对象的生命周期。
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
