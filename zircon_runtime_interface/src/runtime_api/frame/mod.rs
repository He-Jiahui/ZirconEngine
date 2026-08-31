mod frame_demand;
mod frame_shape;
mod highlight_set;
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
mod frame_shape_tests;
