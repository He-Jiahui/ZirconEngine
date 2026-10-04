use crate::core::framework::text::TextGlyph;
use crate::text::layout_geometry::FiniteGeometryAccumulator;

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
    // 后端标记决定簇边界；无标记的兼容输入按相同源范围合并，使测量和光标共享簇划分。
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
        let mut advance = FiniteGeometryAccumulator::default();
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
            let glyph_advance = finite_non_negative(glyph.cluster_advance());
            advance.add(glyph_advance);
            self.index += 1;
        }

        Some(TextGlyphClusterGeometry {
            source_range,
            advance: advance.value(),
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
#[path = "tests/cluster_geometry.rs"]
mod tests;
