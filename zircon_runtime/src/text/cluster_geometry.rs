use crate::core::framework::text::TextGlyph;

use super::{ShapedGlyph, TextRange};

pub(crate) trait ClusterGeometryGlyph {
    fn cluster_source_range(&self) -> TextRange;
    fn cluster_advance(&self) -> f32;
    fn starts_cluster(&self) -> bool;
    fn is_right_to_left(&self) -> bool;
}

impl ClusterGeometryGlyph for ShapedGlyph {
    fn cluster_source_range(&self) -> TextRange {
        self.source_range
    }

    fn cluster_advance(&self) -> f32 {
        self.advance
    }

    fn starts_cluster(&self) -> bool {
        self.cluster_flags.cluster_start
    }

    fn is_right_to_left(&self) -> bool {
        self.cluster_flags.rtl
    }
}

impl ClusterGeometryGlyph for TextGlyph {
    fn cluster_source_range(&self) -> TextRange {
        TextRange {
            start: self.source_range.start,
            end: self.source_range.end,
        }
    }

    fn cluster_advance(&self) -> f32 {
        self.advance
    }

    fn starts_cluster(&self) -> bool {
        self.flags.cluster_start
    }

    fn is_right_to_left(&self) -> bool {
        self.flags.right_to_left
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct TextGlyphClusterGeometry {
    pub(crate) source_range: TextRange,
    pub(crate) advance: f32,
    pub(crate) glyph_start: usize,
    pub(crate) glyph_end: usize,
    /// `None` is a malformed mixed-direction cluster. Measurement can retain its bounded extent,
    /// but caret/hit projection must fail closed instead of choosing an arbitrary direction.
    pub(crate) right_to_left: Option<bool>,
}

pub(crate) struct TextGlyphClusters<'glyphs, G> {
    glyphs: &'glyphs [G],
    index: usize,
    backend_cluster_flags: bool,
}

pub(crate) fn text_glyph_clusters<G>(glyphs: &[G]) -> TextGlyphClusters<'_, G>
where
    G: ClusterGeometryGlyph,
{
    TextGlyphClusters {
        glyphs,
        index: 0,
        backend_cluster_flags: glyphs.iter().any(ClusterGeometryGlyph::starts_cluster),
    }
}

impl<G> Iterator for TextGlyphClusters<'_, G>
where
    G: ClusterGeometryGlyph,
{
    type Item = TextGlyphClusterGeometry;

    fn next(&mut self) -> Option<Self::Item> {
        let first = self.glyphs.get(self.index)?;
        let first_range = first.cluster_source_range();
        let mut source_range = first_range;
        let expected_rtl = first.is_right_to_left();
        let mut consistent_direction = true;
        let mut advance = 0.0;
        let start = self.index;

        while let Some(glyph) = self.glyphs.get(self.index) {
            let glyph_range = glyph.cluster_source_range();
            let starts_next_cluster = if self.backend_cluster_flags {
                glyph.starts_cluster()
            } else {
                glyph_range != first_range
            };
            if self.index > start && starts_next_cluster {
                break;
            }

            consistent_direction &= glyph.is_right_to_left() == expected_rtl;
            source_range.start = source_range.start.min(glyph_range.start);
            source_range.end = source_range.end.max(glyph_range.end);
            advance += finite_non_negative(glyph.cluster_advance());
            self.index += 1;
        }

        Some(TextGlyphClusterGeometry {
            source_range,
            advance,
            glyph_start: start,
            glyph_end: self.index,
            right_to_left: consistent_direction.then_some(expected_rtl),
        })
    }
}

fn finite_non_negative(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
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
}
