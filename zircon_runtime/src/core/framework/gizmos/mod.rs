//! 通用 gizmo 命令边界：调用方收集临时命令或共享保留式资产，再提取为 RenderOverlayExtract 的线段。
//! 命令层不持有视口或 GPU 状态；实际绘制与拾取由下游渲染覆盖层负责。

mod buffer;
mod command;
mod config;
mod extract;
mod retained;

pub use buffer::GizmoBuffer;
pub use command::{GizmoAxis, GizmoCommand};
pub use config::{
    GizmoColorPolicy, GizmoConfig, GizmoConfigGroupId, GizmoLineConfig, GizmoRenderLayer,
    GizmoScreenScalePolicy,
};
pub use extract::{append_gizmo_overlay, extract_gizmo_overlay, GizmoOverlayExtractRequest};
pub use retained::{GizmoAsset, RetainedGizmo};
