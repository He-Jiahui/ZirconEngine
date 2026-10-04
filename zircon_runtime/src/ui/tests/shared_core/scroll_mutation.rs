//! 这组测试从滚动状态变更进入局部布局失效和虚拟窗口发布，避免只改偏移却留下旧命中几何。

use super::*;

mod pointer_routes;
mod property_mutation;
mod virtual_scroll;
