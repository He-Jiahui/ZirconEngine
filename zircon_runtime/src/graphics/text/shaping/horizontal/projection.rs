use zircon_runtime_interface::ui::surface::{UiResolvedStyle, UiTextDirection, UiTextRange};

use crate::core::framework::render::{
    FontFaceId, InstancedFaceId, ShapedGlyph, ShapedGlyphRun, TextOrientation, TextShapeRequest,
};
use crate::graphics::text::font::FontDatabase;

use super::backend::{shape_horizontal_run, HorizontalBackendRun};
use crate::graphics::text::shaping::vertical::source_cluster_text;

#[derive(Clone, Copy)]
struct SourceCluster<'a> {
    glyph_start: usize,
    glyph_end: usize,
    source_range: UiTextRange,
    face: Option<FontFaceId>,
    instance: Option<InstancedFaceId>,
    direction: UiTextDirection,
    /// Resolved ISO15924 tag; backend segments must never cross this boundary.
    script: &'a str,
}

pub(in crate::graphics::text::shaping) fn apply_horizontal_backend_shaping(
    shaped: &mut ShapedGlyphRun,
    request: TextShapeRequest<'_>,
    font_database: &FontDatabase,
) {
    if !should_apply_horizontal_backend(request.orientation) {
        return;
    }
    for line in &mut shaped.lines {
        let source_glyphs = std::mem::take(&mut line.glyphs);
        let clusters = source_clusters(&source_glyphs);
        let mut projected = Vec::with_capacity(source_glyphs.len());
        let mut cluster_index = 0_usize;
        while cluster_index < clusters.len() {
            let segment_end = backend_segment_end(&clusters, cluster_index);
            let segment = &clusters[cluster_index..segment_end];
            let glyph_start = segment[0].glyph_start;
            let glyph_end = segment[segment.len() - 1].glyph_end;
            let source_range = segment_source_range(segment);
            let segment_text = source_cluster_text(request, source_range);
            let backend_run = segment[0].face.and_then(|face| {
                shape_horizontal_run(
                    font_database,
                    face,
                    segment[0].instance,
                    segment_text,
                    segment[0].direction,
                    segment[0].script,
                    request.language,
                    request.features,
                    request.include_kerning,
                    UiResolvedStyle::normalized_font_weight(request.style.font_weight),
                    request.style.font_size,
                )
            });
            if let Some(backend_run) =
                backend_run.filter(|run| valid_backend_run(run, segment_text))
            {
                if let Some(glyphs) = project_backend_run(
                    &source_glyphs[glyph_start..glyph_end],
                    source_range,
                    segment_text,
                    segment[0].instance,
                    backend_run,
                ) {
                    projected.extend(glyphs);
                } else {
                    projected.extend_from_slice(&source_glyphs[glyph_start..glyph_end]);
                }
            } else {
                projected.extend_from_slice(&source_glyphs[glyph_start..glyph_end]);
            }
            cluster_index = segment_end;
        }

        let mut cursor = 0.0_f32;
        for glyph in &mut projected {
            glyph.x = cursor;
            cursor += glyph.advance.max(0.0);
        }
        line.measured_width = cursor;
        line.glyphs = projected;
    }
    shaped.measured_width = shaped
        .lines
        .iter()
        .map(|line| line.measured_width)
        .fold(0.0_f32, f32::max);
}

pub(super) const fn should_apply_horizontal_backend(orientation: TextOrientation) -> bool {
    matches!(orientation, TextOrientation::Horizontal)
}

fn source_clusters(glyphs: &[ShapedGlyph]) -> Vec<SourceCluster<'_>> {
    let mut clusters = Vec::new();
    let mut glyph_start = 0_usize;
    while glyph_start < glyphs.len() {
        let source_range = glyphs[glyph_start].source_range;
        let mut glyph_end = glyph_start + 1;
        while glyph_end < glyphs.len() && glyphs[glyph_end].source_range == source_range {
            glyph_end += 1;
        }
        let cluster = &glyphs[glyph_start..glyph_end];
        let face = cluster[0].font_id;
        let instance = cluster[0].font_instance_id;
        let one_identity = face.is_some()
            && cluster
                .iter()
                .all(|glyph| glyph.font_id == face && glyph.font_instance_id == instance);
        clusters.push(SourceCluster {
            glyph_start,
            glyph_end,
            source_range,
            face: one_identity.then_some(face).flatten(),
            instance: one_identity.then_some(instance).flatten(),
            direction: cluster[0].direction,
            script: cluster[0].script.iso15924.as_str(),
        });
        glyph_start = glyph_end;
    }
    clusters
}

