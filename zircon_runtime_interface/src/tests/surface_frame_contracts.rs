use crate::ui::surface::UiSurfaceWindowState;

fn round_trip<T>(value: &T) -> T
where
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    serde_json::from_str(&serde_json::to_string(value).unwrap()).unwrap()
}

#[test]
fn ui_surface_window_state_round_trips_lifecycle_facts_with_defaults() {
    let state = UiSurfaceWindowState {
        focused: Some(false),
        application_active: Some(true),
        occluded: Some(false),
        close_requested: true,
        closed: false,
        destroyed: true,
        ..Default::default()
    };

    assert_eq!(round_trip(&state), state);

    let legacy_state: UiSurfaceWindowState = serde_json::from_str("{}").unwrap();
    assert_eq!(legacy_state.focused, None);
    assert_eq!(legacy_state.application_active, None);
    assert_eq!(legacy_state.occluded, None);
    assert!(!legacy_state.close_requested);
    assert!(!legacy_state.closed);
    assert!(!legacy_state.destroyed);
}
