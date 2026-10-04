use super::{ScreenSpaceTransmissionSettings, MAX_SCREEN_SPACE_TRANSMISSION_STEPS};

#[test]
fn render_screen_space_transmission_settings_normalize_step_budget() {
    assert_eq!(ScreenSpaceTransmissionSettings::default().steps(), 1);
    assert_eq!(ScreenSpaceTransmissionSettings::new(0).steps(), 0);
    assert_eq!(
        ScreenSpaceTransmissionSettings::new(usize::MAX).steps(),
        MAX_SCREEN_SPACE_TRANSMISSION_STEPS
    );
}
