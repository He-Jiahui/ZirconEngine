use serde::{Deserialize, Serialize};

use crate::ui::layout::UiFrame;

use super::{
    UiResolvedTextLayout, UiTextDirection, UiTextOverflow, UiTextRange, UiTextRenderMode,
    UiTextRunKind,
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
}

impl UiTextPaint {
    pub fn from_shaped_text(shaped: UiShapedText, color: Option<String>) -> Self {
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
        }
    }
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
            lines: layout
                .lines
                .iter()
                .map(|line| UiShapedTextLine {
                    text: line.text.clone(),
                    frame: line.frame,
                    source_range: line.source_range,
                    visual_range: line.visual_range,
                    measured_width: line.measured_width,
                    baseline: line.baseline,
                    direction: line.direction,
                    ellipsized: line.ellipsized,
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
                })
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
    pub clusters: Vec<UiShapedTextCluster>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiShapedTextCluster {
    pub kind: UiTextRunKind,
    pub text: String,
    pub source_range: UiTextRange,
    pub visual_range: UiTextRange,
    pub direction: UiTextDirection,
}
