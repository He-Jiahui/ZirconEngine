//! Stable ABI and DTO contracts shared by runtime hosts, editors, and plugins.

pub mod buffer;
pub mod editor_contribution;
pub mod export;
pub mod handles;
pub mod hub_protocol;
pub mod manifest;
pub mod math;
pub mod plugin_api;
pub mod plugin_diagnostics;
pub mod plugin_events;
pub mod profiling;
pub mod project;
pub mod reflect;
pub mod resource;
pub mod runtime_api;
pub mod script_diagnostics;
pub mod serialization;
pub mod status;
pub mod ui;
pub mod version;
pub mod world_sync;

pub use buffer::{
    ZrByteBufferRef, ZrByteSlice, ZrByteSliceError, ZrFreeBytesFn, ZrOwnedByteBuffer,
    ZrOwnedResultV2, ZrRuntimePayloadLimitV1, ZR_RUNTIME_ACCESSIBILITY_ACTION_REQUEST_LIMIT_V1,
    ZR_RUNTIME_ACCESSIBILITY_TREE_OUTPUT_LIMIT_V1, ZR_RUNTIME_EVENT_PAYLOAD_MAX_ENCODED_BYTES_V1,
    ZR_RUNTIME_FRAME_MAX_DIMENSION_V1, ZR_RUNTIME_FRAME_MAX_RGBA_BYTES_V1,
    ZR_RUNTIME_HOST_REQUEST_OUTPUT_LIMIT_V1, ZR_RUNTIME_JSON_MAX_NESTING_DEPTH_V1,
    ZR_RUNTIME_NATIVE_STRING_LIST_MAX_ITEMS_V1, ZR_RUNTIME_NATIVE_STRING_MAX_ENCODED_BYTES_V1,
    ZR_RUNTIME_OPERATION_REQUEST_LIMIT_V1, ZR_RUNTIME_OPERATION_RESULT_OUTPUT_LIMIT_V1,
    ZR_RUNTIME_PLUGIN_EVENT_OUTPUT_LIMIT_V1, ZR_RUNTIME_PLUGIN_EVENT_SUBSCRIBE_REQUEST_LIMIT_V1,
    ZR_RUNTIME_PROFILE_REQUEST_LIMIT_V1, ZR_RUNTIME_PROFILE_RESPONSE_OUTPUT_LIMIT_V1,
    ZR_RUNTIME_PROJECT_PATH_MAX_ENCODED_BYTES_V1, ZR_RUNTIME_SESSION_PROFILE_MAX_ENCODED_BYTES_V1,
    ZR_RUNTIME_STATUS_DIAGNOSTICS_MAX_ENCODED_BYTES_V1,
    ZR_RUNTIME_WORLD_INVALIDATION_OUTPUT_LIMIT_V1, ZR_RUNTIME_WORLD_QUERY_OUTPUT_LIMIT_V1,
    ZR_RUNTIME_WORLD_QUERY_REQUEST_LIMIT_V1, ZR_RUNTIME_WORLD_WATCH_REQUEST_LIMIT_V1,
};
pub use editor_contribution::{
    SerializedContributionBatch, SerializedContributionBatchError, SerializedEditorContribution,
    SERIALIZED_EDITOR_CONTRIBUTION_BATCH_SCHEMA_V1,
};
pub use handles::{
    ZrRuntimeAllocationId, ZrRuntimePluginHandle, ZrRuntimeSessionHandle, ZrRuntimeViewportHandle,
    ZIRCON_RUNTIME_DEFAULT_VIEWPORT_HANDLE_V1,
};
#[cfg(windows)]
pub use hub_protocol::windows_hub_recent_projects_mutex_name;
pub use hub_protocol::{
    hub_editor_focus_signal_path, hub_recent_project_path_key, hub_recent_projects_lock_path,
    hub_recent_projects_path, hub_recent_projects_path_from_home, merge_hub_recent_projects,
    HubEditorFocusSignalPathError, HubEditorFocusSignalV1, HubEditorLaunchOutcomeV1,
    HubEditorMailboxV1, HubProtocolVersionV1, HubRecentProjectV1, HubRecentProjectsError,
    HubRecentProjectsV1, HubSessionToken, HubSessionTokenParseError, HUB_PROTOCOL_VERSION_V1,
    HUB_RECENT_PROJECT_LIMIT_V1,
};
pub use manifest::{ZrPluginModuleDescriptorV1, ZrPluginModuleKind, ZrRuntimeTargetMode};
pub use plugin_api::{
    ZrComponentDescV1, ZrEventTypeId, ZrHostApiV3, ZrHostApiV4, ZrHostAssetApiV1,
    ZrHostAssetRequestFnV1, ZrHostBridgeApiV1, ZrHostBridgeCallFnV1, ZrHostDiagnosticsApiV1,
    ZrHostDiagnosticsEmitFnV1, ZrHostDiagnosticsMetricFnV1, ZrHostEcsApiV1, ZrHostEcsApiV2,
    ZrHostEventApiV1, ZrHostEventDrainFnV1, ZrHostEventEmitFnV1, ZrHostRegisterComponentFnV1,
    ZrHostRegisterSystemFnV1, ZrHostRegisterSystemFnV2, ZrHostSpawnCommandFnV1,
    ZrNativeSystemAccessV1, ZrNativeSystemInvokeFnV1, ZrPluginApiV1, ZrPluginEntryFnV1,
    ZrPluginEntryFnV3, ZrPluginEntryFnV4, ZrPluginEntryReportV1, ZrPluginSnapshotRestoreFnV1,
    ZrPluginSnapshotSaveFnV1, ZrPluginStateSnapshotApiV1, ZrSystemRegistrationV1,
    ZrSystemRegistrationV2, ZR_NATIVE_SYSTEM_ACCESS_DOMAIN_COMPONENT_V1,
    ZR_NATIVE_SYSTEM_ACCESS_DOMAIN_RESOURCE_V1, ZR_NATIVE_SYSTEM_ACCESS_MODE_READ_V1,
    ZR_NATIVE_SYSTEM_ACCESS_MODE_WRITE_V1, ZR_NATIVE_SYSTEM_THREAD_AFFINITY_MAIN_THREAD_ONLY_V1,
    ZR_NATIVE_SYSTEM_THREAD_AFFINITY_WORKER_SAFE_V1, ZR_PLUGIN_ENTRY_SYMBOL_V1,
    ZR_PLUGIN_ENTRY_SYMBOL_V3, ZR_PLUGIN_ENTRY_SYMBOL_V4,
};
pub use plugin_diagnostics::{RegistrationDiagnostic, RegistrationDiagnosticSeverity};
pub use plugin_events::{
    ZrPluginEventCallbackFnV1, ZrPluginEventCallbackRequestV1, ZrPluginEventCallbackResultV1,
};
pub use profiling::{
    CounterHotspotEntry, CounterHotspotReport, HotspotEntry, HotspotReport, ProfileCaptureConfig,
    ProfileControlCommand, ProfileControlRequest, ProfileControlResponse, ProfileCounterSnapshot,
    ProfileFrameSnapshot, ProfileRecorderRetentionSnapshot, ProfileSampleRetentionSnapshot,
    ProfileSnapshot, ProfileSpanSnapshot, RuntimeDiagnosticMeasurement,
    RuntimeDiagnosticSeriesSnapshot, RuntimeDiagnosticsSnapshot, RuntimeInputDiagnosticsSnapshot,
    RuntimeRenderDeviceDiagnosticsSnapshot, RuntimeSceneAssetReloadDiagnostics, UiHotspotAlert,
    UiHotspotReport, UiScenarioHotspot, ZrRuntimeProfileControlFnV2, PROFILE_COUNTER_HOTSPOTS_FILE,
    PROFILE_DEFAULT_FRAME_BUDGET_MS, PROFILE_DEFAULT_MAX_COUNTERS, PROFILE_DEFAULT_MAX_FRAMES,
    PROFILE_DEFAULT_MAX_SPANS, PROFILE_DEFAULT_OUTPUT_ROOT, PROFILE_DEFAULT_SESSION_ID,
    PROFILE_HOTSPOTS_FILE, PROFILE_SUMMARY_FILE, PROFILE_TIMELINE_NATIVE_FILE,
    PROFILE_TIMELINE_PERFETTO_FILE, PROFILE_UI_HOTSPOTS_FILE,
};
pub use runtime_api::ZrRuntimeImeTextRangeV1;
pub use runtime_api::{
    ZrHostApiV1, ZrRuntimeAccessibilityTreeRequestV1, ZrRuntimeApiV7,
    ZrRuntimeBindViewportSurfaceFnV1, ZrRuntimeBindViewportSurfaceRequestV1,
    ZrRuntimeCaptureAccessibilityTreeFnV2, ZrRuntimeCaptureFrameFnV2, ZrRuntimeCreateSessionFnV3,
    ZrRuntimeCursorGrabModeV1, ZrRuntimeCursorHostRequestKindV1, ZrRuntimeCursorHostRequestV1,
    ZrRuntimeCursorPositionV1, ZrRuntimeDrainHostRequestsFnV2, ZrRuntimeDrainPluginEventsFnV2,
    ZrRuntimeDrainWorldInvalidationsFnV2, ZrRuntimeEntityIdSliceV1, ZrRuntimeEventV1,
    ZrRuntimeFrameDemandV1, ZrRuntimeFrameRequestV1, ZrRuntimeFrameV2,
    ZrRuntimeGamepadRumbleRequestKindV1, ZrRuntimeGamepadRumbleRequestV1, ZrRuntimeGetApiFnV7,
    ZrRuntimeHarvestOperationFnV2, ZrRuntimeHighlightRenderAttributesV1, ZrRuntimeHighlightSetV1,
    ZrRuntimeHostFetchFnV1, ZrRuntimeHostFetchRequestV1, ZrRuntimeHostRequestBatchV1,
    ZrRuntimeHostRequestV1, ZrRuntimeImeCoordinateSpaceV1, ZrRuntimeImeCursorAreaV1,
    ZrRuntimeImeHostRequestKindV1, ZrRuntimeImeHostRequestV1, ZrRuntimeImeSurroundingTextV1,
    ZrRuntimeNativeSurfaceTargetV1, ZrRuntimeOperationDetailKindV2, ZrRuntimeOperationHandle,
    ZrRuntimeOperationOutcomeV1, ZrRuntimeOperationPhase, ZrRuntimeOperationResultV1,
    ZrRuntimeOperationStatusV2, ZrRuntimeOperationSubmitRequestV1,
    ZrRuntimePluginEventDeliveryBatchV1, ZrRuntimePluginEventDeliveryV1,
    ZrRuntimePluginEventSubscribeRequestV1, ZrRuntimePluginEventSubscriptionHandle,
    ZrRuntimePollOperationFnV2, ZrRuntimePresentViewportFnV1, ZrRuntimeQueryWorldFnV2,
    ZrRuntimeReleaseAllocationFnV2, ZrRuntimeSessionConfigV3, ZrRuntimeSubmitHighlightSetFnV1,
    ZrRuntimeSubmitOperationFnV1, ZrRuntimeSubscribePluginEventFnV1, ZrRuntimeTickFrameFnV2,
    ZrRuntimeTranslatedEventV1, ZrRuntimeUnbindViewportSurfaceFnV1,
    ZrRuntimeUnsubscribePluginEventFnV1, ZrRuntimeUnwatchWorldFnV1, ZrRuntimeViewportMetricsV1,
    ZrRuntimeViewportSizeV1, ZrRuntimeWakeSinkV1, ZrRuntimeWatchWorldFnV1,
    ZR_RUNTIME_BUTTON_STATE_PRESSED_V1, ZR_RUNTIME_BUTTON_STATE_RELEASED_V1,
    ZR_RUNTIME_EVENT_KIND_ACCESSIBILITY_ACTION_V1, ZR_RUNTIME_EVENT_KIND_CURSOR_ENTERED_V1,
    ZR_RUNTIME_EVENT_KIND_CURSOR_LEFT_V1, ZR_RUNTIME_EVENT_KIND_FILE_DRAG_DROP_V1,
    ZR_RUNTIME_EVENT_KIND_GAMEPAD_AXIS_V1, ZR_RUNTIME_EVENT_KIND_GAMEPAD_BUTTON_V1,
    ZR_RUNTIME_EVENT_KIND_GAMEPAD_CONNECTION_V1, ZR_RUNTIME_EVENT_KIND_IME_V1,
    ZR_RUNTIME_EVENT_KIND_KEYBOARD_V1, ZR_RUNTIME_EVENT_KIND_LIFECYCLE_V1,
    ZR_RUNTIME_EVENT_KIND_MOUSE_BUTTON_V1, ZR_RUNTIME_EVENT_KIND_MOUSE_MOTION_V1,
    ZR_RUNTIME_EVENT_KIND_MOUSE_WHEEL_V1, ZR_RUNTIME_EVENT_KIND_POINTER_MOVED_V1,
    ZR_RUNTIME_EVENT_KIND_TOUCH_V1, ZR_RUNTIME_EVENT_KIND_VIEWPORT_RESIZED_V1,
    ZR_RUNTIME_EVENT_KIND_WINDOW_STATUS_V1, ZR_RUNTIME_FETCH_FLAG_STREAMING_V1,
    ZR_RUNTIME_FILE_DRAG_CANCELLED_V1, ZR_RUNTIME_FILE_DRAG_DROPPED_V1,
    ZR_RUNTIME_FILE_DRAG_HOVERED_V1, ZR_RUNTIME_FRAME_DEMAND_AFTER_V1,
    ZR_RUNTIME_FRAME_DEMAND_IDLE_V1, ZR_RUNTIME_FRAME_DEMAND_IMMEDIATE_V1,
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
    ZR_RUNTIME_GAMEPAD_CONNECTION_DISCONNECTED_V1, ZR_RUNTIME_GET_API_SYMBOL_V7,
    ZR_RUNTIME_IME_CURSOR_HIDDEN_V1, ZR_RUNTIME_IME_STATE_COMMIT_V1,
    ZR_RUNTIME_IME_STATE_CURSOR_AREA_V1, ZR_RUNTIME_IME_STATE_DELETE_SURROUNDING_V1,
    ZR_RUNTIME_IME_STATE_DISABLED_V1, ZR_RUNTIME_IME_STATE_ENABLED_V1,
    ZR_RUNTIME_IME_STATE_PREEDIT_V1, ZR_RUNTIME_IME_STATE_REQUEST_DISABLE_V1,
    ZR_RUNTIME_IME_STATE_REQUEST_ENABLE_V1, ZR_RUNTIME_IME_STATE_SURROUNDING_TEXT_V1,
    ZR_RUNTIME_KEY_ACTION_PRESSED_V1, ZR_RUNTIME_KEY_ACTION_RELEASED_V1,
    ZR_RUNTIME_KEY_ACTION_TEXT_V1, ZR_RUNTIME_LIFECYCLE_STATE_BACKGROUND_V1,
    ZR_RUNTIME_LIFECYCLE_STATE_FOREGROUND_V1, ZR_RUNTIME_LIFECYCLE_STATE_LOW_MEMORY_V1,
    ZR_RUNTIME_LIFECYCLE_STATE_RESUMED_V1, ZR_RUNTIME_LIFECYCLE_STATE_SUSPENDED_V1,
    ZR_RUNTIME_MOUSE_BUTTON_LEFT_V1, ZR_RUNTIME_MOUSE_BUTTON_MIDDLE_V1,
    ZR_RUNTIME_MOUSE_BUTTON_RIGHT_V1, ZR_RUNTIME_MOUSE_WHEEL_COORDS_PRESENT_V1,
    ZR_RUNTIME_MOUSE_WHEEL_UNIT_LINE_V1, ZR_RUNTIME_MOUSE_WHEEL_UNIT_PIXEL_V1,
    ZR_RUNTIME_NATIVE_SURFACE_KIND_NONE_V1, ZR_RUNTIME_NATIVE_SURFACE_KIND_WIN32_V1,
    ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_DELIVERIES_V1,
    ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_ENCODED_BYTES_V1, ZR_RUNTIME_TOUCH_PHASE_CANCELLED_V1,
    ZR_RUNTIME_TOUCH_PHASE_ENDED_V1, ZR_RUNTIME_TOUCH_PHASE_MOVED_V1,
    ZR_RUNTIME_TOUCH_PHASE_STARTED_V1, ZR_RUNTIME_WINDOW_BOOL_FALSE_V1,
    ZR_RUNTIME_WINDOW_BOOL_TRUE_V1, ZR_RUNTIME_WINDOW_STATUS_BACKEND_SCALE_FACTOR_CHANGED_V1,
    ZR_RUNTIME_WINDOW_STATUS_CLOSE_REQUESTED_V1, ZR_RUNTIME_WINDOW_STATUS_DESTROYED_V1,
    ZR_RUNTIME_WINDOW_STATUS_MOVED_V1, ZR_RUNTIME_WINDOW_STATUS_OCCLUDED_V1,
    ZR_RUNTIME_WINDOW_STATUS_SCALE_FACTOR_CHANGED_V1, ZR_RUNTIME_WINDOW_STATUS_THEME_CHANGED_V1,
    ZR_RUNTIME_WINDOW_THEME_DARK_V1, ZR_RUNTIME_WINDOW_THEME_LIGHT_V1,
    ZR_RUNTIME_WINDOW_THEME_UNKNOWN_V1,
};
pub use script_diagnostics::{ScriptDiagnostic, ScriptDiagnosticSeverity, ScriptSourceLocation};
pub use status::{ZrStatus, ZrStatusCode};
pub use version::{
    ZIRCON_RUNTIME_ABI_VERSION_V1, ZIRCON_RUNTIME_ABI_VERSION_V2, ZIRCON_RUNTIME_ABI_VERSION_V3,
    ZIRCON_RUNTIME_API_VERSION_V7,
};

#[cfg(test)]
mod tests;