fn backend_segment_end(clusters: &[SourceCluster<'_>], start: usize) -> usize {
    let first = clusters[start];
    let Some(face) = first.face else {
        return start + 1;
    };
    let mut end = start + 1;
    while end < clusters.len() {
        let previous = clusters[end - 1];
        let next = clusters[end];
        let adjacent = previous.source_range.end == next.source_range.start
            || next.source_range.end == previous.source_range.start;
        if next.face != Some(face)
            || next.instance != first.instance
            || next.direction != first.direction
            || next.script != first.script
            || !adjacent
        {
            break;
        }
        end += 1;
    }
    end
}

fn segment_source_range(segment: &[SourceCluster<'_>]) -> UiTextRange {
    UiTextRange {
        start: segment
            .iter()
            .map(|cluster| cluster.source_range.start)
            .min()
            .unwrap_or(0),
        end: segment
            .iter()
            .map(|cluster| cluster.source_range.end)
            .max()
            .unwrap_or(0),
    }
}

fn valid_backend_run(run: &HorizontalBackendRun, text: &str) -> bool {
    !run.glyphs.is_empty()
        && run.glyphs.iter().all(|glyph| {
            glyph.source_offset < text.len()
                && text.is_char_boundary(glyph.source_offset)
                && glyph.advance.is_finite()
                && glyph.x_offset.is_finite()
                && glyph.y_offset.is_finite()
        })
}

fn project_backend_run(
    source_glyphs: &[ShapedGlyph],
    source_range: UiTextRange,
    text: &str,
    instance: Option<InstancedFaceId>,
    backend_run: HorizontalBackendRun,
) -> Option<Vec<ShapedGlyph>> {
    let mut boundaries = backend_run
        .glyphs
        .iter()
        .map(|glyph| glyph.source_offset)
        .collect::<Vec<_>>();
    boundaries.push(text.len());
    boundaries.sort_unstable();
    boundaries.dedup();

    let mut previous_offset = None;
    let mut glyphs = Vec::with_capacity(backend_run.glyphs.len());
    for backend in backend_run.glyphs {
        let cluster_end = boundaries
            .iter()
            .copied()
            .find(|boundary| *boundary > backend.source_offset)?;
        let projected_range = UiTextRange {
            start: source_range.start + backend.source_offset,
            end: source_range.start + cluster_end,
        };
        let overlapping = source_glyphs
            .iter()
            .filter(|glyph| ranges_overlap(glyph.source_range, projected_range))
            .collect::<Vec<_>>();
        let template = overlapping.first().copied()?;
        let mut glyph = template.clone();
        glyph.glyph_id = backend.glyph_id;
        glyph.font_instance_id = instance;
        glyph.source_range = projected_range;
        glyph.visual_range = UiTextRange {
            start: overlapping
                .iter()
                .map(|glyph| glyph.visual_range.start)
                .min()
                .unwrap_or(template.visual_range.start),
            end: overlapping
                .iter()
                .map(|glyph| glyph.visual_range.end)
                .max()
                .unwrap_or(template.visual_range.end),
        };
        glyph.advance = backend.advance.abs();
        glyph.offset_x = backend.x_offset;
        glyph.offset_y = -backend.y_offset;
        glyph.cluster_flags.cluster_start = previous_offset != Some(backend.source_offset);
        previous_offset = Some(backend.source_offset);
        glyphs.push(glyph);
    }
    Some(glyphs)
}

fn ranges_overlap(lhs: UiTextRange, rhs: UiTextRange) -> bool {
    lhs.start < rhs.end && rhs.start < lhs.end
}
