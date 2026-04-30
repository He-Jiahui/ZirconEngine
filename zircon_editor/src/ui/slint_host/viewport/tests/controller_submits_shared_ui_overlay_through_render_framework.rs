use std::sync::Arc;

use crate::ui::slint_host::viewport::tests::fake_render_framework::FakeRenderFramework;
use crate::ui::slint_host::viewport::tests::test_extract::test_extract;
use crate::ui::slint_host::viewport::SlintViewportController;
use zircon_runtime::core::math::UVec2;
use zircon_runtime::ui::event_ui::{UiNodeId, UiTreeId};
use zircon_runtime::ui::layout::UiFrame;
use zircon_runtime::ui::surface::{
    UiRenderCommand, UiRenderCommandKind, UiRenderExtract, UiRenderList, UiResolvedStyle,
    UiTextAlign, UiTextRenderMode, UiTextWrap,
};

#[test]
fn controller_submits_shared_ui_overlay_through_render_framework() {
    let framework = Arc::new(FakeRenderFramework::default());
    let controller = SlintViewportController::new_with_framework(framework.clone());

    controller
        .submit_extract_with_ui(
            test_extract(),
            Some(test_ui_extract("Viewport HUD")),
            UVec2::new(300, 150),
        )
        .unwrap();

    let state = framework.state.lock().unwrap();
    assert_eq!(state.submitted_ui_command_counts, vec![1]);
}

fn test_ui_extract(text: &str) -> UiRenderExtract {
    UiRenderExtract {
        tree_id: UiTreeId::new("editor.viewport.test"),
        list: UiRenderList {
            commands: vec![UiRenderCommand {
                node_id: UiNodeId::new(9),
                kind: UiRenderCommandKind::Quad,
                frame: UiFrame::new(8.0, 8.0, 160.0, 24.0),
                clip_frame: None,
                z_index: 0,
                style: UiResolvedStyle {
                    background_color: Some("#16202ccc".to_string()),
                    foreground_color: Some("#eef3ff".to_string()),
                    font: Some("res://fonts/default.font.toml".to_string()),
                    font_size: 13.0,
                    line_height: 16.0,
                    text_align: UiTextAlign::Center,
                    wrap: UiTextWrap::None,
                    text_render_mode: UiTextRenderMode::Auto,
                    ..UiResolvedStyle::default()
                },
                text_layout: None,
                text: Some(text.to_string()),
                image: None,
                opacity: 1.0,
            }],
        },
    }
}
