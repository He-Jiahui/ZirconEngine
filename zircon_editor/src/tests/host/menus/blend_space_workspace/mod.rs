// 汇集混合空间工作台的结构、交互、响应式及视觉测试模块。
use super::*;

#[path = "composite_contracts.rs"]
mod composite_contracts;
#[path = "details_layout.rs"]
mod details_layout;
#[path = "preview_viewport.rs"]
mod preview_viewport;
#[path = "property_editor_rows.rs"]
mod property_editor_rows;
#[path = "responsive_geometry.rs"]
mod responsive_geometry;
#[path = "responsive_layout.rs"]
mod responsive_layout;
#[path = "support.rs"]
pub(super) mod support;
#[path = "timeline_actions.rs"]
mod timeline_actions;
#[path = "toolbar_surface.rs"]
mod toolbar_surface;
#[path = "transport_actions.rs"]
mod transport_actions;
#[path = "validation_actions.rs"]
mod validation_actions;
#[path = "visual_capture.rs"]
pub(super) mod visual_capture;
#[path = "workspace_structure.rs"]
mod workspace_structure;

use support::*;
