//! 帧输出、调度需求、编辑器覆盖层与视口拾取共用的运行时接口类型。
//!
//! 本模块只定义跨 Runtime、App、Editor 的固定布局值、验证方法和无状态形状检查；会话状态、
//! 渲染后端资源及异步拾取票据的实际生命周期由各自所有者管理。`frame_demand` 描述 tick
//! 后的下一帧调度，`frame_shape` 校验 V2 RGBA 长度，`highlight_set` 传递临时借用的实体集，
//! `viewport_pick` 定义绑定已展示帧的请求与完成回执。
//!
//! 类型通过 `runtime_api` 与 crate 根重导出，并由 Runtime 动态 ABI 函数读写。对应的局部
//! 边界测试位于各类型的 `tests` 子模块；跨 crate 调用路径见 Runtime FFI、App 会话和
//! Editor Gateway。

#[path = "frame_demand.rs"]
mod frame_demand;
#[path = "frame_shape.rs"]
mod frame_shape;
#[path = "highlight_set.rs"]
mod highlight_set;
#[path = "viewport_pick.rs"]
mod viewport_pick;

pub use frame_demand::{
    ZrRuntimeFrameDemandV1, ZR_RUNTIME_FRAME_DEMAND_AFTER_V1, ZR_RUNTIME_FRAME_DEMAND_IDLE_V1,
    ZR_RUNTIME_FRAME_DEMAND_IMMEDIATE_V1,
};
pub use frame_shape::{validate_runtime_frame_rgba_shape, ZrRuntimeFrameRgbaShapeError};
pub use highlight_set::{
    ZrRuntimeEntityIdSliceV1, ZrRuntimeHighlightRenderAttributesV1, ZrRuntimeHighlightSetV1,
};
pub use viewport_pick::{
    ZrRuntimeViewportPickDispositionV1, ZrRuntimeViewportPickPurposeV1,
    ZrRuntimeViewportPickRequestV1, ZrRuntimeViewportPickResultV1, ZrRuntimeViewportPickTicket,
    ZrRuntimeViewportPixelV1, ZR_RUNTIME_VIEWPORT_PICK_POLICY_INCLUDE_BACKFACES_V1,
    ZR_RUNTIME_VIEWPORT_PICK_POLICY_INCLUDE_TRANSLUCENT_V1,
};

#[cfg(test)]
#[path = "tests/frame_shape_tests.rs"]
mod frame_shape_tests;
