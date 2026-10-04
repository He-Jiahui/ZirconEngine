use std::path::Path;

use crate::text::font::{FontDatabase, SelectedFaceLineExtents};
use crate::text::shaping::direct_error::BackendGlyphInvariantKind;

use crate::text::shaping::horizontal::backend::{HorizontalBackendGlyph, HorizontalBackendRun};

use super::valid_backend_run;

#[test]
fn direct_backend_validation_reports_a_non_boundary_cluster_offset() {
    let run = HorizontalBackendRun {
        glyphs: vec![HorizontalBackendGlyph {
            glyph_id: 1,
            source_offset: 1,
            unsafe_to_break: false,
            advance: 1.0,
            x_offset: 0.0,
            y_offset: 0.0,
        }],
    };

    assert_eq!(
        valid_backend_run(&run, "é"),
        Err(BackendGlyphInvariantKind::InvalidClusterOffset)
    );
}

#[test]
fn direct_line_uses_scaled_selected_face_content_envelope() {
    let mut database = FontDatabase::default();
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts/FiraSans-Regular.ttf");
    let face = database
        .register_font_file(source, Some("Direct Metrics Face"), 0)
        .expect("register tracked font");
    let source_metrics = database
        .face_metrics(face)
        .expect("face metrics query")
        .expect("tracked face metrics");
    let mut extents = SelectedFaceLineExtents::default();
    let _ = extents.include_face(&database, face, 20.0);
    let envelope = extents
        .resolve_content_envelope(24.0)
        .expect("face metrics");
    let expected_ascent =
        f32::from(source_metrics.ascender.max(0)) * 20.0 / f32::from(source_metrics.units_per_em);

    assert!(envelope.baseline_from_top >= expected_ascent);
    assert!(envelope.line_height >= 24.0);
    assert!((envelope.baseline_from_top - 16.0).abs() > 0.01);
}
