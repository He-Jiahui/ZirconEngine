//! 能力不足时的组件回退策略在此声明，具体选择依赖 Runtime 与 Editor 的消费路径。

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiWidgetEditorFallback {
    Hidden,
    Placeholder,
    DisableInteractions,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiWidgetRuntimeFallback {
    RejectNode,
    PlaceholderNode,
    OmitNode,
}

// TODO: [CR-UICOMPONENT-0001] 确认能力不足时 Editor 与 Runtime 回退枚举由哪条路径执行；
// 目录筛选当前仅读取宿主必需能力集合，缺少回退字段的消费证据；下一步核对实例化和预览路径。
/// 分别声明编辑器预览和运行时节点的能力不足回退意图。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiWidgetFallbackPolicy {
    pub editor: UiWidgetEditorFallback,
    pub runtime: UiWidgetRuntimeFallback,
}

impl UiWidgetFallbackPolicy {
    pub const fn new(editor: UiWidgetEditorFallback, runtime: UiWidgetRuntimeFallback) -> Self {
        Self { editor, runtime }
    }
}

impl Default for UiWidgetFallbackPolicy {
    fn default() -> Self {
        Self::new(
            UiWidgetEditorFallback::Placeholder,
            UiWidgetRuntimeFallback::RejectNode,
        )
    }
}
