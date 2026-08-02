use std::sync::Arc;

use unicode_segmentation::UnicodeSegmentation;
use zircon_runtime_interface::ui::surface::{
    UiResolvedStyle, UiResolvedTextLayout, UiResolvedTextLine, UiRichTextArtifactHandle,
    UiTextRange, UiTextWritingMode,
};

use super::font::shared_font_database_generation;
use super::service::project_shaped_glyph_run_for_runtime;
use super::{SharedTextLayoutSession, TextRange, VerticalMode, text_style};
use crate::core::framework::text::{TextGlyph, TextShapeResult};

#[derive(Clone, Debug)]
pub(crate) struct ResolvedTextGlyphArtifact {
    pub(crate) source_text: Arc<str>,
    pub(crate) source_text_origin: usize,
    pub(crate) font_generation: u64,
    pub(crate) style: UiResolvedStyle,
    pub(crate) writing_mode: UiTextWritingMode,
    pub(crate) lines: Vec<Option<ResolvedTextGlyphArtifactLine>>,
}

#[derive(Clone, Debug)]
pub(crate) struct ResolvedTextGlyphArtifactLine {
    pub(crate) glyphs: Vec<TextGlyph>,
    pub(crate) layout_line: UiResolvedTextLine,
}

pub(crate) fn register_resolved_text_glyph_artifact(
    artifact: Arc<ResolvedTextGlyphArtifact>,
) -> UiRichTextArtifactHandle {
    UiRichTextArtifactHandle::from_runtime_artifact(artifact)
}

pub(crate) fn resolve_resolved_text_glyph_artifact(
    handle: &UiRichTextArtifactHandle,
) -> Option<Arc<ResolvedTextGlyphArtifact>> {
    handle.downcast_runtime_artifact()
}

pub(crate) fn build_resolved_text_glyph_artifact(
    source_text: &str,
    style: &UiResolvedStyle,
    layout: &UiResolvedTextLayout,
    provider: &mut SharedTextLayoutSession,
) -> Option<ResolvedTextGlyphArtifact> {
    let source_text_origin = source_text_origin(source_text, layout.source_range);
    let shaped_style = text_style(&UiResolvedStyle {
        font_size: layout.font_size,
        line_height: layout.line_height,
        ..style.clone()
    });
    let artifact_style = UiResolvedStyle {
        font_size: layout.font_size,
        line_height: layout.line_height,
        ..style.clone()
    };
    let lines = layout
        .lines
        .iter()
        .map(|line| {
            if line.ellipsized {
                return None;
            }
            let projected = shape_line_for_artifact(
                source_text,
                source_text_origin,
                &shaped_style,
                layout.writing_mode,
                line,
                provider,
            )?;
            Some(ResolvedTextGlyphArtifactLine {
                glyphs: visual_glyphs_for_line(source_text, source_text_origin, line, projected),
                layout_line: line.clone(),
            })
        })
        .collect::<Vec<_>>();
    lines
        .iter()
        .any(Option::is_some)
        .then(|| ResolvedTextGlyphArtifact {
            source_text: Arc::from(source_text),
            source_text_origin,
            font_generation: shared_font_database_generation(),
            style: artifact_style,
            writing_mode: layout.writing_mode,
            lines,
        })
}

pub(crate) fn rebuild_resolved_text_glyph_artifact_line(
    artifact: &ResolvedTextGlyphArtifact,
    line_index: usize,
) -> Option<Arc<ResolvedTextGlyphArtifactLine>> {
    let line = artifact
        .lines
        .get(line_index)?
        .as_ref()?
        .layout_line
        .clone();
    let mut provider = SharedTextLayoutSession::new();
    let shaped_style = text_style(&artifact.style);
    let projected = shape_line_for_artifact(
        artifact.source_text.as_ref(),
        artifact.source_text_origin,
        &shaped_style,
        artifact.writing_mode,
        &line,
        &mut provider,
    )?;
    Some(Arc::new(ResolvedTextGlyphArtifactLine {
        glyphs: visual_glyphs_for_line(
            artifact.source_text.as_ref(),
            artifact.source_text_origin,
            &line,
            projected,
        ),
        layout_line: line,
    }))
}

