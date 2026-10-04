use super::*;

fn vertex(seed: f32) -> ScreenSpaceUiSdfVertex {
    ScreenSpaceUiSdfVertex {
        position: [seed, 0.0, 0.0, 1.0],
        uv: [0.0; 2],
        color: [1.0; 4],
        screen_px_range: 1.0,
        atlas_px_range: 1.0,
        page_index: 0,
        decode_mode: 0,
        primitive_kind: 0,
    }
}

fn product(seed: f32) -> SdfCompiledTextSegmentProduct {
    SdfCompiledTextSegmentProduct {
        native_decoration_vertices: vec![vertex(seed)],
        sdf_decoration_vertices: vec![vertex(seed + 1.0)],
        glyph_vertices: vec![vertex(seed + 2.0), vertex(seed + 3.0)],
        draw_plan: SdfTextMaterialDrawPlan {
            materials: vec![SdfTextMaterial {
                fill_color: [seed; 4],
                ..Default::default()
            }],
            draws: vec![SdfTextMaterialDraw {
                vertices: 0..2,
                material_index: 0,
            }],
        },
    }
}

#[test]
fn same_cardinality_patch_preserves_unrelated_segment_products() {
    let mut index = SdfCompiledTextSegmentIndex::default();
    let mut vertices = Vec::new();
    let mut draw_plan = SdfTextMaterialDrawPlan::default();
    index.publish_products(
        vec![product(1.0), product(10.0), product(20.0)],
        &mut vertices,
        &mut draw_plan,
    );
    let before = vertices.clone();

    let report = index
        .apply_replacements(vec![(1, product(40.0))], &mut vertices, &mut draw_plan)
        .expect("same-cardinality segment patch must remain local");

    for range in [
        index.native_vertex_ranges[0].clone(),
        index.sdf_decoration_vertex_ranges[0].clone(),
        index.glyph_vertex_ranges[0].clone(),
        index.native_vertex_ranges[2].clone(),
        index.sdf_decoration_vertex_ranges[2].clone(),
        index.glyph_vertex_ranges[2].clone(),
    ] {
        assert_eq!(&vertices[range.clone()], &before[range]);
    }
    assert_eq!(report.vertex_visit_count, 4);
    assert_eq!(report.material_visit_count, 1);
    assert_eq!(report.changed_vertex_ranges.len(), 3);
}

#[test]
fn cardinality_change_rejects_local_patch_before_mutating_output() {
    let mut index = SdfCompiledTextSegmentIndex::default();
    let mut vertices = Vec::new();
    let mut draw_plan = SdfTextMaterialDrawPlan::default();
    index.publish_products(vec![product(1.0)], &mut vertices, &mut draw_plan);
    let before = vertices.clone();
    let mut replacement = product(10.0);
    replacement.glyph_vertices.push(vertex(99.0));

    assert!(index
        .apply_replacements(vec![(0, replacement)], &mut vertices, &mut draw_plan)
        .is_none());
    assert_eq!(vertices, before);
}

#[test]
fn local_frame_patch_source_does_not_scan_every_segment() {
    let source = include_str!("../segment_product.rs");
    let local = source
        .split("fn patch_retained_frame")
        .nth(1)
        .and_then(|source| source.split("fn rebuild").next())
        .expect("local patch body");
    assert!(!local.contains("frame.segment_products().iter()"));
    assert!(!local.contains("for segment in frame.segment_products()"));
}
