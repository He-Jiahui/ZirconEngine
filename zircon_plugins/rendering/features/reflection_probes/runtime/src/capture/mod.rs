//! 探针捕获的请求、框架执行、cubemap 朝向与结果消费边界；源发布由编辑器/资产管理器负责。
#[path = "../consume.rs"]
mod consume;
#[path = "../execute.rs"]
mod execute;
#[path = "../face_view.rs"]
mod face_view;
#[path = "../request.rs"]
mod request;

pub use consume::{
    encode_reflection_probe_capture_source, register_captured_reflection_probe,
    register_captured_reflection_probe_from_runtime_cache, CapturedReflectionProbeAsset,
    CapturedReflectionProbeConsumeError, CapturedReflectionProbeInfluence,
    CapturedReflectionProbePlacement, EncodedReflectionProbeCaptureSource,
    PersistedReflectionProbeCapture,
};
pub use execute::{
    cancel_reflection_probe_capture, poll_reflection_probe_capture,
    request_reflection_probe_capture, request_reflection_probe_capture_with_placement,
    take_reflection_probe_capture_source, ReflectionProbeCaptureError,
};
pub use face_view::{
    ReflectionProbeCaptureFace, ReflectionProbeCaptureFaceView,
    ReflectionProbeCaptureStorageTransform, REFLECTION_PROBE_CAPTURE_FACE_VIEWS,
};
pub use request::{
    ReflectionProbeCaptureQuality, ReflectionProbeCaptureRequest,
    ReflectionProbeCaptureRequestError, REFLECTION_PROBE_CAPTURE_REQUEST_SCHEMA_VERSION,
};
