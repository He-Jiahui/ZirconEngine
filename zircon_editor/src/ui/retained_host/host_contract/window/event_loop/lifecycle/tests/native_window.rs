use super::*;

#[test]
fn requested_host_window_size_stays_in_physical_pixels() {
    let attributes = native_window_attributes_for_size(
        &PhysicalSize::new(1672, 941),
        WinitPhysicalSize::new(720, 480),
    );

    assert_eq!(
        attributes.surface_size,
        Some(Size::Physical(WinitPhysicalSize::new(1672, 941)))
    );
    assert_eq!(
        attributes.min_surface_size,
        Some(Size::Physical(WinitPhysicalSize::new(720, 480)))
    );
}

#[test]
fn profile_capture_initial_client_size_is_an_exact_physical_extent() {
    assert_eq!(
        parse_profile_initial_client_size(Some("1672"), Some("941")),
        Some(PhysicalSize::new(1672, 941))
    );
    assert_eq!(
        parse_profile_initial_client_size(Some("0"), Some("941")),
        None
    );
    assert_eq!(parse_profile_initial_client_size(Some("640"), None), None);
    assert_eq!(
        parse_profile_initial_client_size(Some("logical"), Some("520")),
        None
    );
}
