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
        Some(UiSurfaceTextRequest {
            font_family: None,
            font_weight: 400,
            font_style: "normal".into(),
            font_size: 18.0,
            line_height: 24.0,
        }),
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
        Some(UiSurfaceTextRequest {
            font_family: None,
            font_weight: 400,
            font_style: "normal".into(),
            font_size: 18.0,
            line_height: 24.0,
        }),
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

#[test]
fn actual_editor_collection_keeps_600_and_650_requests_distinct_from_selected_700_face() {
    let bytes = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/fonts/editor-ui.ttc")).unwrap();
    let mut database = glyphon::fontdb::Database::new();
    database.load_font_data(bytes.clone());
    let mut fonts = FontSystem::new_with_locale_and_db("en-US".into(), database);
    for weight in [600, 650] {
        let mut buffer = Buffer::new(&mut fonts, glyphon::Metrics::new(18.0, 24.0));
        let command = UiSurfaceCommand {
            z_index: 0,
            frame: UiSurfaceRect::new(12.5, 18.75, 300.0, 60.0),
            clip: None,
            kind: zr_rhi::UiSurfaceCommandKind::Clip,
        };
        super::super::prepare_buffer(&mut fonts, &mut buffer, &command,
            "Roboto actual selected face", Some("Roboto"), weight,
            zr_rhi::UiSurfaceTextStyle::Regular);
        let observed = observe_buffer(&fonts, &mut std::collections::HashMap::new(),
            &buffer, &command, 7, "Roboto actual selected face", command.frame,
            Some(UiSurfaceTextRequest {
                font_family: Some("Roboto".into()), font_weight: weight,
                font_style: "normal".into(), font_size: 18.0, line_height: 24.0,
            }));
        assert_eq!(observed.requested.as_ref().unwrap().font_weight, weight);
        assert!(!observed.faces.is_empty());
        for selected in &observed.faces {
            assert_eq!(selected.weight, 700);
            assert_eq!(selected.style, "Normal");
            assert_eq!(selected.face_index, 10);
            assert_eq!(selected.sha256, ZrRuntimeDigestV1::sha256(&bytes).as_str());
            let actual = buffer.layout_runs().flat_map(|run| run.glyphs.iter())
                .find(|glyph| format!("{:?}", glyph.font_id) == selected.font_id)
                .unwrap().font_id;
            let face = fonts.db().face(actual).unwrap();
            assert_eq!(face.weight.0, selected.weight);
            assert_eq!(face.index, selected.face_index);
        }
        assert!(observed.lines.iter().flat_map(|line| &line.font_ids)
            .all(|id| observed.faces.iter().any(|face| &face.font_id == id)));
    }
}
