//! pane内容选择模板交互或混合原生插槽的声明；事件路由仍由宿主绑定。
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaneInteractionMode {
    TemplateOnly,
    HybridNativeSlot,
}
