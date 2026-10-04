//! Reads the same glyphon Buffer subsequently passed to TextRenderer::prepare.
use glyphon::{Buffer, FontSystem};
use zircon_runtime_interface::runtime_build_set::ZrRuntimeDigestV1;
use zr_rhi::{
    UiSurfaceCommand, UiSurfaceRect, UiSurfaceTextFace, UiSurfaceTextLayoutRun, UiSurfaceTextLine,
};
pub(super) fn observe_buffer(
    fonts: &FontSystem,
    cache: &mut std::collections::HashMap<String, UiSurfaceTextFace>,
    buffer: &Buffer,
    command: &UiSurfaceCommand,
    command_index: usize,
    text: &str,
    clip: UiSurfaceRect,
) -> UiSurfaceTextLayoutRun {
    let mut faces = Vec::new();
    let mut ids = std::collections::HashSet::new();
    let lines = buffer
        .layout_runs()
        .map(|run| {
            for glyph in run.glyphs {
                if !ids.insert(glyph.font_id) {
                    continue;
                }
                let font_id = format!("{:?}", glyph.font_id);
                if let Some(face) = cache.get(&font_id) {
                    faces.push(face.clone());
                    continue;
                }
                let Some(face) = fonts.db().face(glyph.font_id) else {
                    continue;
                };
                let Some((sha256, face_index)) =
                    fonts.db().with_face_data(glyph.font_id, |bytes, index| {
                        (ZrRuntimeDigestV1::sha256(bytes).as_str().to_owned(), index)
                    })
                else {
                    continue;
                };
                let observed_face = UiSurfaceTextFace {
                    font_id: font_id.clone(),
                    sha256,
                    face_index,
                    families: face.families.iter().map(|(name, _)| name.clone()).collect(),
                    post_script_name: face.post_script_name.clone(),
                    weight: face.weight.0,
                    style: format!("{:?}", face.style),
                };
                cache.insert(font_id, observed_face.clone());
                faces.push(observed_face);
            }
            let x = run
                .glyphs
                .iter()
                .map(|glyph| glyph.x)
                .reduce(f32::min)
                .unwrap_or(0.0);
            UiSurfaceTextLine {
                original_line_text: run.text.to_owned(),
                line_index: run.line_i,
                byte_range: run
                    .glyphs
                    .iter()
                    .map(|glyph| glyph.start)
                    .min()
                    .zip(run.glyphs.iter().map(|glyph| glyph.end).max()),
                frame: UiSurfaceRect::new(
                    command.frame.x + x,
                    command.frame.y + run.line_top,
                    run.line_w,
                    run.line_height,
                ),
                baseline_y: command.frame.y + run.line_y,
                font_ids: run
                    .glyphs
                    .iter()
                    .map(|glyph| format!("{:?}", glyph.font_id))
                    .collect::<std::collections::BTreeSet<_>>()
                    .into_iter()
                    .collect(),
            }
        })
        .collect();
    UiSurfaceTextLayoutRun {
        command_index,
        text: text.to_owned(),
        clip,
        lines,
        faces,
    }
}
#[cfg(test)]
#[path = "tests/layout_evidence.rs"]
mod tests;
