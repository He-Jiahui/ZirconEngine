mod app_session_configuration_v2;
mod camera;
mod editor_transform;
#[cfg(test)]
#[path = "tests/editor_transform_tests.rs"]
mod editor_transform_tests;
mod events;
mod ime_composition_capability_v2;
mod ime_composition_v2;
mod operation;
mod plugin_event_mirror;
mod requests;
mod session;
mod session_identity;
mod translated_events;
mod viewport;

pub use app_session_configuration_v2::{
    ZrRuntimeAppSessionConfigurationError, ZrRuntimeAppSessionConfigurationV2,
    ZrRuntimeImeCandidateRectV2, ZR_RUNTIME_APP_SESSION_CANDIDATE_RECT_SCHEMA_V2,
    ZR_RUNTIME_APP_SESSION_CAPABILITY_CANDIDATE_RECT_V2,
    ZR_RUNTIME_APP_SESSION_CAPABILITY_IME_COMPOSITION_V2,
    ZR_RUNTIME_APP_SESSION_COMPOSITION_SCHEMA_V2,
    ZR_RUNTIME_APP_SESSION_CONFIGURATION_ABI_VERSION_V2,
    ZR_RUNTIME_APP_SESSION_COORDINATE_SPACE_WINDOW_V1,
    ZR_RUNTIME_APP_SESSION_MAX_NATIVE_COORDINATE_V2, ZR_RUNTIME_CONFIGURE_APP_SESSION_SYMBOL_V2,
};
pub use camera::ZrRuntimeViewportCameraV1;
pub use editor_transform::{
    ZrRuntimeEditorTransformError, ZrRuntimeEditorTransformPhaseV1,
    ZrRuntimeEditorTransformWriteV1, ZrRuntimeTransformV1,
};
pub use events::ZrRuntimeEventV1;
pub use ime_composition_capability_v2::{
    ZrRuntimeImeCompositionNegotiation, ZrRuntimeImeCompositionWireVersion,
    ZR_RUNTIME_IME_COMPOSITION_V2_CAPABILITY, ZR_RUNTIME_IME_COMPOSITION_V2_EVENT_STATE,
    ZR_RUNTIME_IME_COMPOSITION_V2_MIN_API_VERSION,
};
pub use ime_composition_v2::{
    ZrRuntimeImeCompositionContextV2, ZrRuntimeImeCompositionOperationV2,
    ZrRuntimeImeCompositionV2, ZrRuntimeImeCompositionV2Error,
    ZrRuntimeImePreeditClauseAvailabilityV2, ZR_RUNTIME_IME_COMPOSITION_V2_CLAUSE_BYTES,
    ZR_RUNTIME_IME_COMPOSITION_V2_HEADER_BYTES, ZR_RUNTIME_IME_COMPOSITION_V2_MAGIC,
    ZR_RUNTIME_IME_COMPOSITION_V2_MAX_CLAUSES, ZR_RUNTIME_IME_COMPOSITION_V2_SCHEMA,
};
pub use operation::{
    ZrRuntimeHarvestOperationFnV2, ZrRuntimeOperationDetailKindV2, ZrRuntimeOperationHandle,
    ZrRuntimeOperationOutcomeV1, ZrRuntimeOperationPhase, ZrRuntimeOperationResultV1,
    ZrRuntimeOperationStatusV2, ZrRuntimeOperationSubmitRequestV1, ZrRuntimePollOperationFnV2,
    ZrRuntimeSubmitOperationFnV1,
};
pub use plugin_event_mirror::{
    ZrRuntimeDrainPluginEventsFnV2, ZrRuntimePluginEventDeliveryBatchV1,
    ZrRuntimePluginEventDeliveryV1, ZrRuntimePluginEventSubscribeRequestV1,
    ZrRuntimePluginEventSubscriptionHandle, ZrRuntimeSubscribePluginEventFnV1,
    ZrRuntimeUnsubscribePluginEventFnV1, ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_DELIVERIES_V1,
    ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_ENCODED_BYTES_V1,
};
pub use requests::{
    ZrRuntimeAccessibilityTreeRequestV1, ZrRuntimeFrameRequestV1, ZrRuntimeFrameV2,
    ZrRuntimeHostFetchRequestV1,
};
pub use session::{ZrRuntimeSessionConfigV3, ZrRuntimeWakeSinkV1};
pub use session_identity::GatewaySessionIdentity;
pub use translated_events::ZrRuntimeTranslatedEventV1;
pub use viewport::{
    ZrRuntimeBindViewportSurfaceRequestV1, ZrRuntimeNativeSurfaceTargetV1,
    ZrRuntimeViewportMetricsV1, ZrRuntimeViewportSizeV1,
};

#[cfg(test)]
#[path = "tests/session_identity_tests.rs"]
mod session_identity_tests;
