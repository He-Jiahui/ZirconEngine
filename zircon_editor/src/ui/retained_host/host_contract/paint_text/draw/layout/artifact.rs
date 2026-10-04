use std::collections::HashMap;

use zircon_runtime::core::framework::text::{TextFontFaceHandle, TextGlyph, TextGlyphRotation};
use zircon_runtime::ui::surface::UiTextGlyphArtifactRasterFace;

use super::super::placement::retained_text_origin_for_smoothing;
use super::metrics::centered_line_y;
use super::runtime_lines::RuntimeTextLine;
use super::RuntimeTextGlyph;
use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_text::layout_policy::HostTextLayoutPolicy;
use crate::ui::retained_host::host_contract::paint_theme::HostTextSmoothing;

pub(super) struct PositionedArtifactGlyphs {
    pub(super) glyphs: Vec<RuntimeTextGlyph>,
    pub(super) raster_faces: Vec<UiTextGlyphArtifactRasterFace>,
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
    let resolved_faces = artifact_layout.artifact_raster_faces()?;
    let raster_faces = resolved_faces.faces().to_vec();
    let face_indices = raster_faces
        .iter()
        .enumerate()
        .map(|(index, face)| ((face.font_face(), face.font_instance()), index))
        .collect::<HashMap<_, _>>();
    let glyph_capacity = lines.iter().try_fold(0_usize, |capacity, line| {
        capacity.checked_add(line.artifact_line.as_ref()?.glyphs()?.len())
    })?;
    let physical_ppem = physical_ppem(font_size)?;
    let mut glyphs = Vec::with_capacity(glyph_capacity);

    for line in lines {
        let artifact_line = line.artifact_line.as_ref()?;
        let (line_x, baseline_y) =
            artifact_line_origin(line, rect, line_height, smoothing, layout_policy)?;
        for glyph in artifact_line.glyphs()? {
            if !glyph.requires_rasterization {
                continue;
            }
            let face_pair = (glyph.font_face?, glyph.font_instance);
            let raster_face_index = *face_indices.get(&face_pair)?;
            glyphs.push(artifact_glyph_geometry(
                glyph,
                line_x,
                baseline_y,
                physical_ppem,
                raster_face_index,
            )?);
        }
    }

    Some(PositionedArtifactGlyphs {
        glyphs,
        raster_faces,
    })
}

pub(super) fn artifact_glyph_geometry(
    glyph: &TextGlyph,
    line_x: f32,
    baseline_y: f32,
    physical_ppem: u32,
    raster_face_index: usize,
) -> Option<RuntimeTextGlyph> {
    if glyph.rotation != TextGlyphRotation::None || physical_ppem == 0 {
        return None;
    }
    let origin_x = line_x + glyph.position[0] + glyph.offset[0];
    // Horizontal artifact consumers use the line baseline plus the shaped origin adjustment.
    // `position[1]` is shaping-space metadata and is not an additional screen-space baseline.
    let baseline_y = baseline_y + glyph.offset[1];
    (origin_x.is_finite() && baseline_y.is_finite()).then_some(RuntimeTextGlyph {
        glyph_id: glyph.glyph_id,
        physical_ppem,
        origin_x,
        baseline_y,
        raster_face_index,
    })
}

fn artifact_line_origin(
    line: &RuntimeTextLine,
    rect: &FrameRect,
    line_height: f32,
    smoothing: HostTextSmoothing,
    layout_policy: HostTextLayoutPolicy,
) -> Option<(f32, f32)> {
    let artifact_line = line.artifact_line.as_ref()?;
    let baseline = artifact_line.layout_line()?.baseline;
    let line_y = match layout_policy {
        HostTextLayoutPolicy::SingleLineEllipsis => {
            centered_line_y(rect.y, rect.height, line_height)
        }
        HostTextLayoutPolicy::WordWrap => rect.y + line.frame_y,
    };
    let line_x = retained_text_origin_for_smoothing(rect.x + line.frame_x, smoothing);
    (line_x.is_finite() && line_y.is_finite() && baseline.is_finite())
        .then_some((line_x, line_y + baseline))
}

fn physical_ppem(font_size: f32) -> Option<u32> {
    (font_size.is_finite() && font_size > 0.0).then(|| font_size.round().max(1.0) as u32)
}

#[cfg(test)]
#[path = "tests/artifact.rs"]
mod tests;
