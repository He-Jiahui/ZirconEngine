use super::*;
use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
#[test]
fn page_background_cannot_cover_the_published_brand_at_any_scale() {
    use crate::ui::retained_host::host_contract::data::TemplateNodeFrameData;
    use crate::ui::retained_host::host_contract::paint_frame::HostRecordedPaintKind;

    for scale in [1.0, 1.5] {
        let top = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 1280.0 * scale,
            height: 59.0 * scale,
        };
        let page_band = FrameRect {
            x: 0.0,
            y: 33.0 * scale,
            width: top.width,
            height: 26.0 * scale,
        };
        let mut presentation = HostWindowPresentationData::default();
        presentation.host_scene_data.menu_chrome.template_nodes =
            model_rc(vec![TemplatePaneNodeData {
                control_id: "WorkbenchBrandLabel".into(),
                role: "Text".into(),
                text: "Zircon Workbench".into(),
                font_size: 12.0 * scale,
                frame: TemplateNodeFrameData {
                    x: 34.0 * scale,
                    y: 2.0 * scale,
                    width: 140.0 * scale,
                    height: 28.0 * scale,
                },
                ..Default::default()
            }]);
        presentation.host_scene_data.page_chrome.tab_row_frame = page_band.clone();
        // This is the opaque compiled root including the menu spacer.
        presentation.host_scene_data.page_chrome.template_nodes =
            model_rc(vec![TemplatePaneNodeData {
                control_id: "WorkbenchPageChromeRoot".into(),
                role: "Panel".into(),
                surface_variant: "panel".into(),
                frame: TemplateNodeFrameData {
                    x: top.x,
                    y: top.y,
                    width: top.width,
                    height: top.height,
                },
                ..Default::default()
            }]);
        let root = RootFrames {
            top_bar: top.clone(),
            center_band: FrameRect::default(),
            status_bar: FrameRect::default(),
            left_region: FrameRect::default(),
            right_region: FrameRect::default(),
            bottom_region: FrameRect::default(),
            document_region: FrameRect::default(),
            viewport_region: FrameRect::default(),
        };
        let mut frame =
            HostRgbaFrame::recording_only((1280.0 * scale) as u32, (800.0 * scale) as u32);
        draw_top_chrome_layers(&mut frame, &root, &presentation);
        let commands = frame.into_recorded_commands();
        assert!(commands.iter().any(|command| matches!(&command.kind, HostRecordedPaintKind::Text { text, .. } if text == "Zircon Workbench")));
        let page_background = commands
            .iter()
            .find(|command| {
                command.frame == top && matches!(command.kind, HostRecordedPaintKind::Quad { .. })
            })
            .expect("compiled page root paints its background");
        assert_eq!(page_background.clip_frame.as_ref(), Some(&page_band));
        assert_eq!(
            presentation.host_scene_data.page_chrome.tab_row_frame,
            page_band
        );
    }
}

#[test]
fn collapsed_or_outside_page_band_does_not_revive_menu_paint_and_legacy_clip_remains() {
    let top = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 800.0,
        height: 59.0,
    };
    let mut page = HostPageChromeData::default();
    assert_eq!(page_chrome_paint_clip(&top, &page), Some(top.clone()));
    page.template_nodes = model_rc(vec![TemplatePaneNodeData {
        control_id: "WorkbenchPageBar".into(),
        ..Default::default()
    }]);
    assert!(page_chrome_paint_clip(&top, &page).is_none());
    page.tab_row_frame = FrameRect {
        x: 0.0,
        y: 80.0,
        width: 800.0,
        height: 26.0,
    };
    assert!(page_chrome_paint_clip(&top, &page).is_none());
    page.tab_row_frame = FrameRect {
        x: 0.0,
        y: 33.0,
        width: 800.0,
        height: 26.0,
    };
    assert_eq!(
        page_chrome_paint_clip(&top, &page),
        Some(page.tab_row_frame.clone())
    );
}
