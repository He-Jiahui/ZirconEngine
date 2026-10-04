use super::*;

#[derive(Clone, Copy)]
struct Glyph {
    range: TextRange,
    advance: f32,
    start: bool,
    rtl: bool,
}

impl ClusterGeometryGlyph for Glyph {
    fn cluster_source_range(&self) -> TextRange {
        self.range
    }

    fn cluster_advance(&self) -> f32 {
        self.advance
    }

    fn starts_cluster(&self) -> bool {
        self.start
    }

    fn is_right_to_left(&self) -> bool {
        self.rtl
    }
}

#[test]
fn backend_cluster_flags_merge_multiglyph_geometry_once() {
    let glyphs = [
        Glyph {
            range: TextRange { start: 0, end: 1 },
            advance: 10.0,
            start: true,
            rtl: false,
        },
        Glyph {
            range: TextRange { start: 1, end: 2 },
            advance: 20.0,
            start: false,
            rtl: false,
        },
    ];

    assert_eq!(
        text_glyph_clusters(&glyphs).collect::<Vec<_>>(),
        vec![TextGlyphClusterGeometry {
            source_range: TextRange { start: 0, end: 2 },
            advance: 30.0,
            glyph_start: 0,
            glyph_end: 2,
            right_to_left: Some(false),
        }]
    );
}

#[test]
fn legacy_geometry_groups_only_identical_source_ranges() {
    let glyphs = [
        Glyph {
            range: TextRange { start: 0, end: 2 },
            advance: 10.0,
            start: false,
            rtl: false,
        },
        Glyph {
            range: TextRange { start: 0, end: 2 },
            advance: 20.0,
            start: false,
            rtl: false,
        },
        Glyph {
            range: TextRange { start: 2, end: 3 },
            advance: 5.0,
            start: false,
            rtl: false,
        },
    ];

    let clusters = text_glyph_clusters(&glyphs).collect::<Vec<_>>();
    assert_eq!(clusters.len(), 2);
    assert_eq!(clusters[0].advance, 30.0);
    assert_eq!(clusters[1].source_range, TextRange { start: 2, end: 3 });
}

#[test]
fn mixed_direction_cluster_is_explicitly_unusable_for_caret_projection() {
    let glyphs = [
        Glyph {
            range: TextRange { start: 0, end: 1 },
            advance: 10.0,
            start: true,
            rtl: false,
        },
        Glyph {
            range: TextRange { start: 1, end: 2 },
            advance: 20.0,
            start: false,
            rtl: true,
        },
    ];

    assert_eq!(
        text_glyph_clusters(&glyphs)
            .next()
            .and_then(|cluster| cluster.right_to_left),
        None
    );
}

#[test]
fn multiglyph_cluster_advance_stays_finite_when_f32_accumulation_overflows() {
    let glyphs = [
        Glyph {
            range: TextRange { start: 0, end: 1 },
            advance: f32::MAX,
            start: true,
            rtl: false,
        },
        Glyph {
            range: TextRange { start: 0, end: 1 },
            advance: f32::MAX,
            start: false,
            rtl: false,
        },
    ];

    let cluster = text_glyph_clusters(&glyphs)
        .next()
        .expect("one shared cluster");

    assert_eq!(cluster.advance, f32::MAX);
    assert!(cluster.advance.is_finite());
}