fn shape_line_for_artifact(
    source_text: &str,
    source_text_origin: usize,
    style: &crate::text::TextStyle,
    writing_mode: UiTextWritingMode,
    line: &UiResolvedTextLine,
    provider: &mut SharedTextLayoutSession,
) -> Option<TextShapeResult> {
    let source = source_slice(source_text, source_text_origin, line.source_range)?;
    let shaped = if matches!(writing_mode, UiTextWritingMode::VerticalRl) {
        provider.shape_vertical_line(
            source,
            style,
            line.direction.into(),
            TextRange {
                start: line.source_range.start,
                end: line.source_range.end,
            },
            VerticalMode::Mixed,
        )
    } else {
        provider.shape_horizontal_line(
            source,
            style,
            line.direction.into(),
            TextRange {
                start: line.source_range.start,
                end: line.source_range.end,
            },
        )
    };
    Some(project_shaped_glyph_run_for_runtime(shaped.as_ref()))
}

fn visual_glyphs_for_line(
    source_text: &str,
    source_text_origin: usize,
    line: &UiResolvedTextLine,
    shaped: TextShapeResult,
) -> Vec<TextGlyph> {
    let visual_clusters = visual_clusters_for_line(source_text, source_text_origin, line);
    let mut glyphs = shaped
        .runs
        .into_iter()
        .flat_map(|run| run.glyphs)
        .collect::<Vec<_>>();
    if visual_clusters.is_empty() {
        return glyphs;
    }

    let mut source_order = visual_clusters.clone();
    // Direct shaping is logical-order; resolve visual ranks once, then sort while retaining
    // the backend order of glyphs that share a cluster.
    source_order.sort_by(|left, right| {
        left.source_range
            .start
            .cmp(&right.source_range.start)
            .then_with(|| left.source_range.end.cmp(&right.source_range.end))
            .then_with(|| left.visual_index.cmp(&right.visual_index))
    });
    let mut projected = glyphs
        .drain(..)
        .enumerate()
        .map(|(source_index, glyph)| {
            let source_clusters = source_cluster_range_for_glyph(&source_order, &glyph);
            let visual_index = source_order[source_clusters.clone()]
                .iter()
                .map(|cluster| cluster.visual_index)
                .min()
                .unwrap_or(usize::MAX);
            ProjectedGlyph {
                glyph,
                source_index,
                visual_index,
                source_clusters,
            }
        })
        .collect::<Vec<_>>();
    projected.sort_by(|left, right| {
        left.visual_index
            .cmp(&right.visual_index)
            .then_with(|| left.source_index.cmp(&right.source_index))
    });
    apply_resolved_advances(
        &mut projected,
        source_order.as_slice(),
        line.glyph_advances.as_slice(),
        visual_clusters.len(),
    );
    projected.into_iter().map(|glyph| glyph.glyph).collect()
}

#[derive(Clone, Copy)]
struct VisualCluster {
    source_range: UiTextRange,
    visual_index: usize,
}

struct ProjectedGlyph {
    glyph: TextGlyph,
    source_index: usize,
    visual_index: usize,
    source_clusters: std::ops::Range<usize>,
}

