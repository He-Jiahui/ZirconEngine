use std::collections::HashMap;

use zircon_runtime::core::framework::text::{TextFontFaceHandle, TextGlyph, TextGlyphRotation};

use super::super::super::font::{host_runtime_artifact_font_snapshot, HostTextFontSnapshot};
use super::super::placement::retained_text_origin_for_smoothing;
use super::metrics::centered_line_y;
use super::runtime_lines::RuntimeTextLine;
use super::RuntimeTextGlyph;
use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_text::layout_policy::HostTextLayoutPolicy;
use crate::ui::retained_host::host_contract::paint_theme::HostTextSmoothing;

/// Fully preflighted direct artifact input for one retained-host text layout.
///
/// The consumer either receives every raster glyph from the exact runtime artifact or receives
/// `None` and falls back as a whole. This prevents a line from mixing retained-host glyph IDs with
/// runtime fallback-face glyph IDs after a partial face conversion failure.
pub(super) struct PositionedArtifactGlyphs {
    pub(super) glyphs: Vec<RuntimeTextGlyph>,
    pub(super) raster_fonts: Vec<HostTextFontSnapshot>,
}

pub(super) fn positioned_artifact_glyphs(
    lines: &[RuntimeTextLine],
    rect: &FrameRect,
    font_size: f32,
    line_height: f32,
    smoothing: HostTextSmoothing,
    layout_policy: HostTextLayoutPolicy,
) -> Option<PositionedArtifactGlyphs> {
    zircon_runtime::profile_scope!(
        "editor",
        "host_painter",
        "runtime_artifact_glyph_projection"
    );
    let artifact_layout = lines.first()?.artifact_line.as_ref()?;
    if !lines.iter().all(|line| {
        line.artifact_line
            .as_ref()
            .is_some_and(|artifact_line| artifact_layout.shares_artifact_layout_with(artifact_line))
    }) {
        return None;
    }
    let faces = artifact_layout.artifact_raster_faces()?;
    let mut font_indices = HashMap::new();
    let mut raster_fonts = Vec::new();
    let mut glyphs = Vec::new();

    for line in lines {
        let artifact_line = line.artifact_line.as_ref()?;
        let (line_x, line_y) =
            artifact_line_origin(line, rect, line_height, smoothing, layout_policy);

        for glyph in artifact_line.glyphs()? {
            if !glyph.requires_rasterization {
                continue;
            }
            if glyph.rotation != TextGlyphRotation::None {
                return None;
            }

            let face_key = (glyph.font_face?, glyph.font_instance);
            let raster_font_index = if let Some(index) = font_indices.get(&face_key) {
                *index
            } else {
                let runtime_face = faces.face_for(glyph)?;
                let index = raster_fonts.len();
                raster_fonts.push(host_runtime_artifact_font_snapshot(runtime_face)?);
                font_indices.insert(face_key, index);
                index
            };
            glyphs.push(artifact_glyph_geometry(
                glyph,
                line_x,
                line_y,
                font_size,
                raster_font_index,
            )?);
        }
    }

    Some(PositionedArtifactGlyphs {
        glyphs,
        raster_fonts,
    })
}

pub(super) fn artifact_glyph_geometry(
    glyph: &TextGlyph,
    line_x: f32,
    line_y: f32,
    font_size: f32,
    raster_font_index: usize,
) -> Option<RuntimeTextGlyph> {
    let glyph_index = u16::try_from(glyph.glyph_id).ok()?;
    let origin_x = line_x + glyph.position[0] + glyph.offset[0];
    let origin_y = line_y + glyph.position[1] + glyph.offset[1];
    (origin_x.is_finite() && origin_y.is_finite() && font_size.is_finite() && font_size > 0.0)
        .then_some(RuntimeTextGlyph {
            glyph_index,
            px: font_size,
            x: origin_x,
            origin_x,
            y: origin_y,
            raster_font_index: Some(raster_font_index),
        })
}

fn artifact_line_origin(
    line: &RuntimeTextLine,
    rect: &FrameRect,
    line_height: f32,
    smoothing: HostTextSmoothing,
    layout_policy: HostTextLayoutPolicy,
) -> (f32, f32) {
    let line_y = match layout_policy {
        HostTextLayoutPolicy::SingleLineEllipsis => {
            centered_line_y(rect.y, rect.height, line_height)
        }
        HostTextLayoutPolicy::WordWrap => rect.y + line.frame_y,
    };
    (
        retained_text_origin_for_smoothing(rect.x + line.frame_x, smoothing),
        line_y,
    )
}
