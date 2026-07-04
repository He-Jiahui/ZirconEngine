use super::{SampledEquirectangularSamples, EMPTY_SAMPLED_EQUIRECT_ENVIRONMENT_SAMPLES};
use crate::core::math::{Real, Vec4};

pub const PROCEDURAL_SKY_DEFAULT_SOURCE_REVISION: u64 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct IblBakeKey {
    pub source_kind: u32,
    pub source_revision: u64,
    pub horizon_color: [u32; 4],
    pub zenith_color: [u32; 4],
    pub ground_color: [u32; 4],
    pub sampled_environment_hash: [u32; 4],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProceduralSkyParams {
    pub horizon_color: Vec4,
    pub zenith_color: Vec4,
    pub ground_color: Vec4,
    pub intensity: Real,
    pub rotation_radians: Real,
    pub source_revision: u64,
}

impl ProceduralSkyParams {
    pub fn default_gradient() -> Self {
        Self {
            horizon_color: Vec4::new(0.16, 0.19, 0.24, 1.0),
            zenith_color: Vec4::new(0.36, 0.46, 0.63, 1.0),
            ground_color: Vec4::new(0.09, 0.11, 0.14, 1.0),
            intensity: 1.0,
            rotation_radians: 0.0,
            source_revision: PROCEDURAL_SKY_DEFAULT_SOURCE_REVISION,
        }
    }

    pub fn ibl_bake_key(&self) -> IblBakeKey {
        IblBakeKey {
            source_kind: SkyboxMode::ProceduralGradient.source_kind(),
            source_revision: self.source_revision,
            horizon_color: vec4_bits(self.horizon_color),
            zenith_color: vec4_bits(self.zenith_color),
            ground_color: vec4_bits(self.ground_color),
            sampled_environment_hash: [0; 4],
        }
    }
}

impl Default for ProceduralSkyParams {
    fn default() -> Self {
        Self::default_gradient()
    }
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkyboxMode {
    Disabled = 0,
    ProceduralGradient = 1,
    SampledEquirectangular = 2,
}

impl SkyboxMode {
    fn source_kind(self) -> u32 {
        match self {
            Self::Disabled => 0,
            Self::ProceduralGradient => 1,
            Self::SampledEquirectangular => 2,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SampledEquirectangularEnvironment {
    pub samples: SampledEquirectangularSamples,
    pub intensity: Real,
    pub rotation_radians: Real,
    pub source_revision: u64,
    pub source_hash: [u32; 4],
}

impl SampledEquirectangularEnvironment {
    pub fn new(
        samples: SampledEquirectangularSamples,
        source_revision: u64,
        source_hash: [u32; 4],
    ) -> Self {
        Self {
            samples,
            intensity: 1.0,
            rotation_radians: 0.0,
            source_revision,
            source_hash,
        }
    }

    pub fn ibl_bake_key(&self) -> IblBakeKey {
        IblBakeKey {
            source_kind: SkyboxMode::SampledEquirectangular.source_kind(),
            source_revision: self.source_revision,
            horizon_color: [0; 4],
            zenith_color: [0; 4],
            ground_color: [0; 4],
            sampled_environment_hash: self.source_hash,
        }
    }
}

impl Default for SampledEquirectangularEnvironment {
    fn default() -> Self {
        Self {
            samples: EMPTY_SAMPLED_EQUIRECT_ENVIRONMENT_SAMPLES,
            intensity: 1.0,
            rotation_radians: 0.0,
            source_revision: 0,
            source_hash: [0; 4],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SkyboxSettings {
    pub mode: SkyboxMode,
    pub procedural: ProceduralSkyParams,
    pub sampled_equirectangular: SampledEquirectangularEnvironment,
}

impl SkyboxSettings {
    pub fn none() -> Self {
        Self {
            mode: SkyboxMode::Disabled,
            procedural: ProceduralSkyParams::default_gradient(),
            sampled_equirectangular: SampledEquirectangularEnvironment::default(),
        }
    }

    pub fn procedural_default() -> Self {
        Self {
            mode: SkyboxMode::ProceduralGradient,
            procedural: ProceduralSkyParams::default_gradient(),
            sampled_equirectangular: SampledEquirectangularEnvironment::default(),
        }
    }

    pub fn sampled_equirectangular(sampled: SampledEquirectangularEnvironment) -> Self {
        Self {
            mode: SkyboxMode::SampledEquirectangular,
            procedural: ProceduralSkyParams::default_gradient(),
            sampled_equirectangular: sampled,
        }
    }

    pub fn is_enabled(&self) -> bool {
        !matches!(self.mode, SkyboxMode::Disabled)
    }

    pub fn intensity(&self) -> Real {
        match self.mode {
            SkyboxMode::Disabled => 0.0,
            SkyboxMode::ProceduralGradient => self.procedural.intensity,
            SkyboxMode::SampledEquirectangular => self.sampled_equirectangular.intensity,
        }
    }

    pub fn rotation_radians(&self) -> Real {
        match self.mode {
            SkyboxMode::Disabled => 0.0,
            SkyboxMode::ProceduralGradient => self.procedural.rotation_radians,
            SkyboxMode::SampledEquirectangular => self.sampled_equirectangular.rotation_radians,
        }
    }

    pub fn ibl_bake_key(&self) -> Option<IblBakeKey> {
        match self.mode {
            SkyboxMode::Disabled => None,
            SkyboxMode::ProceduralGradient => Some(self.procedural.ibl_bake_key()),
            SkyboxMode::SampledEquirectangular => Some(self.sampled_equirectangular.ibl_bake_key()),
        }
    }

    pub fn sampled_equirectangular_samples(&self) -> &SampledEquirectangularSamples {
        match self.mode {
            SkyboxMode::SampledEquirectangular => &self.sampled_equirectangular.samples,
            SkyboxMode::Disabled | SkyboxMode::ProceduralGradient => {
                &EMPTY_SAMPLED_EQUIRECT_ENVIRONMENT_SAMPLES
            }
        }
    }
}

impl Default for SkyboxSettings {
    fn default() -> Self {
        Self::none()
    }
}

fn vec4_bits(value: Vec4) -> [u32; 4] {
    [
        value.x.to_bits(),
        value.y.to_bits(),
        value.z.to_bits(),
        value.w.to_bits(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::framework::render::SAMPLED_EQUIRECT_ENVIRONMENT_SAMPLE_COUNT;

    #[test]
    fn procedural_default_matches_existing_preview_gradient() {
        let skybox = SkyboxSettings::procedural_default();

        assert_eq!(skybox.mode, SkyboxMode::ProceduralGradient);
        assert_eq!(
            skybox.procedural.horizon_color,
            Vec4::new(0.16, 0.19, 0.24, 1.0)
        );
        assert_eq!(
            skybox.procedural.zenith_color,
            Vec4::new(0.36, 0.46, 0.63, 1.0)
        );
        assert_eq!(
            skybox.procedural.ground_color,
            Vec4::new(0.09, 0.11, 0.14, 1.0)
        );
    }

    #[test]
    fn disabled_skybox_has_no_ibl_bake_key() {
        assert!(SkyboxSettings::none().ibl_bake_key().is_none());
    }

    #[test]
    fn ibl_bake_key_ignores_intensity_and_rotation() {
        let mut first = ProceduralSkyParams::default_gradient();
        let mut second = first;
        second.intensity = 3.5;
        second.rotation_radians = 1.25;

        assert_eq!(first.ibl_bake_key(), second.ibl_bake_key());

        first.horizon_color.x += 0.01;
        assert_ne!(first.ibl_bake_key(), second.ibl_bake_key());
    }

    #[test]
    fn ibl_bake_key_tracks_source_revision() {
        let first = ProceduralSkyParams::default_gradient();
        let mut second = first;
        second.source_revision += 1;

        assert_ne!(first.ibl_bake_key(), second.ibl_bake_key());
    }

    #[test]
    fn sampled_equirectangular_bake_key_tracks_source_hash() {
        let samples = [[0.25, 0.5, 0.75, 1.0]; SAMPLED_EQUIRECT_ENVIRONMENT_SAMPLE_COUNT];
        let first = SampledEquirectangularEnvironment::new(samples, 1, [1, 2, 3, 4]);
        let second = SampledEquirectangularEnvironment::new(samples, 1, [1, 2, 3, 5]);

        assert_ne!(first.ibl_bake_key(), second.ibl_bake_key());
        assert_eq!(
            SkyboxSettings::sampled_equirectangular(first).ibl_bake_key(),
            Some(first.ibl_bake_key())
        );
    }
}
