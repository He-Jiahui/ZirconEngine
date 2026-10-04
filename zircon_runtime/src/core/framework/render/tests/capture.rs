use super::*;

#[test]
fn captured_frame_new_defaults_to_primary_framework_offscreen_source() {
    let frame = CapturedFrame::new(16, 8, vec![0; 16 * 8 * 4], 3);

    assert_eq!(
        frame.capture_report.target_kind,
        RenderCameraTargetKind::PrimarySurface
    );
    assert_eq!(
        frame.capture_report.source,
        RenderCaptureSource::FrameworkOffscreen
    );
    assert_eq!(frame.capture_report.output_size, UVec2::new(16, 8));
    assert_eq!(frame.graph_dump, None);
    assert_eq!(frame.frame_profile_json, None);
}

#[test]
fn texture_capture_report_distinguishes_direct_import_and_conversion_sources() {
    let size = UVec2::new(72, 40);

    let direct = RenderCaptureReport::texture_from_reports(
        size,
        RenderCameraTargetGraphImportReport::direct_imported(size),
        RenderCameraTargetWritebackReport::skipped_direct_import(size),
    );
    let converted = RenderCaptureReport::texture_from_reports(
        size,
        RenderCameraTargetGraphImportReport::requires_conversion_writeback(size),
        RenderCameraTargetWritebackReport::converted(size),
    );

    assert_eq!(direct.source, RenderCaptureSource::TextureDirectGraphImport);
    assert_eq!(
        direct.graph_import_status,
        RenderCameraTargetGraphImportStatus::DirectImported
    );
    assert_eq!(
        converted.source,
        RenderCaptureSource::TextureWritebackConversion
    );
    assert_eq!(
        converted.writeback_status,
        RenderCameraTargetWritebackStatus::Converted
    );
}

#[test]
fn capture_can_carry_profile_json_with_its_graph_dump() {
    let frame = CapturedFrame::with_capture_report_graph_dump_and_frame_profile_json(
        16,
        8,
        vec![0; 16 * 8 * 4],
        7,
        RenderCaptureReport::framework_offscreen(
            RenderCameraTargetKind::PrimarySurface,
            UVec2::new(16, 8),
        ),
        Some("MainScene".to_string()),
        Some("{\"frame_generation\":7}".to_string()),
    );

    assert_eq!(frame.graph_dump.as_deref(), Some("MainScene"));
    assert_eq!(
        frame.frame_profile_json.as_deref(),
        Some("{\"frame_generation\":7}")
    );
}

#[test]
fn captured_frame_clone_shares_graph_dump_text() {
    let graph_dump: Arc<str> = Arc::from("MainScene");
    let mut frame = CapturedFrame::new(16, 8, vec![0; 16 * 8 * 4], 7);
    frame.graph_dump = Some(Arc::clone(&graph_dump));

    let cloned = frame.clone();
    assert!(Arc::ptr_eq(
        frame.graph_dump.as_ref().expect("frame has graph dump"),
        cloned.graph_dump.as_ref().expect("clone has graph dump"),
    ));
}

#[test]
fn hdr_capture_preserves_linear_texels_and_capture_provenance() {
    let report = RenderCaptureReport::framework_offscreen(
        RenderCameraTargetKind::PrimarySurface,
        UVec2::new(2, 1),
    );
    let frame = CapturedHdrFrame::with_capture_report(
        2,
        1,
        vec![[1.5, 0.5, 0.25, 1.0], [0.0, 0.0, 0.0, 1.0]],
        7,
        report,
    );

    assert_eq!(frame.rgba16f.len(), 2);
    assert_eq!(frame.generation, 7);
    assert_eq!(frame.capture_report, report);
}
