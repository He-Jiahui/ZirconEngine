mod abi;
mod constants;
mod frame;
mod host;
mod session;

// Frozen Runtime API versioning and dynamic entry point.
pub use crate::runtime_build_set::{ZIRCON_RUNTIME_API_VERSION_V8, ZR_RUNTIME_GET_API_SYMBOL_V8};

// Runtime table ABI and host-table validation.
pub use abi::{
    validate_runtime_api_v8_shape, validate_runtime_host_api_v1_pointer,
    validate_runtime_host_api_v1_shape, ZrHostApiV1, ZrRuntimeApiV8, ZrRuntimeApiV8ShapeError,
    ZrRuntimeBindViewportSurfaceFnV1, ZrRuntimeCancelViewportPickFnV1,
    ZrRuntimeCaptureAccessibilityTreeFnV2, ZrRuntimeCaptureFrameFnV2, ZrRuntimeCreateSessionFnV3,
    ZrRuntimeDestroySessionFnV1, ZrRuntimeDrainHostRequestsFnV2,
    ZrRuntimeDrainWorldInvalidationsFnV2, ZrRuntimeGetApiFnV8, ZrRuntimeHandleEventFnV1,
    ZrRuntimeHostApiV1PointerError, ZrRuntimeHostApiV1ShapeError, ZrRuntimeHostFetchFnV1,
    ZrRuntimePollViewportPickFnV1, ZrRuntimePresentViewportFnV1, ZrRuntimeProfileControlFnV2,
    ZrRuntimeQueryWorldFnV2, ZrRuntimeReleaseAllocationFnV2, ZrRuntimeRequestViewportPickFnV1,
    ZrRuntimeSubmitHighlightSetFnV1, ZrRuntimeTickFrameFnV2, ZrRuntimeUnbindViewportSurfaceFnV1,
    ZrRuntimeUnwatchWorldFnV1, ZrRuntimeWatchWorldFnV1,
};

