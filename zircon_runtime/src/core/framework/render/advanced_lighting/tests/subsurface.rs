use super::*;

fn profile(profile_id: u32) -> SubsurfaceProfileData {
    SubsurfaceProfileData::new(
        profile_id,
        Vec3::new(0.8, 1.2, 1.8),
        Vec3::new(1.0, 0.45, 0.3),
        1.0,
    )
}

#[test]
fn render_sss_burley_kernel_integrates_to_one() {
    for scatter_radius in [0.25, 1.0, 8.0] {
        let interval_count = 200_000;
        let max_radius = scatter_radius * 48.0;
        let step = max_radius / interval_count as Real;
        let integral = (0..interval_count)
            .map(|index| {
                let radius = (index as Real + 0.5) * step;
                f64::from(burley_radial_pdf(radius, scatter_radius)) * f64::from(step)
            })
            .sum::<f64>();
        assert!(
            (integral - 1.0).abs() < 2.0e-4_f64,
            "radius {scatter_radius} integrated to {integral}"
        );
    }
}

#[test]
fn render_sss_profile_table_caps_at_16() {
    let profiles = (0..20).map(profile).collect::<Vec<_>>();

    let table = resolve_subsurface_profile_table(&profiles);

    assert_eq!(table.profiles.len(), ZR_SSS_MAX_PROFILES);
    assert_eq!(table.profiles.last().unwrap().profile_id, 15);
    assert_eq!(table.active_profile_mask, u32::from(u16::MAX));
    assert_eq!(table.diagnostics.len(), 4);
    assert_eq!(table.diagnostics[0].profile_id, 16);
    assert!(table.diagnostics[0]
        .message
        .contains("16-profile GPU table"));
}

#[test]
fn render_sss_sparse_profile_id_maps_to_matching_gpu_slot() {
    let table = resolve_subsurface_profile_table(&[profile(7)]);

    assert_eq!(table.profiles.len(), 8);
    assert_eq!(table.profiles[7].profile_id, 7);
    assert_eq!(
        table.profiles[7].scatter_radius_rgb,
        profile(7).scatter_radius_rgb
    );
    assert_eq!(table.profiles[0].scatter_radius_rgb, Vec3::ZERO);
    assert!(table.profile_is_active(7));
    assert!(!table.profile_is_active(0));
}

#[test]
fn render_sss_duplicate_profile_id_reports_diagnostic_and_keeps_first_slot() {
    let first = profile(3);
    let mut duplicate = profile(3);
    duplicate.scatter_radius_rgb = Vec3::splat(99.0);

    let table = resolve_subsurface_profile_table(&[first, duplicate]);

    assert_eq!(table.profiles[3], first);
    assert_eq!(table.diagnostics.len(), 1);
    assert!(table.diagnostics[0].message.contains("duplicates"));
}

#[test]
fn render_sss_burley_kernel_rejects_invalid_radius_contracts() {
    assert_eq!(burley_radial_pdf(-1.0, 1.0), 0.0);
    assert_eq!(burley_radial_pdf(1.0, 0.0), 0.0);
    assert_eq!(burley_radial_pdf(1.0, Real::NAN), 0.0);
}
