use serde::{Deserialize, Serialize};

use crate::core::framework::render::{
    RenderLayerSet, RenderResolvedPostProcessSettings, ShaderQualityTier,
    VolumeComponentApplyError, VolumeComponentDescriptor, VolumeParamSchema, VolumeParamValue,
    interp_bool, interp_float_lerp, interp_vec3_lerp,
};
use crate::core::math::{Real, Vec3};

pub const VOLUMETRIC_FOG_COMPONENT_ID: &str = "lighting.volumetric-fog";

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct VolumetricFogSettings {
    pub density: Real,
    pub albedo: Vec3,
    pub phase_g: Real,
    pub height_falloff: Real,
    pub scattering_intensity: Real,
    pub depth_distribution_exp: Real,
    pub temporal: bool,
}

impl VolumetricFogSettings {
    pub const DEFAULT: Self = Self {
        density: 0.02,
        albedo: Vec3::ONE,
        phase_g: 0.2,
        height_falloff: 0.1,
        scattering_intensity: 1.0,
        depth_distribution_exp: 2.0,
        temporal: true,
    };

    pub fn sanitized(self) -> Self {
        Self {
            density: finite_non_negative(self.density),
            albedo: self.albedo.max(Vec3::ZERO),
            phase_g: finite_or(self.phase_g, Self::DEFAULT.phase_g).clamp(-0.9, 0.9),
            height_falloff: finite_non_negative(self.height_falloff),
            scattering_intensity: finite_non_negative(self.scattering_intensity),
            depth_distribution_exp: finite_or(
                self.depth_distribution_exp,
                Self::DEFAULT.depth_distribution_exp,
            )
            .max(0.01),
            temporal: self.temporal,
        }
    }
}

impl Default for VolumetricFogSettings {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FogVolumeData {
    pub volume_id: u64,
    pub bounds_min: Vec3,
    pub bounds_max: Vec3,
    pub density: Real,
    pub albedo: Vec3,
    #[serde(default)]
    pub layer_mask: RenderLayerSet,
}

impl FogVolumeData {
    pub fn contains(&self, world_position: Vec3) -> bool {
        world_position.cmpge(self.bounds_min).all() && world_position.cmple(self.bounds_max).all()
    }

    pub fn sanitized_density(&self) -> Real {
        finite_non_negative(self.density)
    }

