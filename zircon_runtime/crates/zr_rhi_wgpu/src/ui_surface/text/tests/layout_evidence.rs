use super::*;
#[test]
fn actual_glyphon_buffer_exports_fractional_line_and_resolved_font_identity() {
    let mut fonts = FontSystem::new();
    let mut buffer = Buffer::new(&mut fonts, glyphon::Metrics::new(18.0, 24.0));
    let command = UiSurfaceCommand {
        z_index: 0,
        frame: UiSurfaceRect::new(12.5, 18.75, 300.0, 60.0),
        clip: None,
        kind: zr_rhi::UiSurfaceCommandKind::Clip,
    };
    super::super::prepare_buffer(
        &mut fonts,
        &mut buffer,
        &command,
        "Actual line",
        None,
        400,
        zr_rhi::UiSurfaceTextStyle::Regular,
    );
    let mut cache = std::collections::HashMap::new();
    let run = observe_buffer(
        &fonts,
        &mut cache,
        &buffer,
        &command,
        7,
        "Actual line",
        command.frame,
    );
    let actual = buffer
        .layout_runs()
        .next()
        .expect("installed fonts produce a real shaped line");
    assert_eq!(run.lines[0].frame.y, command.frame.y + actual.line_top);
    assert_eq!(run.lines[0].frame.width, actual.line_w);
    assert_eq!(run.lines[0].baseline_y, command.frame.y + actual.line_y);
    assert!(!run.faces.is_empty());
    assert!(run.faces.iter().all(|face| face.sha256.len() == 64));
    assert_eq!(run.command_index, 7);
    let cache_count = cache.len();
    let repeated = observe_buffer(
        &fonts,
        &mut cache,
        &buffer,
        &command,
        8,
        "Actual line",
        command.frame,
    );
    assert_eq!(cache_count, cache.len());
    assert_eq!(run.faces.len(), repeated.faces.len());
    assert!(run
        .lines
        .iter()
        .flat_map(|line| &line.font_ids)
        .all(|id| run.faces.iter().any(|face| &face.font_id == id)));
    for face in &run.faces {
        let actual_id = actual
            .glyphs
            .iter()
            .find(|glyph| format!("{:?}", glyph.font_id) == face.font_id)
            .unwrap()
            .font_id;
        let (digest, index) = fonts
            .db()
            .with_face_data(actual_id, |bytes, index| {
                (ZrRuntimeDigestV1::sha256(bytes).as_str().to_owned(), index)
            })
            .unwrap();
        assert_eq!(face.sha256, digest);
        assert_eq!(face.face_index, index);
    }
}
