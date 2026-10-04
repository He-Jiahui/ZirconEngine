use serde::{Deserialize, Serialize};
use unicode_segmentation::UnicodeSegmentation;

use crate::ui::layout::UiFrame;

use super::{
    UiRenderResourceKey, UiResolvedStyle, UiResolvedTextLayout, UiResourceUvRect, UiTextCaret,
    UiTextComposition, UiTextDecorations, UiTextDirection, UiTextDistanceFieldEffects,
    UiTextOverflow, UiTextRange, UiTextRenderMode, UiTextRunKind, UiTextSelection,
    UiTextWritingMode,
};

#[cfg(test)]
#[path = "text_shape/tests/performance_tests.rs"]
mod performance_tests;

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
    #[serde(default)]
    pub shaped: UiTextShapeArtifact,
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
            shaped: UiTextShapeArtifact::Canonical(shaped),
            selection: None,
            caret: None,
            composition: None,
            decorations: Vec::new(),
            runs,
        }
    }
}

/// The only render-facing glyph identity accepted by the interface.
///
/// Layout geometry alone does not identify a glyph or its font face. Runtime producers therefore
/// either publish the canonical shaped artifact or explicitly report that it is unavailable.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiTextShapeArtifact {
    #[default]
    Unavailable,
    Canonical(UiShapedText),
}