    pub fn sanitized_albedo(&self) -> Vec3 {
        self.albedo.max(Vec3::ZERO)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum FroxelGridQuality {
    Low,
    #[default]
    Medium,
    High,
}

impl FroxelGridQuality {
    pub const fn from_shader_quality(quality: ShaderQualityTier) -> Self {
        match quality {
            ShaderQualityTier::Low => Self::Low,
            ShaderQualityTier::Medium => Self::Medium,
            ShaderQualityTier::High | ShaderQualityTier::Ultra => Self::High,
        }
    }

    pub const fn dimensions(self) -> [u32; 3] {
        match self {
            Self::Low => [160, 90, 48],
            Self::Medium => [160, 90, 64],
            Self::High => [160, 90, 96],
        }
    }

    pub const fn supports_local_volumes(self) -> bool {
        !matches!(self, Self::Low)
    }

    pub const fn supports_temporal(self) -> bool {
        matches!(self, Self::High)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct FroxelGridParams {
    pub dimensions: [u32; 3],
    pub near_depth: Real,
    pub far_depth: Real,
    pub depth_distribution_exp: Real,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VolumetricIntegrationStep {
    pub radiance: Vec3,
    pub transmittance: Real,
}

pub fn henyey_greenstein_phase(phase_g: Real, cos_theta: Real) -> Real {
    let phase_g = finite_or(phase_g, 0.0).clamp(-0.9, 0.9);
    let cos_theta = finite_or(cos_theta, 0.0).clamp(-1.0, 1.0);
    let denominator = (1.0 + phase_g * phase_g - 2.0 * phase_g * cos_theta).max(0.0001);
    (1.0 - phase_g * phase_g) / (4.0 * std::f32::consts::PI * denominator * denominator.sqrt())
}

pub fn integrate_volumetric_step(
    scattering: Vec3,
    extinction: Real,
    step_length: Real,
) -> VolumetricIntegrationStep {
    let scattering = scattering.max(Vec3::ZERO);
    let extinction = finite_non_negative(extinction);
    let step_length = finite_non_negative(step_length);
    let optical_depth = extinction * step_length;
    let transmittance = (-optical_depth).exp();
    let radiance_scale = if extinction > 0.000001 {
        (1.0 - transmittance) / extinction
    } else {
        step_length
    };
    VolumetricIntegrationStep {
        radiance: scattering * radiance_scale,
        transmittance,
    }
}

impl FroxelGridParams {
    pub fn for_quality(
        quality: FroxelGridQuality,
        near_depth: Real,
        far_depth: Real,
        depth_distribution_exp: Real,
    ) -> Self {
        Self {
            dimensions: quality.dimensions(),
            near_depth,
            far_depth,
            depth_distribution_exp,
        }
        .sanitized()
    }

    pub fn sanitized(self) -> Self {
        let near_depth = finite_or(self.near_depth, 0.1).max(0.0001);
        let far_depth = finite_or(self.far_depth, near_depth + 1.0).max(near_depth + 0.0001);
        Self {
            dimensions: self.dimensions.map(|extent| extent.max(1)),
            near_depth,
            far_depth,
            depth_distribution_exp: finite_or(self.depth_distribution_exp, 2.0).max(0.01),
        }
    }

    pub fn slice_depth(self, slice_index: u32) -> Real {
        let params = self.sanitized();
        let slice_count = params.dimensions[2];
        let clamped_slice = slice_index.min(slice_count - 1);
        let normalized = ((clamped_slice as Real + 0.5) / slice_count as Real)
            .powf(params.depth_distribution_exp);
        params.near_depth * (params.far_depth / params.near_depth).powf(normalized)
    }
}

const VOLUMETRIC_FOG_PARAMS: [VolumeParamSchema; 7] = [
    VolumeParamSchema::new(
        "density",
        VolumeParamValue::Float(VolumetricFogSettings::DEFAULT.density),
        interp_float_lerp,
    ),
    VolumeParamSchema::new(
        "albedo",
        VolumeParamValue::Vec3(VolumetricFogSettings::DEFAULT.albedo),
        interp_vec3_lerp,
    ),
    VolumeParamSchema::new(
        "phase_g",
        VolumeParamValue::Float(VolumetricFogSettings::DEFAULT.phase_g),
        interp_float_lerp,
    ),
    VolumeParamSchema::new(
        "height_falloff",
        VolumeParamValue::Float(VolumetricFogSettings::DEFAULT.height_falloff),
        interp_float_lerp,
    ),
    VolumeParamSchema::new(
        "scattering_intensity",
        VolumeParamValue::Float(VolumetricFogSettings::DEFAULT.scattering_intensity),
        interp_float_lerp,
    ),
    VolumeParamSchema::new(
        "depth_distribution_exp",
        VolumeParamValue::Float(VolumetricFogSettings::DEFAULT.depth_distribution_exp),
        interp_float_lerp,
    ),
    VolumeParamSchema::new(
        "temporal",
        VolumeParamValue::Bool(VolumetricFogSettings::DEFAULT.temporal),
        interp_bool,
    ),
];

pub const VOLUMETRIC_FOG_VOLUME_COMPONENT: VolumeComponentDescriptor =
    VolumeComponentDescriptor::new(
        VOLUMETRIC_FOG_COMPONENT_ID,
        &VOLUMETRIC_FOG_PARAMS,
        read_volumetric_fog,
        apply_volumetric_fog,
    );

fn read_volumetric_fog(settings: &RenderResolvedPostProcessSettings) -> Vec<VolumeParamValue> {
    let value = settings.volumetric_fog;
    vec![
        VolumeParamValue::Float(value.density),
        VolumeParamValue::Vec3(value.albedo),
        VolumeParamValue::Float(value.phase_g),
        VolumeParamValue::Float(value.height_falloff),
        VolumeParamValue::Float(value.scattering_intensity),
        VolumeParamValue::Float(value.depth_distribution_exp),
        VolumeParamValue::Bool(value.temporal),
    ]
}

fn apply_volumetric_fog(
    settings: &mut RenderResolvedPostProcessSettings,
    component_id: &'static str,
    values: &[VolumeParamValue],
) -> Result<(), VolumeComponentApplyError> {
    settings.volumetric_fog = VolumetricFogSettings {
        density: values[0].float(component_id, "density")?,
        albedo: values[1].vec3(component_id, "albedo")?,
        phase_g: values[2].float(component_id, "phase_g")?,
        height_falloff: values[3].float(component_id, "height_falloff")?,
        scattering_intensity: values[4].float(component_id, "scattering_intensity")?,
        depth_distribution_exp: values[5].float(component_id, "depth_distribution_exp")?,
        temporal: values[6].bool(component_id, "temporal")?,
    }
    .sanitized();
    Ok(())
}

fn finite_non_negative(value: Real) -> Real {
    finite_or(value, 0.0).max(0.0)
}

fn finite_or(value: Real, fallback: Real) -> Real {
    if value.is_finite() { value } else { fallback }
}

#[cfg(test)]
mod tests;