// Stable wire values for surface, input, lifecycle, and host fetch requests.
pub use constants::{
    ZR_RUNTIME_BUTTON_STATE_PRESSED_V1, ZR_RUNTIME_BUTTON_STATE_RELEASED_V1,
    ZR_RUNTIME_EVENT_KIND_ACCESSIBILITY_ACTION_V1, ZR_RUNTIME_EVENT_KIND_CLIPBOARD_RESULT_V1,
    ZR_RUNTIME_EVENT_KIND_CURSOR_ENTERED_V1, ZR_RUNTIME_EVENT_KIND_CURSOR_LEFT_V1,
    ZR_RUNTIME_EVENT_KIND_EDITOR_TRANSFORM_WRITE_V1, ZR_RUNTIME_EVENT_KIND_FILE_DRAG_DROP_V1,
    ZR_RUNTIME_EVENT_KIND_GAMEPAD_AXIS_V1, ZR_RUNTIME_EVENT_KIND_GAMEPAD_BUTTON_V1,
    ZR_RUNTIME_EVENT_KIND_GAMEPAD_CONNECTION_V1, ZR_RUNTIME_EVENT_KIND_IME_V1,
    ZR_RUNTIME_EVENT_KIND_KEYBOARD_V1, ZR_RUNTIME_EVENT_KIND_LIFECYCLE_V1,
    ZR_RUNTIME_EVENT_KIND_MOUSE_BUTTON_V1, ZR_RUNTIME_EVENT_KIND_MOUSE_MOTION_V1,
    ZR_RUNTIME_EVENT_KIND_MOUSE_WHEEL_V1, ZR_RUNTIME_EVENT_KIND_POINTER_MOVED_V1,
    ZR_RUNTIME_EVENT_KIND_TOUCH_V1, ZR_RUNTIME_EVENT_KIND_VIEWPORT_CAMERA_V1,
    ZR_RUNTIME_EVENT_KIND_VIEWPORT_RESIZED_V1, ZR_RUNTIME_EVENT_KIND_WINDOW_STATUS_V1,
    ZR_RUNTIME_FETCH_FLAG_STREAMING_V1, ZR_RUNTIME_FILE_DRAG_CANCELLED_V1,
    ZR_RUNTIME_FILE_DRAG_DROPPED_V1, ZR_RUNTIME_FILE_DRAG_HOVERED_V1,
    ZR_RUNTIME_GAMEPAD_AXIS_DPAD_X_V1, ZR_RUNTIME_GAMEPAD_AXIS_DPAD_Y_V1,
    ZR_RUNTIME_GAMEPAD_AXIS_LEFT_STICK_X_V1, ZR_RUNTIME_GAMEPAD_AXIS_LEFT_STICK_Y_V1,
    ZR_RUNTIME_GAMEPAD_AXIS_LEFT_Z_V1, ZR_RUNTIME_GAMEPAD_AXIS_RIGHT_STICK_X_V1,
    ZR_RUNTIME_GAMEPAD_AXIS_RIGHT_STICK_Y_V1, ZR_RUNTIME_GAMEPAD_AXIS_RIGHT_Z_V1,
    ZR_RUNTIME_GAMEPAD_AXIS_UNKNOWN_V1, ZR_RUNTIME_GAMEPAD_BUTTON_C_V1,
    ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_DOWN_V1, ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_LEFT_V1,
    ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_RIGHT_V1, ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_UP_V1,
    ZR_RUNTIME_GAMEPAD_BUTTON_EAST_V1, ZR_RUNTIME_GAMEPAD_BUTTON_LEFT_THUMB_V1,
    ZR_RUNTIME_GAMEPAD_BUTTON_LEFT_TRIGGER2_V1, ZR_RUNTIME_GAMEPAD_BUTTON_LEFT_TRIGGER_V1,
    ZR_RUNTIME_GAMEPAD_BUTTON_MODE_V1, ZR_RUNTIME_GAMEPAD_BUTTON_NORTH_V1,
    ZR_RUNTIME_GAMEPAD_BUTTON_RIGHT_THUMB_V1, ZR_RUNTIME_GAMEPAD_BUTTON_RIGHT_TRIGGER2_V1,
    ZR_RUNTIME_GAMEPAD_BUTTON_RIGHT_TRIGGER_V1, ZR_RUNTIME_GAMEPAD_BUTTON_SELECT_V1,
    ZR_RUNTIME_GAMEPAD_BUTTON_SOUTH_V1, ZR_RUNTIME_GAMEPAD_BUTTON_START_V1,
    ZR_RUNTIME_GAMEPAD_BUTTON_UNKNOWN_V1, ZR_RUNTIME_GAMEPAD_BUTTON_WEST_V1,
    ZR_RUNTIME_GAMEPAD_BUTTON_Z_V1, ZR_RUNTIME_GAMEPAD_CONNECTION_CONNECTED_V1,
    ZR_RUNTIME_GAMEPAD_CONNECTION_DISCONNECTED_V1, ZR_RUNTIME_IME_CURSOR_HIDDEN_V1,
    ZR_RUNTIME_IME_STATE_COMMIT_V1, ZR_RUNTIME_IME_STATE_CURSOR_AREA_V1,
    ZR_RUNTIME_IME_STATE_DELETE_SURROUNDING_V1, ZR_RUNTIME_IME_STATE_DISABLED_V1,
    ZR_RUNTIME_IME_STATE_ENABLED_V1, ZR_RUNTIME_IME_STATE_PREEDIT_V1,
    ZR_RUNTIME_IME_STATE_REQUEST_DISABLE_V1, ZR_RUNTIME_IME_STATE_REQUEST_ENABLE_V1,
    ZR_RUNTIME_IME_STATE_SURROUNDING_TEXT_V1, ZR_RUNTIME_KEY_ACTION_PRESSED_V1,
    ZR_RUNTIME_KEY_ACTION_RELEASED_V1, ZR_RUNTIME_KEY_ACTION_TEXT_V1,
    ZR_RUNTIME_LIFECYCLE_STATE_BACKGROUND_V1, ZR_RUNTIME_LIFECYCLE_STATE_FOREGROUND_V1,
    ZR_RUNTIME_LIFECYCLE_STATE_LOW_MEMORY_V1, ZR_RUNTIME_LIFECYCLE_STATE_RESUMED_V1,
    ZR_RUNTIME_LIFECYCLE_STATE_SUSPENDED_V1, ZR_RUNTIME_MOUSE_BUTTON_LEFT_V1,
    ZR_RUNTIME_MOUSE_BUTTON_MIDDLE_V1, ZR_RUNTIME_MOUSE_BUTTON_RIGHT_V1,
    ZR_RUNTIME_MOUSE_WHEEL_COORDS_PRESENT_V1, ZR_RUNTIME_MOUSE_WHEEL_UNIT_LINE_V1,
    ZR_RUNTIME_MOUSE_WHEEL_UNIT_PIXEL_V1, ZR_RUNTIME_NATIVE_SURFACE_KIND_NONE_V1,
    ZR_RUNTIME_NATIVE_SURFACE_KIND_WIN32_V1, ZR_RUNTIME_TOUCH_PHASE_CANCELLED_V1,
    ZR_RUNTIME_TOUCH_PHASE_ENDED_V1, ZR_RUNTIME_TOUCH_PHASE_MOVED_V1,
    ZR_RUNTIME_TOUCH_PHASE_STARTED_V1, ZR_RUNTIME_VIEWPORT_CAMERA_PROJECTION_ORTHOGRAPHIC_V1,
    ZR_RUNTIME_VIEWPORT_CAMERA_PROJECTION_PERSPECTIVE_V1, ZR_RUNTIME_WINDOW_BOOL_FALSE_V1,
    ZR_RUNTIME_WINDOW_BOOL_TRUE_V1, ZR_RUNTIME_WINDOW_STATUS_BACKEND_SCALE_FACTOR_CHANGED_V1,
    ZR_RUNTIME_WINDOW_STATUS_CLOSE_REQUESTED_V1, ZR_RUNTIME_WINDOW_STATUS_DESTROYED_V1,
    ZR_RUNTIME_WINDOW_STATUS_MOVED_V1, ZR_RUNTIME_WINDOW_STATUS_OCCLUDED_V1,
    ZR_RUNTIME_WINDOW_STATUS_SCALE_FACTOR_CHANGED_V1,
    ZR_RUNTIME_WINDOW_STATUS_SURFACE_RECREATED_V1, ZR_RUNTIME_WINDOW_STATUS_THEME_CHANGED_V1,
    ZR_RUNTIME_WINDOW_THEME_DARK_V1, ZR_RUNTIME_WINDOW_THEME_LIGHT_V1,
    ZR_RUNTIME_WINDOW_THEME_UNKNOWN_V1,
};

