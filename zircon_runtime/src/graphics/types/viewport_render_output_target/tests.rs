use super::*;
use crate::core::resource::ResourceId;

#[test]
fn output_target_plans_keep_diagnostic_payloads_out_of_production_builds() {
    let source = concat!(
        include_str!("writeback.rs"),
        include_str!("graph_import.rs")
    );

    for field in [
        "texture: Option<ResourceHandle<TextureMarker>>",
        "target_format: Option<String>",
        "expected_target_format: Option<String>",
        "source_format: Option<String>",
    ] {
        assert_eq!(
            source
                .matches(&format!("#[cfg(test)]\n    {field}"))
                .count(),
            2,
            "both plan types must compile {field} only for tests",
        );
    }
}

#[test]
fn output_target_writeback_plan_ignores_non_texture_targets() {
    let headless = ViewportRenderOutputTarget::Headless {
        size: UVec2::new(64, 32),
    };

    let plan = headless.writeback_plan(Some(FRAMEWORK_OUTPUT_FORMAT_LABEL));

    assert_eq!(plan.target_kind(), RenderCameraTargetKind::Headless);
    assert_eq!(plan.status(), ViewportTextureWritebackStatus::NotRequested);
    assert_eq!(plan.texture(), None);
    assert_eq!(plan.size(), None);
}

#[test]
fn output_target_writeback_plan_waits_for_target_descriptor() {
    let texture = texture_handle("tests/writeback/pending");
    let target = ViewportRenderOutputTarget::Texture {
        handle: texture,
        size: UVec2::new(128, 72),
        format: FRAMEWORK_OUTPUT_FORMAT_LABEL,
    };

    let plan = target.writeback_plan(None);

    assert_eq!(
        plan.status(),
        ViewportTextureWritebackStatus::PendingTargetDescriptor
    );
    assert_eq!(plan.texture(), Some(texture));
    assert_eq!(plan.size(), Some(UVec2::new(128, 72)));
    assert_eq!(plan.source_format(), Some(FRAMEWORK_OUTPUT_FORMAT_LABEL));
    assert_eq!(plan.target_format(), Some(FRAMEWORK_OUTPUT_FORMAT_LABEL));
    assert_eq!(
        plan.expected_target_format(),
        Some(FRAMEWORK_OUTPUT_FORMAT_LABEL)
    );
}

#[test]
fn output_target_writeback_plan_accepts_matching_srgb_format() {
    let texture = texture_handle("tests/writeback/ready");
    let target = ViewportRenderOutputTarget::Texture {
        handle: texture,
        size: UVec2::new(128, 72),
        format: FRAMEWORK_OUTPUT_FORMAT_LABEL,
    };

    let plan = target.writeback_plan(Some(" RGBA8UNORM_SRGB "));

    assert_eq!(
        plan.status(),
        ViewportTextureWritebackStatus::ReadyForSrgbCopy
    );
    assert_eq!(plan.texture(), Some(texture));
    assert_eq!(plan.size(), Some(UVec2::new(128, 72)));
    assert_eq!(plan.source_format(), Some(FRAMEWORK_OUTPUT_FORMAT_LABEL));
    assert_eq!(plan.target_format(), Some(FRAMEWORK_OUTPUT_FORMAT_LABEL));
    assert_eq!(
        plan.expected_target_format(),
        Some(FRAMEWORK_OUTPUT_FORMAT_LABEL)
    );
}

#[test]
fn output_target_writeback_plan_accepts_linear_rgba_target_for_conversion() {
    let texture = texture_handle("tests/writeback/linear");
    let target = ViewportRenderOutputTarget::Texture {
        handle: texture,
        size: UVec2::new(128, 72),
        format: LINEAR_OUTPUT_FORMAT_LABEL,
    };

    let plan = target.writeback_plan(Some("rgba8unorm"));

    assert_eq!(
        plan.status(),
        ViewportTextureWritebackStatus::ReadyForConversion
    );
    assert_eq!(plan.texture(), Some(texture));
    assert_eq!(plan.source_format(), Some(FRAMEWORK_OUTPUT_FORMAT_LABEL));
    assert_eq!(plan.target_format(), Some("rgba8unorm"));
    assert_eq!(
        plan.expected_target_format(),
        Some(LINEAR_OUTPUT_FORMAT_LABEL)
    );
}

#[test]
fn output_target_writeback_plan_blocks_unsupported_target_format() {
    let texture = texture_handle("tests/writeback/unsupported");
    let target = ViewportRenderOutputTarget::Texture {
        handle: texture,
        size: UVec2::new(128, 72),
        format: "rgba16float",
    };

    let plan = target.writeback_plan(Some("rgba16float"));

    assert_eq!(
        plan.status(),
        ViewportTextureWritebackStatus::BlockedFormatMismatch
    );
    assert_eq!(plan.texture(), Some(texture));
    assert_eq!(plan.source_format(), Some(FRAMEWORK_OUTPUT_FORMAT_LABEL));
    assert_eq!(plan.target_format(), Some("rgba16float"));
    assert_eq!(plan.expected_target_format(), Some("rgba16float"));
}

