use zircon_runtime_interface::ui::dispatch::UiInputSequence;

use super::PlatformInputTranslation;

#[test]
fn dpi_events_translate_with_the_applied_window_metrics() {
    let source = include_str!("../platform_input.rs");
    let metrics = source
        .find(".with_window_metrics(self.current_window_metrics())")
        .expect("platform events should retain the current DPI metrics");
    let translate = source
        .find("translate_winit_window_event(context, event)")
        .expect("platform events should use the shared Winit translator");

    assert!(metrics < translate);
    assert!(source.contains("WindowEvent::SurfaceResized(_)"));
    assert!(source.contains("WindowEvent::ScaleFactorChanged { .. }"));
}

#[test]
fn untranslated_platform_input_retains_its_assigned_sequence() {
    let translation = PlatformInputTranslation {
        sequence: UiInputSequence::new(41),
        event: None,
    };

    assert_eq!(translation.sequence, UiInputSequence::new(41));
    assert!(translation.event.is_none());
}
