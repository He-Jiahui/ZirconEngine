use serde::{Deserialize, Serialize};

use crate::core::math::{Real, Vec3};

pub const ZR_SSS_MAX_PROFILES: usize = 16;
pub const ZR_SSS_BURLEY_SAMPLE_COUNT: u32 = 64;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SubsurfaceProfileData {
    pub profile_id: u32,
    /// RGB mean-free path in millimetres.
    pub scatter_radius_rgb: Vec3,
    /// Per-channel tint applied to scattered diffuse lighting.
    pub falloff_rgb: Vec3,
    /// Converts the authored millimetre radius to the current world scale.
    pub world_unit_scale: Real,
}

impl SubsurfaceProfileData {
    pub const fn new(
        profile_id: u32,
        scatter_radius_rgb: Vec3,
        falloff_rgb: Vec3,
        world_unit_scale: Real,
    ) -> Self {
        Self {
            profile_id,
            scatter_radius_rgb,
            falloff_rgb,
            world_unit_scale,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubsurfaceProfileDiagnostic {
    pub profile_id: u32,
    pub message: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SubsurfaceProfileTable {
    pub profiles: Vec<SubsurfaceProfileData>,
    pub active_profile_mask: u32,
    pub diagnostics: Vec<SubsurfaceProfileDiagnostic>,
}

impl SubsurfaceProfileTable {
    pub const fn profile_is_active(&self, profile_id: u32) -> bool {
        profile_id < ZR_SSS_MAX_PROFILES as u32
            && (self.active_profile_mask & (1_u32 << profile_id)) != 0
    }
}

pub fn resolve_subsurface_profile_table(
    profiles: &[SubsurfaceProfileData],
) -> SubsurfaceProfileTable {
    let mut slots = [None; ZR_SSS_MAX_PROFILES];
    let mut active_profile_mask = 0_u32;
    let mut diagnostics = Vec::with_capacity(subsurface_diagnostic_capacity(profiles.len()));
    for profile in profiles {
        let Ok(slot) = usize::try_from(profile.profile_id) else {
            continue;
        };
        if slot >= ZR_SSS_MAX_PROFILES {
            diagnostics.push(SubsurfaceProfileDiagnostic {
                profile_id: profile.profile_id,
                message: format!(
                    "subsurface profile {} exceeds the {}-profile GPU table and was ignored",
                    profile.profile_id, ZR_SSS_MAX_PROFILES
                ),
            });
            continue;
        }
        if slots[slot].is_some() {
            diagnostics.push(SubsurfaceProfileDiagnostic {
                profile_id: profile.profile_id,
                message: format!(
                    "subsurface profile {} duplicates an occupied GPU slot and was ignored",
                    profile.profile_id
                ),
            });
            continue;
        }
        slots[slot] = Some(*profile);
        active_profile_mask |= 1_u32 << profile.profile_id;
    }
    let table_len = slots
        .iter()
        .rposition(Option::is_some)
        .map_or(0, |slot| slot + 1);
    let profiles = slots[..table_len]
        .iter()
        .enumerate()
        .map(|(slot, profile)| {
            profile.unwrap_or_else(|| {
                SubsurfaceProfileData::new(slot as u32, Vec3::ZERO, Vec3::ZERO, 0.0)
            })
        })
        .collect();
    SubsurfaceProfileTable {
        profiles,
        active_profile_mask,
        diagnostics,
    }
}

fn subsurface_diagnostic_capacity(profile_count: usize) -> usize {
    if profile_count > ZR_SSS_MAX_PROFILES {
        profile_count
    } else {
        0
    }
}

/// Radial probability density `2*pi*r*R(r)` of the normalized Burley profile.
/// Integrating this function over `[0, infinity)` yields one.
pub fn burley_radial_pdf(radius_mm: Real, scatter_radius_mm: Real) -> Real {
    if !radius_mm.is_finite()
        || !scatter_radius_mm.is_finite()
        || radius_mm < 0.0
        || scatter_radius_mm <= 0.0
    {
        return 0.0;
    }
    let radius = radius_mm / scatter_radius_mm;
    ((-radius).exp() + (-radius / 3.0).exp()) / (4.0 * scatter_radius_mm)
}

#[cfg(test)]
mod tests {
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
}

#[cfg(test)]
#[path = "subsurface/diagnostic_capacity_tests.rs"]
mod diagnostic_capacity_tests;