#[test]
fn output_target_writeback_plan_blocks_prepared_format_drift() {
    let texture = texture_handle("tests/writeback/prepared-format-drift");
    let target = ViewportRenderOutputTarget::Texture {
        handle: texture,
        size: UVec2::new(128, 72),
        format: LINEAR_OUTPUT_FORMAT_LABEL,
    };

    let plan = target.writeback_plan(Some(FRAMEWORK_OUTPUT_FORMAT_LABEL));

    assert_eq!(
        plan.status(),
        ViewportTextureWritebackStatus::BlockedPreparedFormatMismatch
    );
    assert_eq!(plan.texture(), Some(texture));
    assert_eq!(plan.source_format(), Some(FRAMEWORK_OUTPUT_FORMAT_LABEL));
    assert_eq!(plan.target_format(), Some(FRAMEWORK_OUTPUT_FORMAT_LABEL));
    assert_eq!(
        plan.expected_target_format(),
        Some(LINEAR_OUTPUT_FORMAT_LABEL)
    );
}

#[test]
fn output_target_graph_import_plan_marks_srgb_texture_ready_for_direct_import() {
    let texture = texture_handle("tests/graph-import/srgb");
    let target = ViewportRenderOutputTarget::Texture {
        handle: texture,
        size: UVec2::new(128, 72),
        format: FRAMEWORK_OUTPUT_FORMAT_LABEL,
    };

    let plan = target.graph_import_plan(Some("rgba8unorm_srgb"));

    assert_eq!(
        plan.status(),
        ViewportTextureGraphImportStatus::ReadyForDirectImport
    );
    assert_eq!(plan.texture(), Some(texture));
    assert_eq!(plan.size(), Some(UVec2::new(128, 72)));
    assert_eq!(plan.source_format(), Some(FRAMEWORK_OUTPUT_FORMAT_LABEL));
    assert_eq!(plan.target_format(), Some(FRAMEWORK_OUTPUT_FORMAT_LABEL));
    assert_eq!(
        plan.expected_target_format(),
        Some(FRAMEWORK_OUTPUT_FORMAT_LABEL)
    );
}

#[test]
fn output_target_graph_import_plan_keeps_linear_texture_on_conversion_writeback_path() {
    let texture = texture_handle("tests/graph-import/linear");
    let target = ViewportRenderOutputTarget::Texture {
        handle: texture,
        size: UVec2::new(128, 72),
        format: LINEAR_OUTPUT_FORMAT_LABEL,
    };

    let plan = target.graph_import_plan(Some("rgba8unorm"));

    assert_eq!(
        plan.status(),
        ViewportTextureGraphImportStatus::RequiresConversionWriteback
    );
    assert_eq!(plan.texture(), Some(texture));
    assert_eq!(plan.source_format(), Some(FRAMEWORK_OUTPUT_FORMAT_LABEL));
    assert_eq!(plan.target_format(), Some("rgba8unorm"));
    assert_eq!(
        plan.expected_target_format(),
        Some(LINEAR_OUTPUT_FORMAT_LABEL)
    );
}

#[test]
fn output_target_graph_import_plan_blocks_unsupported_target_format() {
    let texture = texture_handle("tests/graph-import/unsupported");
    let target = ViewportRenderOutputTarget::Texture {
        handle: texture,
        size: UVec2::new(128, 72),
        format: "rgba16float",
    };

    let plan = target.graph_import_plan(Some("rgba16float"));

    assert_eq!(
        plan.status(),
        ViewportTextureGraphImportStatus::BlockedFormatMismatch
    );
    assert_eq!(plan.texture(), Some(texture));
    assert_eq!(plan.target_format(), Some("rgba16float"));
    assert_eq!(plan.expected_target_format(), Some("rgba16float"));
}

#[test]
fn output_target_graph_import_plan_blocks_prepared_format_drift() {
    let texture = texture_handle("tests/graph-import/prepared-format-drift");
    let target = ViewportRenderOutputTarget::Texture {
        handle: texture,
        size: UVec2::new(128, 72),
        format: FRAMEWORK_OUTPUT_FORMAT_LABEL,
    };

    let plan = target.graph_import_plan(Some(LINEAR_OUTPUT_FORMAT_LABEL));

    assert_eq!(
        plan.status(),
        ViewportTextureGraphImportStatus::BlockedPreparedFormatMismatch
    );
    assert_eq!(plan.texture(), Some(texture));
    assert_eq!(plan.target_format(), Some(LINEAR_OUTPUT_FORMAT_LABEL));
    assert_eq!(
        plan.expected_target_format(),
        Some(FRAMEWORK_OUTPUT_FORMAT_LABEL)
    );
}

#[test]
fn output_target_from_camera_target_retains_resolved_texture_format() {
    let texture = texture_handle("tests/output-target/from-camera-target");
    let target = ViewportRenderOutputTarget::from_camera_target(
        &RenderCameraTarget::Texture(texture),
        UVec2::new(96, 54),
        Some(LINEAR_OUTPUT_FORMAT_LABEL),
    );

    assert_eq!(target.texture_handle(), Some(texture));
    assert_eq!(target.size(), Some(UVec2::new(96, 54)));
    assert_eq!(target.texture_format(), Some(LINEAR_OUTPUT_FORMAT_LABEL));
}

fn texture_handle(label: &str) -> ResourceHandle<TextureMarker> {
    ResourceHandle::new(ResourceId::from_stable_label(label))
}
