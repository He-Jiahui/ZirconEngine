//! 会话契约的结构入口，汇合创建配置、宿主事件、视口描述和异步操作的公共导出。
//! [`ZrRuntimeSessionConfigV3`] 的字节借用于同步创建，[`ZrRuntimeWakeSinkV1`] 有独立注册寿命；
//! [`GatewaySessionIdentity`] 是 Editor 保存和比较的值身份，不持有底层 Runtime 资源。
//! [`ZrRuntimeViewportCameraV1`] 通过 JSON 事件传递，原生表面绑定则使用 C 布局请求。
//! 各构造器的字段封装不能替代实际入口的预算、版本、平台或生命周期检查。
//!
//! # 参考
//! 下方子模块分别定义协商、事件、操作和视口契约；公开符号经 `runtime_api` 与 crate 根导出。
//! 两个显式测试模块仅在 `cfg(test)` 下接入，不改变生产模块路径。

#[path = "app_session_configuration_v2.rs"]
mod app_session_configuration_v2;
#[path = "camera.rs"]
mod camera;
#[path = "editor_transform.rs"]
mod editor_transform;
#[cfg(test)]
#[path = "tests/editor_transform_tests.rs"]
mod editor_transform_tests;
#[path = "events.rs"]
mod events;
#[path = "ime_compose_capability_v2.rs"]
mod ime_composition_capability_v2;
#[path = "ime_composition_v2.rs"]
mod ime_composition_v2;
#[path = "operation.rs"]
mod operation;
#[path = "plugin_event_mirror.rs"]
mod plugin_event_mirror;
#[path = "requests.rs"]
mod requests;
#[path = "session.rs"]
mod session;
#[path = "session_identity.rs"]
mod session_identity;
#[path = "translated_events.rs"]
mod translated_events;
#[path = "viewport.rs"]
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
