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
