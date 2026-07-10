use zircon_runtime_interface::ui::surface::{UiTextDirection, UiTextRange};

use crate::core::framework::render::{
    FontFaceId, ShapedGlyph, ShapedGlyphRotation, ShapedGlyphRun, TextOrientation, TextShapeRequest,
};
use crate::graphics::text::font::FontDatabase;

use super::backend::{shape_vertical_run, VerticalBackendDirection, VerticalBackendRun};
use super::{source_cluster_text, vertical_glyph_rotation};

#[derive(Clone, Copy)]
struct SourceCluster {
    glyph_start: usize,
    glyph_end: usize,
    source_range: UiTextRange,
    face: Option<FontFaceId>,
    direction: UiTextDirection,
    upright: bool,
}

pub(super) fn apply_vertical_backend_shaping(
    shaped: &mut ShapedGlyphRun,
    request: TextShapeRequest<'_>,
    font_database: &FontDatabase,
) -> Vec<Vec<Option<f32>>> {
    if !matches!(request.orientation, TextOrientation::Vertical) {
        return shaped
            .lines
            .iter()
            .map(|line| vec![None; line.glyphs.len()])
            .collect();
    }

    let mut all_advances = Vec::with_capacity(shaped.lines.len());
    for line in &mut shaped.lines {
        let source_glyphs = std::mem::take(&mut line.glyphs);
        let clusters = source_clusters(&source_glyphs, request);
        let mut projected = Vec::with_capacity(source_glyphs.len());
        let mut advances = Vec::with_capacity(source_glyphs.len());
        let mut cluster_index = 0_usize;
        while cluster_index < clusters.len() {
            let segment_end = backend_segment_end(&clusters, cluster_index);
            let segment = &clusters[cluster_index..segment_end];
            let glyph_start = segment[0].glyph_start;
            let glyph_end = segment[segment.len() - 1].glyph_end;
            let source_range = segment_source_range(segment);
            let segment_text = source_cluster_text(request, source_range);
            let backend_run = segment[0]
                .upright
                .then_some(segment[0].face)
                .flatten()
                .and_then(|face| {
                    shape_vertical_run(
                        font_database,
                        face,
                        segment_text,
                        vertical_backend_direction(segment[0].direction),
                        request.language,
                        request.features,
                        request.include_kerning,
                        request.style.font_size,
                    )
                })
                .filter(|run| valid_backend_run(run, segment_text));

            if let Some(backend_run) = backend_run {
                if let Some((glyphs, run_advances)) = project_backend_run(
                    &source_glyphs[glyph_start..glyph_end],
                    source_range,
                    segment_text,
                    backend_run,
                ) {
                    projected.extend(glyphs);
                    advances.extend(run_advances);
                } else {
                    projected.extend_from_slice(&source_glyphs[glyph_start..glyph_end]);
                    advances.resize(projected.len(), None);
                }
            } else {
                projected.extend_from_slice(&source_glyphs[glyph_start..glyph_end]);
                advances.resize(projected.len(), None);
            }
            cluster_index = segment_end;
        }
        line.glyphs = projected;
        all_advances.push(advances);
    }
    all_advances
}

pub(super) fn vertical_backend_direction(direction: UiTextDirection) -> VerticalBackendDirection {
    if matches!(direction, UiTextDirection::RightToLeft) {
        VerticalBackendDirection::BottomToTop
    } else {
        VerticalBackendDirection::TopToBottom
    }
}

fn source_clusters(glyphs: &[ShapedGlyph], request: TextShapeRequest<'_>) -> Vec<SourceCluster> {
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
        let one_face = face.is_some() && cluster.iter().all(|glyph| glyph.font_id == face);
        let upright = matches!(
            vertical_glyph_rotation(
                request.vertical_mode,
                source_cluster_text(request, source_range),
            ),
            ShapedGlyphRotation::None
        );
        clusters.push(SourceCluster {
            glyph_start,
            glyph_end,
            source_range,
            face: one_face.then_some(face).flatten(),
            direction: cluster[0].direction,
            upright,
        });
        glyph_start = glyph_end;
    }
    clusters
}

fn backend_segment_end(clusters: &[SourceCluster], start: usize) -> usize {
    let first = clusters[start];
    if !first.upright || first.face.is_none() {
        return start + 1;
    }
    let mut end = start + 1;
    while end < clusters.len() {
        let previous = clusters[end - 1];
        let next = clusters[end];
        let adjacent = previous.source_range.end == next.source_range.start
            || next.source_range.end == previous.source_range.start;
        if !next.upright
            || next.face != first.face
            || next.direction != first.direction
            || !adjacent
        {
            break;
        }
        end += 1;
    }
    end
}

fn segment_source_range(segment: &[SourceCluster]) -> UiTextRange {
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

fn valid_backend_run(run: &VerticalBackendRun, text: &str) -> bool {
    !run.glyphs.is_empty()
        && run.glyphs.iter().all(|glyph| {
            glyph.source_offset < text.len()
                && text.is_char_boundary(glyph.source_offset)
                && glyph.y_advance.is_finite()
                && glyph.x_offset.is_finite()
                && glyph.y_offset.is_finite()
        })
}

fn project_backend_run(
    source_glyphs: &[ShapedGlyph],
    source_range: UiTextRange,
    text: &str,
    backend_run: VerticalBackendRun,
) -> Option<(Vec<ShapedGlyph>, Vec<Option<f32>>)> {
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
    let mut advances = Vec::with_capacity(backend_run.glyphs.len());
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
        glyph.advance = backend.y_advance.abs();
        glyph.offset_x = backend.x_offset;
        glyph.offset_y = -backend.y_offset;
        glyph.cluster_flags.cluster_start = previous_offset != Some(backend.source_offset);
        previous_offset = Some(backend.source_offset);
        advances.push(Some(glyph.advance));
        glyphs.push(glyph);
    }
    Some((glyphs, advances))
}

fn ranges_overlap(lhs: UiTextRange, rhs: UiTextRange) -> bool {
    lhs.start < rhs.end && rhs.start < lhs.end
}
