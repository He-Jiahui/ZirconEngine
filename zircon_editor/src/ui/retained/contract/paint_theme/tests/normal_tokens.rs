use super::*;
use crate::ui::retained_host::host_contract::paint_theme::{
    capture_host_paint_theme_snapshot, enter_host_paint_theme_scope,
    host_paint_theme_snapshot_from_tokens_for_test, HostPaintThemeSnapshot,
};

fn token_snapshot(
    id: &str,
    settings_generation: Option<u64>,
    theme_generation: u64,
) -> Arc<HostPaintThemeSnapshot> {
    let mut tokens = EditorDesignTokens::workbench_dark();
    tokens.id = id.into();
    Arc::new(HostPaintThemeSnapshot::from_tokens(
        Arc::new(tokens),
        settings_generation,
        theme_generation,
        1.0,
    ))
}

#[test]
fn token_input_requires_a_frame_held_theme() {
    let _authority = capture_host_paint_theme_snapshot();
    assert!(active_host_paint_token_input().is_none());
}

#[test]
fn nested_frames_keep_their_own_token_input_and_restore_the_previous_frame() {
    let first = token_snapshot("first-frame", Some(7), 13);
    let second = token_snapshot("second-frame", Some(8), 14);
    let _first = enter_host_paint_theme_scope(first);
    let observed_first = active_host_paint_token_input().unwrap();
    assert_eq!(observed_first["tokens"]["id"], "first-frame");
    assert_eq!(observed_first["settingsGeneration"], 7);
    assert_eq!(observed_first["themeGeneration"], 13);
    {
        let _second = enter_host_paint_theme_scope(second);
        let observed_second = active_host_paint_token_input().unwrap();
        assert_eq!(observed_second["tokens"]["id"], "second-frame");
        assert_eq!(observed_second["settingsGeneration"], 8);
        assert_eq!(observed_second["themeGeneration"], 14);
    }
    assert_eq!(active_host_paint_token_input().unwrap(), observed_first);
}

#[test]
fn dpi_projection_keeps_the_original_token_handle_and_settings_generation() {
    let original = token_snapshot("scaled-frame", Some(9), 20);
    let scaled = Arc::new(original.at_scale(1.5));
    let original_input = original.token_input.as_ref().unwrap();
    let scaled_input = scaled.token_input.as_ref().unwrap();
    assert!(Arc::ptr_eq(&original_input.tokens, &scaled_input.tokens));
    assert_eq!(scaled_input.settings_generation, Some(9));
    assert_eq!(scaled.generation(), 21);
    assert_eq!(
        scaled.metrics.row_height,
        original.base_metrics.row_height * 1.5
    );
    let _scope = enter_host_paint_theme_scope(scaled);
    let observed = active_host_paint_token_input().unwrap();
    assert_eq!(observed["scaleFactor"], 1.5);
    assert_eq!(
        observed["tokens"],
        serde_json::to_value(original_input.tokens.as_ref()).unwrap()
    );
}

#[test]
fn partial_theme_overrides_invalidate_full_token_lineage() {
    let original = token_snapshot("partial-override", Some(9), 20);
    let overrides = [
        original.with_metrics(original.base_metrics),
        original.with_palette(original.palette),
        original.with_text_preferences(Arc::clone(&original.text_preferences)),
    ];
    for overridden in overrides {
        assert_eq!(overridden.generation(), 21);
        let _scope = enter_host_paint_theme_scope(Arc::new(overridden));
        assert!(active_host_paint_token_input().is_none());
    }
}

#[test]
fn settings_less_capture_does_not_invent_a_settings_generation() {
    let tokens = EditorDesignTokens::workbench_dark();
    let snapshot = host_paint_theme_snapshot_from_tokens_for_test(&tokens);
    let _scope = enter_host_paint_theme_scope(snapshot);
    let observed = active_host_paint_token_input().unwrap();
    assert!(observed["settingsGeneration"].is_null());
    assert_eq!(observed["tokens"], serde_json::to_value(tokens).unwrap());
}