fn visual_clusters_for_line(
    source_text: &str,
    source_text_origin: usize,
    line: &UiResolvedTextLine,
) -> Vec<VisualCluster> {
    if line.runs.is_empty() {
        let mut source_graphemes = source_slice(source_text, source_text_origin, line.source_range)
            .map(|source| {
                source
                    .grapheme_indices(true)
                    .map(|(start, grapheme)| UiTextRange {
                        start: line.source_range.start + start,
                        end: line.source_range.start + start + grapheme.len(),
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        if matches!(
            line.direction,
            zircon_runtime_interface::ui::surface::UiTextDirection::RightToLeft
        ) {
            source_graphemes.reverse();
        }
        return line
            .text
            .grapheme_indices(true)
            .enumerate()
            .map(|(visual_index, _)| VisualCluster {
                source_range: source_graphemes
                    .get(visual_index)
                    .copied()
                    .unwrap_or(line.source_range),
                visual_index,
            })
            .collect();
    }
    let run_maps = line
        .runs
        .iter()
        .map(|run| RunSourceMap::new(source_text, source_text_origin, run))
        .collect::<Vec<_>>();
    let mut first_run = 0_usize;
    line.text
        .grapheme_indices(true)
        .enumerate()
        .map(|(visual_index, (start, grapheme))| {
            let visual_range = UiTextRange {
                start: line.visual_range.start + start,
                end: line.visual_range.start + start + grapheme.len(),
            };
            while run_maps
                .get(first_run)
                .is_some_and(|run| run.visual_range.end <= visual_range.start)
            {
                first_run += 1;
            }
            let mut source_range = None;
            for run in run_maps[first_run..]
                .iter()
                .take_while(|run| run.visual_range.start < visual_range.end)
            {
                if let Some(range) = run.source_range_for_visual(visual_range) {
                    source_range = Some(merge_ranges(source_range, range));
                }
            }
            VisualCluster {
                source_range: source_range.unwrap_or(line.source_range),
                visual_index,
            }
        })
        .collect()
}

struct RunSourceMap {
    visual_range: UiTextRange,
    visual_graphemes: Vec<UiTextRange>,
    source_graphemes: Vec<UiTextRange>,
}

impl RunSourceMap {
    fn new(
        source_text: &str,
        source_text_origin: usize,
        run: &zircon_runtime_interface::ui::surface::UiResolvedTextRun,
    ) -> Self {
        let visual_graphemes = run
            .text
            .grapheme_indices(true)
            .map(|(start, grapheme)| UiTextRange {
                start: run.visual_range.start + start,
                end: run.visual_range.start + start + grapheme.len(),
            })
            .collect::<Vec<_>>();
        let mut source_graphemes = source_slice(source_text, source_text_origin, run.source_range)
            .map(|source| {
                source
                    .grapheme_indices(true)
                    .map(|(start, grapheme)| UiTextRange {
                        start: run.source_range.start + start,
                        end: run.source_range.start + start + grapheme.len(),
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        if source_graphemes.len() != visual_graphemes.len() {
            source_graphemes = vec![run.source_range; visual_graphemes.len()];
        } else if matches!(
            run.direction,
            zircon_runtime_interface::ui::surface::UiTextDirection::RightToLeft
        ) {
            source_graphemes.reverse();
        }
        Self {
            visual_range: run.visual_range,
            visual_graphemes,
            source_graphemes,
        }
    }

    fn source_range_for_visual(&self, visual_range: UiTextRange) -> Option<UiTextRange> {
        let index = self
            .visual_graphemes
            .partition_point(|range| range.end <= visual_range.start);
        self.visual_graphemes
            .get(index)
            .filter(|range| ranges_overlap(**range, visual_range))?;
        self.source_graphemes.get(index).copied()
    }
}

fn source_cluster_range_for_glyph(
    source_order: &[VisualCluster],
    glyph: &TextGlyph,
) -> std::ops::Range<usize> {
    let start = source_order
        .partition_point(|cluster| cluster.source_range.end <= glyph.source_range.start);
    let end =
        source_order.partition_point(|cluster| cluster.source_range.start < glyph.source_range.end);
    start..end
}

fn apply_resolved_advances(
    glyphs: &mut [ProjectedGlyph],
    source_order: &[VisualCluster],
    advances: &[f32],
    cluster_count: usize,
) {
    if advances.len() != cluster_count {
        return;
    }
    for glyph in glyphs
        .iter_mut()
        .filter(|glyph| !glyph.source_clusters.is_empty())
    {
        glyph.glyph.advance = 0.0;
    }
    let mut first_glyph_by_cluster = vec![None; cluster_count];
    for (glyph_index, glyph) in glyphs.iter().enumerate() {
        for cluster in &source_order[glyph.source_clusters.clone()] {
            first_glyph_by_cluster[cluster.visual_index].get_or_insert(glyph_index);
        }
    }
    for (cluster_index, advance) in advances.iter().copied().enumerate() {
        let Some(glyph_index) = first_glyph_by_cluster[cluster_index] else {
            continue;
        };
        if advance.is_finite() {
            glyphs[glyph_index].glyph.advance += advance.max(0.0);
        }
    }
}

fn merge_ranges(current: Option<UiTextRange>, next: UiTextRange) -> UiTextRange {
    let Some(current) = current else {
        return next;
    };
    UiTextRange {
        start: current.start.min(next.start),
        end: current.end.max(next.end),
    }
}

fn ranges_overlap(left: UiTextRange, right: UiTextRange) -> bool {
    left.start < right.end && right.start < left.end
}

fn source_text_origin(source_text: &str, layout_source_range: UiTextRange) -> usize {
    (source_text.len()
        == layout_source_range
            .end
            .saturating_sub(layout_source_range.start))
    .then_some(layout_source_range.start)
    .unwrap_or_default()
}

fn source_slice(
    source_text: &str,
    source_text_origin: usize,
    source_range: UiTextRange,
) -> Option<&str> {
    let start = source_range.start.checked_sub(source_text_origin)?;
    let end = source_range.end.checked_sub(source_text_origin)?;
    source_text.get(start..end)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::framework::text::{
        TextGlyphFlags, TextGlyphRotation, TextShapeResult, TextShapeRun,
    };
    use zircon_runtime_interface::ui::layout::UiFrame;
    use zircon_runtime_interface::ui::surface::{UiTextDirection, UiTextRunKind};

    #[test]
    fn visual_glyph_artifact_keeps_contextual_arabic_glyphs_in_visual_order() {
        let line = UiResolvedTextLine {
            text: "مالس".to_string(),
            frame: UiFrame::new(0.0, 0.0, 40.0, 12.0),
            source_range: UiTextRange { start: 0, end: 8 },
            visual_range: UiTextRange { start: 0, end: 8 },
            measured_width: 40.0,
            glyph_advances: vec![10.0; 4],
            baseline: 9.0,
            direction: UiTextDirection::RightToLeft,
            runs: vec![
                visual_run("م", 6, 8, 0, 2),
                visual_run("ا", 4, 6, 2, 4),
                visual_run("ل", 2, 4, 4, 6),
                visual_run("س", 0, 2, 6, 8),
            ],
            ellipsized: false,
        };

        let glyphs = visual_glyphs_for_line(
            "سلام",
            0,
            &line,
            TextShapeResult {
                runs: vec![TextShapeRun {
                    source_range: 0..8,
                    direction: crate::core::framework::text::TextDirection::RightToLeft,
                    glyphs: vec![
                        glyph(101, 0..2),
                        glyph(102, 2..4),
                        glyph(103, 4..6),
                        glyph(104, 6..8),
                    ],
                }],
                metrics: Default::default(),
                resolved_direction: crate::core::framework::text::TextDirection::RightToLeft,
            },
        );

        assert_eq!(
            glyphs
                .iter()
                .map(|glyph| glyph.glyph_id)
                .collect::<Vec<_>>(),
            vec![104, 103, 102, 101]
        );
    }

    #[test]
    fn visual_glyph_artifact_projects_resolved_advance_to_an_unsplit_ligature() {
        let line = UiResolvedTextLine {
            text: "fi".to_string(),
            frame: UiFrame::new(0.0, 0.0, 30.0, 12.0),
            source_range: UiTextRange { start: 0, end: 2 },
            visual_range: UiTextRange { start: 0, end: 2 },
            measured_width: 30.0,
            glyph_advances: vec![12.0, 18.0],
            baseline: 9.0,
            direction: UiTextDirection::LeftToRight,
            runs: vec![zircon_runtime_interface::ui::surface::UiResolvedTextRun {
                kind: UiTextRunKind::Plain,
                text: "fi".to_string(),
                source_range: UiTextRange { start: 0, end: 2 },
                visual_range: UiTextRange { start: 0, end: 2 },
                direction: UiTextDirection::LeftToRight,
            }],
            ellipsized: false,
        };

        let glyphs = visual_glyphs_for_line(
            "fi",
            0,
            &line,
            TextShapeResult {
                runs: vec![TextShapeRun {
                    source_range: 0..2,
                    direction: crate::core::framework::text::TextDirection::LeftToRight,
                    glyphs: vec![glyph(77, 0..2)],
                }],
                metrics: Default::default(),
                resolved_direction: crate::core::framework::text::TextDirection::LeftToRight,
            },
        );

        assert_eq!(glyphs.len(), 1);
        assert_eq!(glyphs[0].advance, 30.0);
    }

    #[test]
    fn visual_glyph_artifact_preserves_tab_and_justified_space_advances() {
        let line = UiResolvedTextLine {
            text: "a\tb c".to_string(),
            frame: UiFrame::new(0.0, 0.0, 91.0, 12.0),
            source_range: UiTextRange { start: 0, end: 5 },
            visual_range: UiTextRange { start: 0, end: 5 },
            measured_width: 91.0,
            glyph_advances: vec![9.0, 40.0, 9.0, 24.0, 9.0],
            baseline: 9.0,
            direction: UiTextDirection::LeftToRight,
            runs: vec![visual_run("a\tb c", 0, 5, 0, 5)],
            ellipsized: false,
        };

        let glyphs = visual_glyphs_for_line(
            "a\tb c",
            0,
            &line,
            TextShapeResult {
                runs: vec![TextShapeRun {
                    source_range: 0..5,
                    direction: crate::core::framework::text::TextDirection::LeftToRight,
                    glyphs: vec![
                        glyph(1, 0..1),
                        glyph(2, 1..2),
                        glyph(3, 2..3),
                        glyph(4, 3..4),
                        glyph(5, 4..5),
                    ],
                }],
                metrics: Default::default(),
                resolved_direction: crate::core::framework::text::TextDirection::LeftToRight,
            },
        );

        assert_eq!(
            glyphs.iter().map(|glyph| glyph.advance).collect::<Vec<_>>(),
            line.glyph_advances
        );
    }

    fn visual_run(
        text: &str,
        source_start: usize,
        source_end: usize,
        visual_start: usize,
        visual_end: usize,
    ) -> zircon_runtime_interface::ui::surface::UiResolvedTextRun {
        UiResolvedTextRun {
            kind: UiTextRunKind::Plain,
            text: text.to_string(),
            source_range: UiTextRange {
                start: source_start,
                end: source_end,
            },
            visual_range: UiTextRange {
                start: visual_start,
                end: visual_end,
            },
            direction: UiTextDirection::RightToLeft,
        }
    }

    fn glyph(glyph_id: u32, source_range: std::ops::Range<usize>) -> TextGlyph {
        TextGlyph {
            glyph_id,
            source_range,
            visual_range: 0..0,
            advance: 10.0,
            position: [0.0, 0.0],
            offset: [0.0, 0.0],
            font_face: None,
            font_instance: None,
            rotation: TextGlyphRotation::None,
            bidi_level: 1,
            flags: TextGlyphFlags {
                right_to_left: true,
                ..TextGlyphFlags::default()
            },
            requires_rasterization: true,
        }
    }
}
