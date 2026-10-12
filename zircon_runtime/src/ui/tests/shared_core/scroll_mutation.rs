//! 这组测试从滚动状态变更进入局部布局失效和虚拟窗口发布，避免只改偏移却留下旧命中几何。

use super::*;

#[path = "scroll_mutation/pointer_routes.rs"]
mod pointer_routes;
#[path = "scroll_mutation/property_mutation.rs"]
mod property_mutation;
#[path = "scroll_mutation/virtual_scroll.rs"]
mod virtual_scroll;
