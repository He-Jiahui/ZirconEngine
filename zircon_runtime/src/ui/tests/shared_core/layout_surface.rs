//! 这组测试覆盖测量、容器几何与渲染抽取的共享入口，强调布局缓存经安排后才能成为表面输出。

use super::*;

#[path = "layout_surface/container_overlays.rs"]
mod container_overlays;
#[path = "layout_surface/layout_measurement.rs"]
mod layout_measurement;
#[path = "layout_surface/render_extract.rs"]
mod render_extract;