// Frame scheduling, capture validation, and highlight payloads.
pub use frame::{
    validate_runtime_frame_rgba_shape, ZrRuntimeEntityIdSliceV1, ZrRuntimeFrameDemandV1,
    ZrRuntimeFrameRgbaShapeError, ZrRuntimeHighlightRenderAttributesV1, ZrRuntimeHighlightSetV1,
    ZrRuntimeViewportPickDispositionV1, ZrRuntimeViewportPickPurposeV1,
    ZrRuntimeViewportPickRequestV1, ZrRuntimeViewportPickResultV1, ZrRuntimeViewportPickTicket,
    ZrRuntimeViewportPixelV1, ZR_RUNTIME_FRAME_DEMAND_AFTER_V1, ZR_RUNTIME_FRAME_DEMAND_IDLE_V1,
    ZR_RUNTIME_FRAME_DEMAND_IMMEDIATE_V1, ZR_RUNTIME_VIEWPORT_PICK_POLICY_INCLUDE_BACKFACES_V1,
    ZR_RUNTIME_VIEWPORT_PICK_POLICY_INCLUDE_TRANSLUCENT_V1,
};

// Runtime-originated host requests.
pub use host::{
    ZrRuntimeClipboardHostRequestV1, ZrRuntimeClipboardResultV1, ZrRuntimeCursorGrabModeV1,
    ZrRuntimeCursorHostRequestKindV1, ZrRuntimeCursorHostRequestV1, ZrRuntimeCursorPositionV1,
    ZrRuntimeGamepadRumbleRequestKindV1, ZrRuntimeGamepadRumbleRequestV1,
    ZrRuntimeHostRequestBatchV1, ZrRuntimeHostRequestV1, ZrRuntimeImeCoordinateSpaceV1,
    ZrRuntimeImeCursorAreaV1, ZrRuntimeImeHostRequestKindV1, ZrRuntimeImeHostRequestV1,
    ZrRuntimeImeSurroundingTextV1, ZrRuntimeImeTextRangeV1,
    ZrRuntimeProjectSceneTransitionPolicyV1, ZrRuntimeProjectSceneTransitionRequestErrorV1,
    ZrRuntimeProjectSceneTransitionRequestV1, ZrRuntimeProjectSceneTransitionResultV1,
    ZrRuntimeProjectSceneTransitionStatusV1, ZrRuntimeUiActionHostRequestV1,
    ZrRuntimeUiHostRequestKindV1, ZrRuntimeUiHostRequestV1,
};

// Session lifecycle, event, operation, plugin event, and viewport contracts.
pub use session::{
    GatewaySessionIdentity, ZrRuntimeAccessibilityTreeRequestV1,
    ZrRuntimeBindViewportSurfaceRequestV1, ZrRuntimeDrainPluginEventsFnV2,
    ZrRuntimeEditorTransformError, ZrRuntimeEditorTransformPhaseV1,
    ZrRuntimeEditorTransformWriteV1, ZrRuntimeEventV1, ZrRuntimeFrameRequestV1, ZrRuntimeFrameV2,
    ZrRuntimeHarvestOperationFnV2, ZrRuntimeHostFetchRequestV1, ZrRuntimeNativeSurfaceTargetV1,
    ZrRuntimeOperationDetailKindV2, ZrRuntimeOperationHandle, ZrRuntimeOperationOutcomeV1,
    ZrRuntimeOperationPhase, ZrRuntimeOperationResultV1, ZrRuntimeOperationStatusV2,
    ZrRuntimeOperationSubmitRequestV1, ZrRuntimePluginEventDeliveryBatchV1,
    ZrRuntimePluginEventDeliveryV1, ZrRuntimePluginEventSubscribeRequestV1,
    ZrRuntimePluginEventSubscriptionHandle, ZrRuntimePollOperationFnV2, ZrRuntimeSessionConfigV3,
    ZrRuntimeSubmitOperationFnV1, ZrRuntimeSubscribePluginEventFnV1, ZrRuntimeTransformV1,
    ZrRuntimeTranslatedEventV1, ZrRuntimeUnsubscribePluginEventFnV1, ZrRuntimeViewportCameraV1,
    ZrRuntimeViewportMetricsV1, ZrRuntimeViewportSizeV1, ZrRuntimeWakeSinkV1,
    ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_DELIVERIES_V1,
    ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_ENCODED_BYTES_V1,
};
