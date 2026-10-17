use serde::{Deserialize, Serialize};

/// 阶段报告携带的失效原因标签，供诊断展示及投影关联使用。
///
/// 标签本身不标记节点为脏，也不触发重建；多个原因由报告中的向量承载。
/// Serde 使用蛇形名称传递标签；原因是否适用于某阶段由生产者选择。
///
/// # 参考
/// [`super::UiPipelineStageReport`]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiPipelineDirtyReason {
    Input,
    Focus,
    WidgetBehavior,
    Text,
    Style,
    Layout,
    LayoutMetrics,
    Picking,
    HitGrid,
    A11y,
    Render,
    Template,
    Window,
    HostRequest,
    Diagnostics,
}
