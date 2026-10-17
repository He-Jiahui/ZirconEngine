use super::*;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

#[test]
fn draw_text_fallback_color_projects_from_host_palette() {
    let mut palette = PALETTE;
    palette.text = [10, 11, 12, 255];

    assert_eq!(fallback_text_from_host(palette), [10, 11, 12, 255]);
}

#[test]
fn authored_request_reaches_real_raster_and_recording_without_leaking_on_early_return() {
    use crate::ui::retained_host::host_contract::data::FrameRect;
    use crate::ui::retained_host::host_contract::paint_frame::HostRecordedPaintKind;
    use crate::ui::retained_host::host_contract::paint_text::{
        active_authored_font_request, font_request_for_face, HostTextFontFace,
        HostTextFontRequestScope, TextPaintEvidenceScope,
    };
    use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;
    let request = font_request_for_face(HostTextFontFace::Mono);
    let rect = FrameRect {
        x: 8.0,
        y: 16.0,
        width: 140.0,
        height: 8.0,
    };
    let clip = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 160.0,
        height: 48.0,
    };
    let command = HostPaintCommand::text(
        rect.clone(),
        Some(clip),
        1,
        "Authored input".into(),
        [255; 4],
        17.0,
        8.0,
        UiTextRunPaintStyle::default(),
        1.0,
    )
    .with_text_font_request(Some(request.clone()));
    let evidence = TextPaintEvidenceScope::begin().unwrap();
    let mut pixels = HostRgbaFrame::filled(160, 48, [0, 0, 0, 255]);
    assert!(draw_text_command(&mut pixels, &command));
    assert!(pixels
        .as_bytes()
        .chunks_exact(4)
        .any(|pixel| pixel != [0, 0, 0, 255]));
    let evidence = evidence.finish().unwrap();
    assert_eq!(evidence["runs"][0]["fontFamily"], request.family);
    assert_eq!(evidence["runs"][0]["fontWeight"], request.weight);
    assert_eq!(evidence["runs"][0]["fontSize"], 17.0);
    assert_eq!(evidence["runs"][0]["lineHeight"], 8.0);
    assert!(evidence["runs"][0]["glyphs"]
        .as_array()
        .is_some_and(|glyphs| !glyphs.is_empty()));
    let mut recorded = HostRgbaFrame::recording_only(160, 48);
    assert!(draw_text_command(&mut recorded, &command));
    let recorded = recorded.into_recorded_commands();
    assert_eq!(recorded[0].frame, rect);
    let HostRecordedPaintKind::Text {
        font_request,
        font_size,
        line_height,
        ..
    } = &recorded[0].kind
    else {
        panic!("text recording");
    };
    assert_eq!(font_request.as_ref(), Some(&request));
    assert_eq!((*font_size, *line_height), (17.0, 8.0));
    assert!(active_authored_font_request().is_none());
    let _outer = HostTextFontRequestScope::enter(Some(&request));
    let empty = HostPaintCommand::text(
        rect,
        None,
        1,
        "".into(),
        [255; 4],
        17.0,
        8.0,
        UiTextRunPaintStyle::default(),
        1.0,
    );
    assert!(draw_text_command(&mut pixels, &empty));
    assert_eq!(active_authored_font_request(), Some(request));
}
