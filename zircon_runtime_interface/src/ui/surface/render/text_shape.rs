use serde::{Deserialize, Serialize};
use unicode_segmentation::UnicodeSegmentation;

use crate::ui::layout::UiFrame;

use super::{
    UiRenderResourceKey, UiResolvedTextLayout, UiResourceUvRect, UiTextCaret, UiTextComposition,
    UiTextDirection, UiTextOverflow, UiTextRange, UiTextRenderMode, UiTextRunKind, UiTextSelection,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiTextPaint {
    pub source_text: String,
    pub color: Option<String>,
    pub font: Option<String>,
    pub font_family: Option<String>,
    pub font_size: f32,
    pub line_height: f32,
    pub render_mode: UiTextRenderMode,
    pub overflow: UiTextOverflow,
    pub shaped: Option<UiShapedText>,
    #[serde(default)]
    pub selection: Option<UiTextSelection>,
    #[serde(default)]
    pub caret: Option<UiTextCaret>,
    #[serde(default)]
    pub composition: Option<UiTextComposition>,
    #[serde(default)]
    pub decorations: Vec<UiTextPaintDecoration>,
    #[serde(default)]
    pub runs: Vec<UiTextPaintRun>,
}

impl UiTextPaint {
    pub fn from_shaped_text(shaped: UiShapedText, color: Option<String>) -> Self {
        let runs = text_paint_runs_from_shaped(
            &shaped,
            &color,
            &None,
            &None,
            shaped.font_size,
            shaped.line_height,
        );
        Self {
            source_text: shaped.source_text.clone(),
            color,
            font: None,
            font_family: None,
            font_size: shaped.font_size,
            line_height: shaped.line_height,
            render_mode: shaped.render_mode,
            overflow: shaped.overflow,
            shaped: Some(shaped),
            selection: None,
            caret: None,
            composition: None,
            decorations: Vec::new(),
            runs,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiTextPaintRun {
    pub kind: UiTextRunKind,
    pub text: String,
    pub source_range: UiTextRange,
    pub visual_range: UiTextRange,
    pub frame: UiFrame,
    pub color: Option<String>,
    pub font: Option<String>,
    pub font_family: Option<String>,
    pub font_size: f32,
    pub line_height: f32,
    #[serde(default)]
    pub style: UiTextRunPaintStyle,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiTextRunPaintStyle {
    pub strong: bool,
    pub emphasis: bool,
    pub code: bool,
}

impl UiTextRunPaintStyle {
    pub const fn from_run_kind(kind: UiTextRunKind) -> Self {
        match kind {
            UiTextRunKind::Strong => Self {
                strong: true,
                emphasis: false,
                code: false,
            },
            UiTextRunKind::Emphasis => Self {
                strong: false,
                emphasis: true,
                code: false,
            },
            UiTextRunKind::Code => Self {
                strong: false,
                emphasis: false,
                code: true,
            },
            UiTextRunKind::Plain | UiTextRunKind::Link => Self {
                strong: false,
                emphasis: false,
                code: false,
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiTextPaintDecoration {
    pub kind: UiTextPaintDecorationKind,
    pub range: UiTextRange,
    pub frame: UiFrame,
    pub color: String,
}

impl UiTextPaintDecoration {
    pub fn selection(range: UiTextRange, frame: UiFrame, color: impl Into<String>) -> Self {
        Self {
            kind: UiTextPaintDecorationKind::Selection,
            range,
            frame,
            color: color.into(),
        }
    }

    pub fn composition_underline(
        range: UiTextRange,
        frame: UiFrame,
        color: impl Into<String>,
    ) -> Self {
        Self {
            kind: UiTextPaintDecorationKind::CompositionUnderline,
            range,
            frame,
            color: color.into(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiTextPaintDecorationKind {
    Selection,
    Caret,
    CompositionUnderline,
    Outline,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiShapedText {
    pub source_text: String,
    pub source_range: UiTextRange,
    pub direction: UiTextDirection,
    pub overflow: UiTextOverflow,
    pub font_size: f32,
    pub line_height: f32,
    pub measured_width: f32,
    pub measured_height: f32,
    pub render_mode: UiTextRenderMode,
    #[serde(default)]
    pub font_key: Option<UiRenderResourceKey>,
    #[serde(default)]
    pub atlas_resource: Option<UiRenderResourceKey>,
    #[serde(default)]
    pub ellipsis_range: Option<UiTextRange>,
    pub lines: Vec<UiShapedTextLine>,
}

impl UiShapedText {
    pub fn from_resolved_layout(
        source_text: impl Into<String>,
        layout: &UiResolvedTextLayout,
        render_mode: UiTextRenderMode,
    ) -> Self {
        Self {
            source_text: source_text.into(),
            source_range: layout.source_range,
            direction: layout.direction,
            overflow: layout.overflow,
            font_size: layout.font_size,
            line_height: layout.line_height,
            measured_width: layout.measured_width,
            measured_height: layout.measured_height,
            render_mode,
            font_key: None,
            atlas_resource: None,
            ellipsis_range: None,
            lines: layout.lines.iter().map(shaped_line_from_resolved).collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiShapedTextLine {
    pub text: String,
    pub frame: UiFrame,
    pub source_range: UiTextRange,
    pub visual_range: UiTextRange,
    pub measured_width: f32,
    pub baseline: f32,
    pub direction: UiTextDirection,
    pub ellipsized: bool,
    #[serde(default)]
    pub glyphs: Vec<UiShapedGlyph>,
    pub clusters: Vec<UiShapedTextCluster>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiShapedGlyph {
    pub glyph_id: u32,
    pub source_range: UiTextRange,
    pub visual_frame: UiFrame,
    pub advance: f32,
    #[serde(default)]
    pub atlas_resource: Option<UiRenderResourceKey>,
    #[serde(default)]
    pub uv_rect: Option<UiResourceUvRect>,
}

impl UiShapedGlyph {
    pub fn new(
        glyph_id: u32,
        source_range: UiTextRange,
        visual_frame: UiFrame,
        advance: f32,
    ) -> Self {
        Self {
            glyph_id,
            source_range,
            visual_frame,
            advance,
            atlas_resource: None,
            uv_rect: None,
        }
    }

    pub fn with_atlas(
        mut self,
        atlas_resource: UiRenderResourceKey,
        uv_rect: UiResourceUvRect,
    ) -> Self {
        self.atlas_resource = Some(atlas_resource);
        self.uv_rect = Some(uv_rect);
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiShapedTextCluster {
    pub kind: UiTextRunKind,
    pub text: String,
    pub source_range: UiTextRange,
    pub visual_range: UiTextRange,
    pub direction: UiTextDirection,
}

fn shaped_line_from_resolved(line: &super::UiResolvedTextLine) -> UiShapedTextLine {
    UiShapedTextLine {
        text: line.text.clone(),
        frame: line.frame,
        source_range: line.source_range,
        visual_range: line.visual_range,
        measured_width: line.measured_width,
        baseline: line.baseline,
        direction: line.direction,
        ellipsized: line.ellipsized,
        glyphs: shaped_glyphs_for_line(line),
        clusters: line
            .runs
            .iter()
            .map(|run| UiShapedTextCluster {
                kind: run.kind,
                text: run.text.clone(),
                source_range: run.source_range,
                visual_range: run.visual_range,
                direction: run.direction,
            })
            .collect(),
    }
}

fn shaped_glyphs_for_line(line: &super::UiResolvedTextLine) -> Vec<UiShapedGlyph> {
    let graphemes = line.text.grapheme_indices(true).collect::<Vec<_>>();
    if graphemes.is_empty() {
        return Vec::new();
    }

    let advance = line.frame.width.max(0.0) / graphemes.len() as f32;
    graphemes
        .iter()
        .enumerate()
        .map(|(index, (visual_start, grapheme))| {
            let visual_end = *visual_start + grapheme.len();
            UiShapedGlyph::new(
                synthetic_glyph_id(grapheme),
                source_range_for_visual_span(line, *visual_start, visual_end),
                UiFrame::new(
                    line.frame.x + advance * index as f32,
                    line.frame.y,
                    advance.max(0.0),
                    line.frame.height,
                ),
                advance,
            )
        })
        .collect()
}

fn source_range_for_visual_span(
    line: &super::UiResolvedTextLine,
    visual_start: usize,
    visual_end: usize,
) -> UiTextRange {
    let mut source_start = usize::MAX;
    let mut source_end = 0;
    for run in &line.runs {
        let overlap_start = visual_start.max(run.visual_range.start);
        let overlap_end = visual_end.min(run.visual_range.end);
        if overlap_start >= overlap_end {
            continue;
        }

        let local_start = overlap_start.saturating_sub(run.visual_range.start);
        let local_end = overlap_end.saturating_sub(run.visual_range.start);
        source_start = source_start.min(run.source_range.start + local_start);
        source_end = source_end.max(run.source_range.start + local_end);
    }

    if source_start == usize::MAX {
        UiTextRange {
            start: line.source_range.start,
            end: line.source_range.start,
        }
    } else {
        UiTextRange {
            start: source_start,
            end: source_end.max(source_start),
        }
    }
}

fn synthetic_glyph_id(grapheme: &str) -> u32 {
    let mut hash = 2_166_136_261_u32;
    for byte in grapheme.as_bytes() {
        hash ^= *byte as u32;
        hash = hash.wrapping_mul(16_777_619);
    }
    hash.max(1)
}

pub(crate) fn text_paint_runs_from_shaped(
    shaped: &UiShapedText,
    color: &Option<String>,
    font: &Option<String>,
    font_family: &Option<String>,
    font_size: f32,
    line_height: f32,
) -> Vec<UiTextPaintRun> {
    let mut runs = Vec::new();
    for line in &shaped.lines {
        for cluster in &line.clusters {
            if cluster.text.is_empty() {
                continue;
            }
            runs.push(UiTextPaintRun {
                kind: cluster.kind,
                text: cluster.text.clone(),
                source_range: cluster.source_range,
                visual_range: cluster.visual_range,
                frame: text_run_frame(line, cluster.visual_range),
                color: color.clone(),
                font: font.clone(),
                font_family: font_family.clone(),
                font_size,
                line_height,
                style: UiTextRunPaintStyle::from_run_kind(cluster.kind),
            });
        }
    }
    runs
}

fn text_run_frame(line: &UiShapedTextLine, visual_range: UiTextRange) -> UiFrame {
    let visual_start = grapheme_floor(line.text.as_str(), visual_range.start);
    let visual_end = grapheme_ceil(line.text.as_str(), visual_range.end);
    let x0 = line_visual_x(line, visual_start);
    let x1 = line_visual_x(line, visual_end);
    UiFrame::new(x0.min(x1), line.frame.y, (x1 - x0).abs(), line.frame.height)
}

fn line_visual_x(line: &UiShapedTextLine, visual_offset: usize) -> f32 {
    let text = line.text.as_str();
    let offset = grapheme_floor(text, visual_offset.min(text.len()));
    let total_units = text.graphemes(true).count().max(1) as f32;
    let before_units = text[..offset].graphemes(true).count() as f32;
    line.frame.x + (line.frame.width.max(0.0) * before_units / total_units)
}

fn grapheme_floor(text: &str, offset: usize) -> usize {
    let mut offset = offset.min(text.len());
    while offset > 0 && !text.is_char_boundary(offset) {
        offset -= 1;
    }
    for (start, grapheme) in text.grapheme_indices(true) {
        let end = start + grapheme.len();
        if start < offset && offset < end {
            return start;
        }
        if start >= offset {
            break;
        }
    }
    offset
}

fn grapheme_ceil(text: &str, offset: usize) -> usize {
    let mut offset = offset.min(text.len());
    while offset < text.len() && !text.is_char_boundary(offset) {
        offset += 1;
    }
    for (start, grapheme) in text.grapheme_indices(true) {
        let end = start + grapheme.len();
        if start < offset && offset < end {
            return end;
        }
        if start >= offset {
            break;
        }
    }
    offset
}
