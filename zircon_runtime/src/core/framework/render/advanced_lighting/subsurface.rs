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

/// GPU 固定槽位与活跃位掩码的帧级映射；稀疏 profile_id 仍对应同编号槽位，供图编译和散射通道共享。
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

/// 将场景配置收敛为固定容量的 GPU profile 表；溢出和重复编号只记录诊断，先出现的槽位获胜。
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
#[path = "tests/subsurface.rs"]
mod tests;

#[cfg(test)]
#[path = "subsurface/tests/diagnostic_capacity_tests.rs"]
mod diagnostic_capacity_tests;
