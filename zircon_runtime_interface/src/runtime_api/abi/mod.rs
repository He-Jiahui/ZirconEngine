//! 冻结函数表、函数指针类型和表头检查的公共出口。
//!
//! [`validate_runtime_api_v8_shape`] 与 Host 表形状检查只处理版本和大小；
//! 原始指针读取、提供方身份、必需槽与动态库有效期须在调用边界分别建立。

#[path = "api_shape.rs"]
mod api_shape;
#[path = "api_table.rs"]
mod api_table;
#[path = "host_api_shape.rs"]
mod host_api_shape;

pub use api_shape::{validate_runtime_api_v8_shape, ZrRuntimeApiV8ShapeError};
pub use api_table::{
    ZrHostApiV1, ZrRuntimeApiV8, ZrRuntimeBindViewportSurfaceFnV1,
    ZrRuntimeBindViewportSurfaceFnV2, ZrRuntimeCancelViewportPickFnV1,
    ZrRuntimeCaptureAccessibilityTreeFnV2, ZrRuntimeCaptureFrameFnV2,
    ZrRuntimeConfigureAppSessionFnV2, ZrRuntimeCreateSessionFnV3, ZrRuntimeDestroySessionFnV1,
    ZrRuntimeDrainHostRequestsFnV2, ZrRuntimeDrainWorldInvalidationsFnV2, ZrRuntimeGetApiFnV8,
    ZrRuntimeHandleEventFnV1, ZrRuntimeHostFetchFnV1, ZrRuntimePollViewportPickFnV1,
    ZrRuntimePresentViewportFnV1, ZrRuntimeProfileControlFnV2, ZrRuntimeQueryWorldFnV2,
    ZrRuntimeReleaseAllocationFnV2, ZrRuntimeRequestViewportPickFnV1,
    ZrRuntimeSubmitHighlightSetFnV1, ZrRuntimeTickFrameFnV2, ZrRuntimeUnbindViewportSurfaceFnV1,
    ZrRuntimeUnwatchWorldFnV1, ZrRuntimeWatchWorldFnV1,
};
pub use host_api_shape::{
    validate_runtime_host_api_v1_pointer, validate_runtime_host_api_v1_shape,
    ZrRuntimeHostApiV1PointerError, ZrRuntimeHostApiV1ShapeError,
};

#[cfg(test)]
#[path = "tests/api_shape_tests.rs"]
mod api_shape_tests;
#[cfg(test)]
#[path = "tests/host_api_shape_tests.rs"]
mod host_api_shape_tests;
