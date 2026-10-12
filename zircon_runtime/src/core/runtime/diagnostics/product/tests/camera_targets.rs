use crate::core::diagnostics::DiagnosticStore;
use crate::core::framework::render::{
    RenderCameraTargetGraphImportReport, RenderCameraTargetKind, RenderCameraTargetWritebackReport,
    RenderCaptureReport, RenderCaptureSource, RenderStats,
};
use crate::core::math::UVec2;

use super::super::record;
use super::assert_series;

#[test]
fn render_product_diagnostics_record_texture_conversion_writeback_marker() {
    let mut store = DiagnosticStore::default();
    let stats = RenderStats {
        submitted_frames: 12,
        last_camera_target_writeback: RenderCameraTargetWritebackReport::converted(UVec2::new(
            72, 40,
        )),
        ..RenderStats::default()
    };

    record(&mut store, &stats);

    assert_series(
        &store,
        "render.camera.target.writeback.converted",
        1.0,
        "bool",
    );
    assert_series(
        &store,
        "render.camera.target.writeback.converted_count",
        1.0,
        "count",
    );
    assert_series(
        &store,
        "render.camera.target.writeback.conversion_debug_marker_emitted",
        1.0,
        "bool",
    );
    assert_series(
        &store,
        "render.camera.target.writeback.debug_marker_emitted",
        0.0,
        "bool",
    );
    assert_series(
        &store,
        "render.camera.target.writeback.width",
        72.0,
        "count",
    );
    assert_series(
        &store,
        "render.camera.target.writeback.height",
        40.0,
        "count",
    );
}

#[test]
fn render_product_diagnostics_record_texture_direct_graph_import_readiness() {
    let mut store = DiagnosticStore::default();
    let stats = RenderStats {
        submitted_frames: 12,
        last_camera_target_graph_import:
            RenderCameraTargetGraphImportReport::ready_for_direct_import(UVec2::new(96, 54)),
        ..RenderStats::default()
    };

    record(&mut store, &stats);

    assert_series(
        &store,
        "render.camera.target.graph_import.ready_for_direct_import",
        1.0,
        "bool",
    );
    assert_series(
        &store,
        "render.camera.target.graph_import.direct_imported",
        0.0,
        "bool",
    );
    assert_series(
        &store,
        "render.camera.target.graph_import.requires_conversion_writeback",
        0.0,
        "bool",
    );
    assert_series(
        &store,
        "render.camera.target.graph_import.direct_import_count",
        0.0,
        "count",
    );
    assert_series(
        &store,
        "render.camera.target.graph_import.conversion_writeback_count",
        0.0,
        "count",
    );
    assert_series(
        &store,
        "render.camera.target.graph_import.width",
        96.0,
        "count",
    );
    assert_series(
        &store,
        "render.camera.target.graph_import.height",
        54.0,
        "count",
    );
}

#[test]
fn render_product_diagnostics_record_texture_direct_graph_import_execution() {
    let mut store = DiagnosticStore::default();
    let stats = RenderStats {
        submitted_frames: 12,
        last_camera_target_graph_import: RenderCameraTargetGraphImportReport::direct_imported(
            UVec2::new(96, 54),
        ),
        last_camera_target_writeback: RenderCameraTargetWritebackReport::skipped_direct_import(
            UVec2::new(96, 54),
        ),
        ..RenderStats::default()
    };

    record(&mut store, &stats);

    assert_series(
        &store,
        "render.camera.target.graph_import.ready_for_direct_import",
        0.0,
        "bool",
    );
    assert_series(
        &store,
        "render.camera.target.graph_import.direct_imported",
        1.0,
        "bool",
    );
    assert_series(
        &store,
        "render.camera.target.graph_import.direct_import_count",
        1.0,
        "count",
    );
    assert_series(
        &store,
        "render.camera.target.writeback.skipped_direct_import",
        1.0,
        "bool",
    );
    assert_series(
        &store,
        "render.camera.target.writeback.copy_count",
        0.0,
        "count",
    );
}

#[test]
fn render_product_diagnostics_record_camera_stack_suppressed_target_output() {
    let mut store = DiagnosticStore::default();
    let stats = RenderStats {
        submitted_frames: 12,
        last_camera_target_graph_import:
            RenderCameraTargetGraphImportReport::suppressed_by_camera_stack(UVec2::new(96, 54)),
        last_camera_target_writeback: RenderCameraTargetWritebackReport::suppressed_by_camera_stack(
            UVec2::new(96, 54),
        ),
        ..RenderStats::default()
    };

    record(&mut store, &stats);

    assert_series(
        &store,
        "render.camera.target.graph_import.suppressed_by_camera_stack",
        1.0,
        "bool",
    );
    assert_series(
        &store,
        "render.camera.target.graph_import.direct_imported",
        0.0,
        "bool",
    );
    assert_series(
        &store,
        "render.camera.target.graph_import.direct_import_count",
        0.0,
        "count",
    );
    assert_series(
        &store,
        "render.camera.target.writeback.suppressed_by_camera_stack",
        1.0,
        "bool",
    );
    assert_series(
        &store,
        "render.camera.target.writeback.skipped_direct_import",
        0.0,
        "bool",
    );
    assert_series(
        &store,
        "render.camera.target.writeback.copy_count",
        0.0,
        "count",
    );
}

#[test]
fn render_product_diagnostics_record_capture_source_report() {
    let mut store = DiagnosticStore::default();
    let stats = RenderStats {
        submitted_frames: 12,
        last_capture_report: RenderCaptureReport::new(
            RenderCameraTargetKind::Texture,
            RenderCaptureSource::TextureWritebackConversion,
            UVec2::new(72, 40),
            crate::core::framework::render::RenderCameraTargetGraphImportStatus::RequiresConversionWriteback,
            crate::core::framework::render::RenderCameraTargetWritebackStatus::Converted,
        ),
        ..RenderStats::default()
    };

    record(&mut store, &stats);

    assert_series(
        &store,
        "render.capture.source.texture_writeback_conversion",
        1.0,
        "bool",
    );
    assert_series(
        &store,
        "render.capture.source.texture_direct_graph_import",
        0.0,
        "bool",
    );
    assert_series(&store, "render.capture.width", 72.0, "count");
    assert_series(&store, "render.capture.height", 40.0, "count");
}
