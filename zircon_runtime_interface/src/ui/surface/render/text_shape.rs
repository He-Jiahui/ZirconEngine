use serde::{Deserialize, Serialize};
use unicode_segmentation::UnicodeSegmentation;

use crate::ui::layout::UiFrame;

use super::{
    UiRenderResourceKey, UiResolvedStyle, UiResolvedTextLayout, UiResourceUvRect, UiTextCaret,
    UiTextComposition, UiTextDecorations, UiTextDirection, UiTextDistanceFieldEffects,
    UiTextOverflow, UiTextRange, UiTextRenderMode, UiTextRunKind, UiTextSelection,
    UiTextWritingMode,
};

fn default_text_font_weight() -> u16 {
    UiResolvedStyle::DEFAULT_FONT_WEIGHT
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiTextPaint {
    pub source_text: String,
    pub color: Option<String>,
    pub font: Option<String>,
    pub font_family: Option<String>,
    #[serde(default = "default_text_font_weight")]
    pub font_weight: u16,
    pub font_size: f32,
    pub line_height: f32,
    #[serde(default)]
    pub writing_mode: UiTextWritingMode,
    pub render_mode: UiTextRenderMode,
    #[serde(default)]
    pub text_effects: UiTextDistanceFieldEffects,
    #[serde(default)]
    pub text_decorations: UiTextDecorations,
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
        let font_weight = default_text_font_weight();
        let runs = text_paint_runs_from_shaped(
            &shaped,
            &color,
            &None,
            &None,
            font_weight,
            shaped.font_size,
            shaped.line_height,
        );
        Self {
            source_text: shaped.source_text.clone(),
            color,
            font: None,
            font_family: None,
            font_weight,
            font_size: shaped.font_size,
            line_height: shaped.line_height,
            writing_mode: shaped.writing_mode,
            render_mode: shaped.render_mode,
            text_effects: UiTextDistanceFieldEffects::default(),
            text_decorations: UiTextDecorations::default(),
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
    #[serde(default = "default_text_font_weight")]
    pub font_weight: u16,
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
    #[serde(default = "default_text_decoration_thickness")]
    pub thickness: f32,
}

impl UiTextPaintDecoration {
    pub fn selection(range: UiTextRange, frame: UiFrame, color: impl Into<String>) -> Self {
        Self {
            kind: UiTextPaintDecorationKind::Selection,
            range,
            frame,
            color: color.into(),
            thickness: default_text_decoration_thickness(),
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
            thickness: default_text_decoration_thickness(),
        }
    }

    pub fn table_cell_background(
        range: UiTextRange,
        frame: UiFrame,
        color: impl Into<String>,
    ) -> Self {
        Self {
            kind: UiTextPaintDecorationKind::TableCellBackground,
            range,
            frame,
            color: color.into(),
            thickness: default_text_decoration_thickness(),
        }
    }

    pub fn table_cell_border(
        range: UiTextRange,
        frame: UiFrame,
        color: impl Into<String>,
        thickness: f32,
    ) -> Self {
        Self {
            kind: UiTextPaintDecorationKind::TableCellBorder,
            range,
            frame,
            color: color.into(),
            thickness: thickness.max(0.0),
        }
    }
}

const fn default_text_decoration_thickness() -> f32 {
    1.0
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiTextPaintDecorationKind {
    Selection,
    Caret,
    CompositionUnderline,
    Outline,
    TableCellBackground,
    TableCellBorder,
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
    #[serde(default)]
    pub writing_mode: UiTextWritingMode,
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
            writing_mode: layout.writing_mode,
            render_mode,
            font_key: None,
            atlas_resource: None,
            ellipsis_range: None,
            lines: layout
                .lines
                .iter()
                .map(|line| shaped_line_from_resolved(line, layout.writing_mode))
                .collect(),
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font_id: Option<UiRenderResourceKey>,
    #[serde(default, skip_serializing_if = "UiShapedGlyphClusterFlags::is_default")]
    pub cluster_flags: UiShapedGlyphClusterFlags,
    #[serde(default, skip_serializing_if = "UiShapedGlyphRotation::is_none")]
    pub rotation: UiShapedGlyphRotation,
    #[serde(default)]
    pub atlas_resource: Option<UiRenderResourceKey>,
    #[serde(default)]
    pub uv_rect: Option<UiResourceUvRect>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiShapedGlyphClusterFlags {
    pub cluster_start: bool,
    pub rtl: bool,
    pub whitespace: bool,
    pub space: bool,
    pub tab: bool,
    pub mandatory_break: bool,
    pub soft_break: bool,
    pub virtual_glyph: bool,
}

impl UiShapedGlyphClusterFlags {
    pub fn is_default(&self) -> bool {
        *self == Self::default()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiShapedGlyphRotation {
    #[default]
    None,
    Cw90,
}

impl UiShapedGlyphRotation {
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }
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
            font_id: None,
            cluster_flags: UiShapedGlyphClusterFlags::default(),
            rotation: UiShapedGlyphRotation::None,
            atlas_resource: None,
            uv_rect: None,
        }
    }

    pub fn with_font_id(mut self, font_id: UiRenderResourceKey) -> Self {
        self.font_id = Some(font_id);
        self
    }

    pub fn with_cluster_flags(mut self, cluster_flags: UiShapedGlyphClusterFlags) -> Self {
        self.cluster_flags = cluster_flags;
        self
    }

    pub fn with_rotation(mut self, rotation: UiShapedGlyphRotation) -> Self {
        self.rotation = rotation;
        self
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

fn shaped_line_from_resolved(
    line: &super::UiResolvedTextLine,
    writing_mode: UiTextWritingMode,
) -> UiShapedTextLine {
    UiShapedTextLine {
        text: line.text.clone(),
        frame: line.frame,
        source_range: line.source_range,
        visual_range: line.visual_range,
        measured_width: line.measured_width,
        baseline: line.baseline,
        direction: line.direction,
        ellipsized: line.ellipsized,
        glyphs: shaped_glyphs_for_line(line, writing_mode),
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

fn shaped_glyphs_for_line(
    line: &super::UiResolvedTextLine,
    writing_mode: UiTextWritingMode,
) -> Vec<UiShapedGlyph> {
    let graphemes = line.text.grapheme_indices(true).collect::<Vec<_>>();
    if graphemes.is_empty() {
        return Vec::new();
    }

    let advances = glyph_advances_for_line(line, graphemes.len());
    if matches!(writing_mode, UiTextWritingMode::VerticalRl) {
        return shaped_vertical_glyphs_for_line(line, &graphemes, advances);
    }

    let mut cursor_x = line.frame.x;
    graphemes
        .iter()
        .zip(advances)
        .map(|((visual_start, grapheme), advance)| {
            let visual_end = *visual_start + grapheme.len();
            let visual_frame =
                UiFrame::new(cursor_x, line.frame.y, advance.max(0.0), line.frame.height);
            cursor_x += advance.max(0.0);
            UiShapedGlyph::new(
                synthetic_glyph_id(grapheme),
                source_range_for_visual_span(line, *visual_start, visual_end),
                visual_frame,
                advance,
            )
            .with_cluster_flags(cluster_flags_for_grapheme(grapheme, line.direction))
        })
        .collect()
}

fn shaped_vertical_glyphs_for_line(
    line: &super::UiResolvedTextLine,
    graphemes: &[(usize, &str)],
    advances: Vec<f32>,
) -> Vec<UiShapedGlyph> {
    let mut cursor_y = line.frame.y;
    graphemes
        .iter()
        .zip(advances)
        .map(|((visual_start, grapheme), advance)| {
            let visual_end = *visual_start + grapheme.len();
            let advance = advance.max(0.0);
            let visual_frame = UiFrame::new(line.frame.x, cursor_y, line.frame.width, advance);
            cursor_y += advance;
            UiShapedGlyph::new(
                synthetic_glyph_id(grapheme),
                source_range_for_visual_span(line, *visual_start, visual_end),
                visual_frame,
                advance,
            )
            .with_cluster_flags(cluster_flags_for_grapheme(grapheme, line.direction))
            .with_rotation(vertical_grapheme_rotation(grapheme))
        })
        .collect()
}

fn vertical_grapheme_rotation(grapheme: &str) -> UiShapedGlyphRotation {
    if grapheme
        .chars()
        .all(|ch| ch.is_ascii() && !ch.is_whitespace())
    {
        UiShapedGlyphRotation::Cw90
    } else {
        UiShapedGlyphRotation::None
    }
}

fn glyph_advances_for_line(line: &super::UiResolvedTextLine, grapheme_count: usize) -> Vec<f32> {
    if line.glyph_advances.len() == grapheme_count {
        let advances = line
            .glyph_advances
            .iter()
            .map(|advance| sanitized_advance(*advance))
            .collect::<Vec<_>>();
        if advances.iter().any(|advance| *advance > 0.0) {
            return advances;
        }
    }

    let fallback_width = if line.measured_width.is_finite() && line.measured_width > 0.0 {
        line.measured_width
    } else {
        line.frame.width.max(0.0)
    };
    let fallback_advance = fallback_width / grapheme_count.max(1) as f32;
    vec![fallback_advance; grapheme_count]
}

fn sanitized_advance(advance: f32) -> f32 {
    if advance.is_finite() {
        advance.max(0.0)
    } else {
        0.0
    }
}

fn cluster_flags_for_grapheme(
    grapheme: &str,
    direction: UiTextDirection,
) -> UiShapedGlyphClusterFlags {
    UiShapedGlyphClusterFlags {
        cluster_start: true,
        rtl: matches!(direction, UiTextDirection::RightToLeft),
        whitespace: grapheme.chars().any(char::is_whitespace),
        space: grapheme.chars().any(|ch| matches!(ch, ' ' | '\u{00a0}')),
        tab: grapheme.contains('\t'),
        mandatory_break: grapheme.chars().any(|ch| matches!(ch, '\n' | '\r')),
        soft_break: false,
        virtual_glyph: false,
    }
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
        let mapped = source_range_for_run_visual_span(run, local_start, local_end);
        source_start = source_start.min(mapped.start);
        source_end = source_end.max(mapped.end);
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

fn source_range_for_run_visual_span(
    run: &super::UiResolvedTextRun,
    local_start: usize,
    local_end: usize,
) -> UiTextRange {
    if local_start >= local_end {
        return UiTextRange {
            start: run.source_range.start,
            end: run.source_range.start,
        };
    }
    if run.source_range.end.saturating_sub(run.source_range.start) != run.text.len() {
        return run.source_range;
    }
    UiTextRange {
        start: run.source_range.start + local_start,
        end: run.source_range.start + local_end,
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
    font_weight: u16,
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
                frame: text_run_frame(shaped.writing_mode, line, cluster.visual_range),
                color: color.clone(),
                font: font.clone(),
                font_family: font_family.clone(),
                font_weight,
                font_size,
                line_height,
                style: UiTextRunPaintStyle::from_run_kind(cluster.kind),
            });
        }
    }
    runs
}

fn text_run_frame(
    writing_mode: UiTextWritingMode,
    line: &UiShapedTextLine,
    visual_range: UiTextRange,
) -> UiFrame {
    let visual_start = grapheme_floor(line.text.as_str(), visual_range.start);
    let visual_end = grapheme_ceil(line.text.as_str(), visual_range.end);
    if matches!(writing_mode, UiTextWritingMode::VerticalRl) {
        let y0 = line_visual_y(line, visual_start);
        let y1 = line_visual_y(line, visual_end);
        return UiFrame::new(line.frame.x, y0.min(y1), line.frame.width, (y1 - y0).abs());
    }

    let x0 = line_visual_x(line, visual_start);
    let x1 = line_visual_x(line, visual_end);
    UiFrame::new(x0.min(x1), line.frame.y, (x1 - x0).abs(), line.frame.height)
}

fn line_visual_y(line: &UiShapedTextLine, visual_offset: usize) -> f32 {
    let text = line.text.as_str();
    let offset = grapheme_floor(text, visual_offset.min(text.len()));
    let total_units = text.graphemes(true).count();
    let before_units = text[..offset].graphemes(true).count();
    if line.glyphs.len() == total_units {
        return line.frame.y
            + line
                .glyphs
                .iter()
                .take(before_units)
                .map(|glyph| sanitized_advance(glyph.advance))
                .sum::<f32>();
    }

    let total_units = total_units.max(1) as f32;
    let before_units = before_units as f32;
    line.frame.y + (line.frame.height.max(0.0) * before_units / total_units)
}

fn line_visual_x(line: &UiShapedTextLine, visual_offset: usize) -> f32 {
    let text = line.text.as_str();
    let offset = grapheme_floor(text, visual_offset.min(text.len()));
    let total_units = text.graphemes(true).count();
    let before_units = text[..offset].graphemes(true).count();
    if line.glyphs.len() == total_units {
        return line.frame.x
            + line
                .glyphs
                .iter()
                .take(before_units)
                .map(|glyph| sanitized_advance(glyph.advance))
                .sum::<f32>();
    }

    let total_units = total_units.max(1) as f32;
    let before_units = before_units as f32;
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
