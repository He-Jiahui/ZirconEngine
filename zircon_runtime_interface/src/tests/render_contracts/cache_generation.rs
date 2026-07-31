use crate::ui::{
    event_ui::UiNodeId,
    layout::UiFrame,
    surface::{UiRenderCommand, UiRenderCommandKind, UiResolvedStyle, UiVisualAssetRef},
};

#[test]
fn ui_render_command_cache_generation_streams_json_bytes_once_per_conversion() {
    let command = representative_command();
    let expected_generation = legacy_json_generation(&command);

    let single = command.to_paint_element(7);
    assert_eq!(single.cache_generation, Some(expected_generation));

    let elements = command.to_paint_elements(11);
    assert_eq!(elements.len(), 4);
    assert!(
        elements
            .iter()
            .all(|element| element.cache_generation == Some(expected_generation)),
        "every paint element from one command must share its precomputed generation"
    );

    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/ui/surface/render/command.rs"
    ));
    assert!(!source.contains("serde_json::to_vec("));
    assert!(source.contains("serde_json::to_writer(&mut writer, value)"));
    assert_eq!(
        source
            .matches("let cache_generation = self.cache_generation();")
            .count(),
        2,
        "single- and multi-element conversion must each compute generation once"
    );

    let base_start = source
        .find("fn base_paint_element(")
        .expect("base paint element helper");
    let cache_start = source
        .find("fn cache_generation(")
        .expect("cache generation helper");
    let base_source = &source[base_start..cache_start];
    assert!(base_source.contains("cache_generation: u64"));
    assert!(!base_source.contains("self.cache_generation()"));
}

fn representative_command() -> UiRenderCommand {
    UiRenderCommand {
        node_id: UiNodeId::new(178),
        kind: UiRenderCommandKind::Image,
        frame: UiFrame::new(4.0, 8.0, 96.0, 32.0),
        clip_frame: Some(UiFrame::new(0.0, 0.0, 128.0, 64.0)),
        z_index: 9,
        style: UiResolvedStyle {
            background_color: Some("#102030FF".to_string()),
            foreground_color: Some("#F0F4F8FF".to_string()),
            border_color: Some("#506070FF".to_string()),
            border_width: 1.5,
            corner_radius: 4.0,
            ..UiResolvedStyle::default()
        },
        text_layout: None,
        text: Some("Streaming generation".to_string()),
        image: Some(UiVisualAssetRef::Image(
            "textures/ui/generation.png".to_string(),
        )),
        opacity: 0.875,
    }
}

fn legacy_json_generation(command: &UiRenderCommand) -> u64 {
    let bytes = serde_json::to_vec(command).unwrap_or_default();
    bytes.iter().fold(0xcbf29ce484222325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
    })
}