impl UiTextShapeArtifact {
    pub fn canonical(&self) -> Option<&UiShapedText> {
        match self {
            Self::Unavailable => None,
            Self::Canonical(shaped) => Some(shaped),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiTextPaintRun {
    pub kind: UiTextRunKind,
    pub text: String,
    /// 原始文本中的 UTF-8 字节区间，按逻辑来源定位。
    pub source_range: UiTextRange,
    /// 解析后视觉文本中的 UTF-8 字节区间，用于字形几何投影。
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

    pub fn composition_highlight(
        range: UiTextRange,
        frame: UiFrame,
        color: impl Into<String>,
    ) -> Self {
        Self {
            kind: UiTextPaintDecorationKind::CompositionHighlight,
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
    CompositionHighlight,
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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiShapedTextLine {
    pub text: String,
    pub frame: UiFrame,
    pub source_range: UiTextRange,
    pub visual_range: UiTextRange,
    pub measured_width: f32,
    /// Cross-axis offset relative to `frame`: y for horizontal text and x for `VerticalRl`.
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

pub(crate) fn text_paint_runs_from_shaped(
    shaped: &UiShapedText,
    color: &Option<String>,
    font: &Option<String>,
    font_family: &Option<String>,
    font_weight: u16,
    font_size: f32,
    line_height: f32,
) -> Vec<UiTextPaintRun> {
    let run_capacity = shaped.lines.iter().map(|line| line.clusters.len()).sum();
    let mut runs = Vec::with_capacity(run_capacity);
    for line in &shaped.lines {
        let metrics = (line.clusters.len() > 1).then(|| ShapedTextLineMetrics::from_line(line));
        for cluster in &line.clusters {
            if cluster.text.is_empty() {
                continue;
            }
            let frame = metrics
                .as_ref()
                .map(|metrics| {
                    text_run_frame_with_metrics(
                        shaped.writing_mode,
                        line,
                        cluster.visual_range,
                        metrics,
                    )
                })
                .unwrap_or_else(|| text_run_frame(shaped.writing_mode, line, cluster.visual_range));
            runs.push(UiTextPaintRun {
                kind: cluster.kind,
                text: cluster.text.clone(),
                source_range: cluster.source_range,
                visual_range: cluster.visual_range,
                frame,
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

pub(crate) fn text_paint_runs_from_resolved_layout(
    layout: &UiResolvedTextLayout,
    color: &Option<String>,
    font: &Option<String>,
    font_family: &Option<String>,
    font_weight: u16,
    font_size: f32,
    line_height: f32,
) -> Vec<UiTextPaintRun> {
    let run_capacity = layout.lines.iter().map(|line| line.runs.len()).sum();
    let mut runs = Vec::with_capacity(run_capacity);
    // 非空 run 须连续覆盖 visual_range 且匹配文本切片；任一处不合契约就拒绝整批投影。
    for line in &layout.lines {
        let mut expected_visual_start = line.visual_range.start;
        let mut has_nonempty_run = false;
        let mut metrics: Option<ResolvedTextLineMetrics> = None;
        for run in &line.runs {
            if run.text.is_empty() {
                continue;
            }
            if run.visual_range.start != expected_visual_start
                || line.text.get(run.visual_range.start..run.visual_range.end)
                    != Some(run.text.as_str())
            {
                return Vec::new();
            }
            has_nonempty_run = true;
            expected_visual_start = run.visual_range.end;
            let frame = if line.runs.len() > 1 {
                let metrics =
                    metrics.get_or_insert_with(|| ResolvedTextLineMetrics::from_line(line));
                if !metrics.valid {
                    return Vec::new();
                }
                resolved_text_run_frame_with_metrics(
                    layout.writing_mode,
                    line,
                    run.visual_range,
                    metrics,
                )
            } else {
                let Some(frame) =
                    resolved_text_run_frame(layout.writing_mode, line, run.visual_range)
                else {
                    return Vec::new();
                };
                frame
            };
            runs.push(UiTextPaintRun {
                kind: run.kind,
                text: run.text.clone(),
                source_range: run.source_range,
                visual_range: run.visual_range,
                frame,
                color: color.clone(),
                font: font.clone(),
                font_family: font_family.clone(),
                font_weight,
                font_size,
                line_height,
                style: UiTextRunPaintStyle::from_run_kind(run.kind),
            });
        }
        if has_nonempty_run && expected_visual_start != line.visual_range.end {
            return Vec::new();
        }
    }
    runs
}

#[derive(Clone, Debug)]
struct ShapedTextLineMetrics {
    boundaries: Vec<usize>,
    advances: Option<Vec<f32>>,
}

impl ShapedTextLineMetrics {
    fn from_line(line: &UiShapedTextLine) -> Self {
        let boundaries = grapheme_boundaries(line.text.as_str());
        let advances = (line.glyphs.len() == boundaries.len().saturating_sub(1)).then(|| {
            let mut prefix = Vec::with_capacity(boundaries.len());
            prefix.push(0.0);
            for glyph in &line.glyphs {
                prefix.push(
                    prefix.last().copied().unwrap_or_default() + sanitized_advance(glyph.advance),
                );
            }
            prefix
        });
        Self {
            boundaries,
            advances,
        }
    }

    fn frame(
        &self,
        writing_mode: UiTextWritingMode,
        line: &UiShapedTextLine,
        visual_range: UiTextRange,
    ) -> UiFrame {
        let start = self.floor_index(visual_range.start);
        let end = self.ceil_index(visual_range.end);
        let start_position = self.position(line, start, writing_mode);
        let end_position = self.position(line, end, writing_mode);
        if matches!(writing_mode, UiTextWritingMode::VerticalRl) {
            UiFrame::new(
                line.frame.x,
                start_position.min(end_position),
                line.frame.width,
                (end_position - start_position).abs(),
            )
        } else {
            UiFrame::new(
                start_position.min(end_position),
                line.frame.y,
                (end_position - start_position).abs(),
                line.frame.height,
            )
        }
    }

    fn floor_index(&self, offset: usize) -> usize {
        let offset = offset.min(*self.boundaries.last().unwrap_or(&0));
        self.boundaries
            .partition_point(|boundary| *boundary <= offset)
            .saturating_sub(1)
    }

    fn ceil_index(&self, offset: usize) -> usize {
        let offset = offset.min(*self.boundaries.last().unwrap_or(&0));
        self.boundaries
            .partition_point(|boundary| *boundary < offset)
    }

    fn position(
        &self,
        line: &UiShapedTextLine,
        index: usize,
        writing_mode: UiTextWritingMode,
    ) -> f32 {
        let total = self.boundaries.len().saturating_sub(1).max(1);
        let before = index.min(total);
        let base = if matches!(writing_mode, UiTextWritingMode::VerticalRl) {
            line.frame.y
        } else {
            line.frame.x
        };
        if let Some(prefix) = &self.advances {
            return base + prefix[before];
        }
        let extent = if matches!(writing_mode, UiTextWritingMode::VerticalRl) {
            line.frame.height.max(0.0)
        } else {
            line.frame.width.max(0.0)
        };
        base + extent * before as f32 / total as f32
    }
}

#[derive(Clone, Debug)]
struct ResolvedTextLineMetrics {
    boundaries: Vec<usize>,
    advances: Vec<f32>,
    valid: bool,
}

impl ResolvedTextLineMetrics {
    fn from_line(line: &super::UiResolvedTextLine) -> Self {
        let boundaries = grapheme_boundaries(line.text.as_str());
        let valid = line.glyph_advances.len() == boundaries.len().saturating_sub(1)
            && line
                .glyph_advances
                .iter()
                .all(|advance| advance.is_finite() && *advance >= 0.0);
        let mut advances = Vec::with_capacity(line.glyph_advances.len() + 1);
        advances.push(0.0);
        for advance in &line.glyph_advances {
            advances.push(advances.last().copied().unwrap_or_default() + *advance);
        }
        Self {
            boundaries,
            advances,
            valid,
        }
    }

    fn frame(
        &self,
        writing_mode: UiTextWritingMode,
        line: &super::UiResolvedTextLine,
        visual_range: UiTextRange,
    ) -> Option<UiFrame> {
        if visual_range.start > visual_range.end
            || visual_range.end > line.text.len()
            || !line.text.is_char_boundary(visual_range.start)
            || !line.text.is_char_boundary(visual_range.end)
        {
            return None;
        }
        let start = self.floor_index(visual_range.start);
        let end = self.ceil_index(visual_range.end);
        if start >= end || !self.valid {
            return None;
        }
        let leading = self.advances[start];
        let advance = line.glyph_advances[start..end].iter().sum::<f32>();
        if matches!(writing_mode, UiTextWritingMode::VerticalRl) {
            Some(UiFrame::new(
                line.frame.x,
                line.frame.y + leading,
                line.frame.width,
                advance,
            ))
        } else {
            Some(UiFrame::new(
                line.frame.x + leading,
                line.frame.y,
                advance,
                line.frame.height,
            ))
        }
    }

    fn floor_index(&self, offset: usize) -> usize {
        let offset = offset.min(*self.boundaries.last().unwrap_or(&0));
        self.boundaries
            .partition_point(|boundary| *boundary <= offset)
            .saturating_sub(1)
    }

    fn ceil_index(&self, offset: usize) -> usize {
        let offset = offset.min(*self.boundaries.last().unwrap_or(&0));
        self.boundaries
            .partition_point(|boundary| *boundary < offset)
    }
}

fn grapheme_boundaries(text: &str) -> Vec<usize> {
    let mut boundaries = text
        .grapheme_indices(true)
        .map(|(start, _)| start)
        .collect::<Vec<_>>();
    boundaries.push(text.len());
    boundaries
}

fn text_run_frame_with_metrics(
    writing_mode: UiTextWritingMode,
    line: &UiShapedTextLine,
    visual_range: UiTextRange,
    metrics: &ShapedTextLineMetrics,
) -> UiFrame {
    metrics.frame(writing_mode, line, visual_range)
}

fn resolved_text_run_frame_with_metrics(
    writing_mode: UiTextWritingMode,
    line: &super::UiResolvedTextLine,
    visual_range: UiTextRange,
    metrics: &ResolvedTextLineMetrics,
) -> UiFrame {
    metrics
        .frame(writing_mode, line, visual_range)
        .expect("validated resolved text line metrics")
}

/// Resolves paint-run bounds only from layout-provided advances. This path deliberately refuses
/// incomplete advance data rather than synthesizing a width or a glyph artifact.
fn resolved_text_run_frame(
    writing_mode: UiTextWritingMode,
    line: &super::UiResolvedTextLine,
    visual_range: UiTextRange,
) -> Option<UiFrame> {
    let text = line.text.as_str();
    if visual_range.start > visual_range.end
        || visual_range.end > text.len()
        || !text.is_char_boundary(visual_range.start)
        || !text.is_char_boundary(visual_range.end)
    {
        return None;
    }
    let visual_start = grapheme_floor(text, visual_range.start);
    let visual_end = grapheme_ceil(text, visual_range.end);
    if visual_start >= visual_end {
        return None;
    }
    let grapheme_count = text.graphemes(true).count();
    if line.glyph_advances.len() != grapheme_count
        || line
            .glyph_advances
            .iter()
            .any(|advance| !advance.is_finite() || *advance < 0.0)
    {
        return None;
    }
    let start_index = text[..visual_start].graphemes(true).count();
    let end_index = text[..visual_end].graphemes(true).count();
    let leading = line.glyph_advances[..start_index].iter().sum::<f32>();
    let advance = line.glyph_advances[start_index..end_index]
        .iter()
        .sum::<f32>();
    if matches!(writing_mode, UiTextWritingMode::VerticalRl) {
        Some(UiFrame::new(
            line.frame.x,
            line.frame.y + leading,
            line.frame.width,
            advance,
        ))
    } else {
        Some(UiFrame::new(
            line.frame.x + leading,
            line.frame.y,
            advance,
            line.frame.height,
        ))
    }
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

fn sanitized_advance(advance: f32) -> f32 {
    if advance.is_finite() {
        advance.max(0.0)
    } else {
        0.0
    }
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

#[cfg(test)]
#[path = "tests/text_shape.rs"]
mod tests;

#[cfg(test)]
#[path = "text_shape/tests/resolved_layout_tests.rs"]
mod resolved_layout_tests;

#[cfg(all(test, windows))]
#[path = "text_shape/tests/projection_profile.rs"]
mod projection_profile;
