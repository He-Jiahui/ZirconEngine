use zircon_runtime::graphics::RenderFeatureDescriptor;

mod capability;
mod capture;
mod plugin;

pub use capability::{EDITOR_CAPABILITY, RUNTIME_CAPABILITIES, RUNTIME_CAPABILITY};
pub use capture::{
    CapturedReflectionProbeAsset, CapturedReflectionProbeConsumeError,
    CapturedReflectionProbeInfluence, CapturedReflectionProbePlacement,
    EncodedReflectionProbeCaptureSource, PersistedReflectionProbeCapture,
    REFLECTION_PROBE_CAPTURE_FACE_VIEWS, REFLECTION_PROBE_CAPTURE_REQUEST_SCHEMA_VERSION,
    ReflectionProbeCaptureError, ReflectionProbeCaptureFace, ReflectionProbeCaptureFaceView,
    ReflectionProbeCaptureQuality, ReflectionProbeCaptureRequest,
    ReflectionProbeCaptureRequestError, ReflectionProbeCaptureStorageTransform,
    cancel_reflection_probe_capture, encode_reflection_probe_capture_source,
    poll_reflection_probe_capture, register_captured_reflection_probe,
    register_captured_reflection_probe_from_runtime_cache, request_reflection_probe_capture,
    request_reflection_probe_capture_with_placement, take_reflection_probe_capture_source,
};
pub use plugin::{
    RenderingReflectionProbesRuntimeFeature, feature_manifest, plugin_feature_registration,
    runtime_plugin_feature,
};

pub const FEATURE_ID: &str = "rendering.reflection_probes";
pub const FEATURE_NAME: &str = "reflection_probes";
pub fn render_feature_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        FEATURE_NAME,
        vec![
            "view".to_string(),
            "lighting".to_string(),
            "post_process".to_string(),
        ],
        Vec::new(),
        Vec::new(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reflection_probes_feature_has_no_unrequested_capture_or_composite_pass() {
        let report = plugin_feature_registration();

        assert!(report.is_success(), "{:?}", report.diagnostics);
        assert!(report.manifest.enabled_by_default);
        assert!(
            report.extensions.render_features()[0]
                .stage_passes
                .is_empty()
        );
        assert!(report.extensions.render_pass_executors().is_empty());
    }
}
