mod api_shape;
mod api_table;
mod host_api_shape;

pub use api_shape::{validate_runtime_api_v8_shape, ZrRuntimeApiV8ShapeError};
pub use api_table::{
    ZrHostApiV1, ZrRuntimeApiV8, ZrRuntimeBindViewportSurfaceFnV1, ZrRuntimeCancelViewportPickFnV1,
    ZrRuntimeCaptureAccessibilityTreeFnV2, ZrRuntimeCaptureFrameFnV2, ZrRuntimeCreateSessionFnV3,
    ZrRuntimeDestroySessionFnV1, ZrRuntimeDrainHostRequestsFnV2,
    ZrRuntimeDrainWorldInvalidationsFnV2, ZrRuntimeGetApiFnV8, ZrRuntimeHandleEventFnV1,
    ZrRuntimeHostFetchFnV1, ZrRuntimePollViewportPickFnV1, ZrRuntimePresentViewportFnV1,
    ZrRuntimeProfileControlFnV2, ZrRuntimeQueryWorldFnV2, ZrRuntimeReleaseAllocationFnV2,
    ZrRuntimeRequestViewportPickFnV1, ZrRuntimeSubmitHighlightSetFnV1, ZrRuntimeTickFrameFnV2,
    ZrRuntimeUnbindViewportSurfaceFnV1, ZrRuntimeUnwatchWorldFnV1, ZrRuntimeWatchWorldFnV1,
};
pub use host_api_shape::{
    validate_runtime_host_api_v1_pointer, validate_runtime_host_api_v1_shape,
    ZrRuntimeHostApiV1PointerError, ZrRuntimeHostApiV1ShapeError,
};

#[cfg(test)]
mod api_shape_tests;
#[cfg(test)]
mod host_api_shape_tests;
