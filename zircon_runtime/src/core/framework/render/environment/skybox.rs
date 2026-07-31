use super::{
    IblBakeArtifactContents, IblBakeArtifactRequest, SourceCubemapIrradianceCube,
    SourceCubemapIrradianceSh9, SourceCubemapMipChain,
};
use crate::core::math::{Real, Vec4};

pub const PROCEDURAL_SKY_DEFAULT_SOURCE_REVISION: u64 = 1;
const PROCEDURAL_SKY_DEFAULT_SUN_ANGULAR_RADIUS_RADIANS: Real = 0.004_65;
const PROCEDURAL_SKY_MIN_SUN_ANGULAR_RADIUS_RADIANS: Real = 0.001;
const PROCEDURAL_SKY_MAX_SUN_ANGULAR_RADIUS_RADIANS: Real = std::f32::consts::FRAC_PI_2;
const PROCEDURAL_SKY_SUN_INNER_RADIUS_SCALE: Real = 0.72;
const PROCEDURAL_SKY_MIN_SUN_DIRECTION_LENGTH_SQUARED: Real = 1.0e-12;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct IblBakeKey {
    pub source_kind: u32,
    pub source_revision: u64,
    pub horizon_color: [u32; 4],
    pub zenith_color: [u32; 4],
    pub ground_color: [u32; 4],
    pub source_hash: [u32; 4],
}

impl IblBakeKey {
    pub const fn source_cubemap(source_revision: u64, source_hash: [u32; 4]) -> Self {
        Self {
            source_kind: SkyboxMode::SourceCubemap as u32,
            source_revision,
            horizon_color: [0; 4],
            zenith_color: [0; 4],
            ground_color: [0; 4],
            source_hash,
        }
    }
}

/// Cached GPU-texture identity; full artifact provenance remains outside the frame upload path.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct SourceCubemapUploadKey {
    pub source_revision: u64,
    pub source_hash: [u32; 4],
    pub pmrem_hash: [u32; 4],
    pub irradiance_cube_hash: [u32; 4],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProceduralSkyParams {
    pub horizon_color: Vec4,
    pub zenith_color: Vec4,
    pub ground_color: Vec4,
    pub sun_direction: Vec4,
    pub sun_color: Vec4,
    pub sun_intensity: Real,
    pub sun_angular_radius_radians: Real,
    pub intensity: Real,
    pub rotation_radians: Real,
    pub source_revision: u64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct ResolvedProceduralSun {
    pub(crate) direction: Vec4,
    pub(crate) intensity_and_cosines: Vec4,
}

impl ResolvedProceduralSun {
    pub(crate) fn direction_for_sampling_rotation(self, rotation_radians: Real) -> Vec4 {
        if self.direction.w < 0.5 || rotation_radians == 0.0 || !rotation_radians.is_finite() {
            return self.direction;
        }
        let (sine, cosine) = rotation_radians.sin_cos();
        Vec4::new(
            self.direction.x * cosine + self.direction.z * sine,
            self.direction.y,
            -self.direction.x * sine + self.direction.z * cosine,
            1.0,
        )
    }
}

impl ProceduralSkyParams {
    pub fn default_gradient() -> Self {
        Self {
            horizon_color: Vec4::new(0.16, 0.19, 0.24, 1.0),
            zenith_color: Vec4::new(0.36, 0.46, 0.63, 1.0),
            ground_color: Vec4::new(0.09, 0.11, 0.14, 1.0),
            sun_direction: Vec4::new(0.0, 1.0, 0.0, 0.0),
            sun_color: Vec4::ONE,
            sun_intensity: 0.0,
            sun_angular_radius_radians: PROCEDURAL_SKY_DEFAULT_SUN_ANGULAR_RADIUS_RADIANS,
            intensity: 1.0,
            rotation_radians: 0.0,
            source_revision: PROCEDURAL_SKY_DEFAULT_SOURCE_REVISION,
        }
    }

    pub fn ibl_bake_key(&self) -> IblBakeKey {
        let sun = self.resolved_sun();
        IblBakeKey {
            source_kind: SkyboxMode::ProceduralGradient.source_kind(),
            source_revision: self.source_revision,
            horizon_color: vec4_bits(self.horizon_color),
            zenith_color: vec4_bits(self.zenith_color),
            ground_color: vec4_bits(self.ground_color),
            source_hash: procedural_sun_hash(self, sun),
        }
    }

    pub(crate) fn resolved_sun(&self) -> ResolvedProceduralSun {
        let intensity = if self.sun_intensity.is_finite() {
            self.sun_intensity.max(0.0)
        } else {
            0.0
        };
        let direction_x = f64::from(self.sun_direction.x);
        let direction_y = f64::from(self.sun_direction.y);
        let direction_z = f64::from(self.sun_direction.z);
        let direction_length_squared =
            direction_x * direction_x + direction_y * direction_y + direction_z * direction_z;
        if intensity <= 0.0
            || !direction_length_squared.is_finite()
            || direction_length_squared
                <= f64::from(PROCEDURAL_SKY_MIN_SUN_DIRECTION_LENGTH_SQUARED)
        {
            return ResolvedProceduralSun::default();
        }

        let inverse_direction_length = direction_length_squared.sqrt().recip();
        let normalized_direction = Vec4::new(
            (direction_x * inverse_direction_length) as Real,
            (direction_y * inverse_direction_length) as Real,
            (direction_z * inverse_direction_length) as Real,
            1.0,
        );
        let angular_radius = if self.sun_angular_radius_radians.is_finite() {
            self.sun_angular_radius_radians.clamp(
                PROCEDURAL_SKY_MIN_SUN_ANGULAR_RADIUS_RADIANS,
                PROCEDURAL_SKY_MAX_SUN_ANGULAR_RADIUS_RADIANS,
            )
        } else {
            PROCEDURAL_SKY_DEFAULT_SUN_ANGULAR_RADIUS_RADIANS
        };
        ResolvedProceduralSun {
            direction: normalized_direction,
            intensity_and_cosines: Vec4::new(
                intensity,
                angular_radius.cos(),
                (angular_radius * PROCEDURAL_SKY_SUN_INNER_RADIUS_SCALE).cos(),
                0.0,
            ),
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
    SourceCubemap = 3,
}

impl SkyboxMode {
    fn source_kind(self) -> u32 {
        match self {
            Self::Disabled => 0,
            Self::ProceduralGradient => 1,
            Self::SourceCubemap => 3,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SourceCubemapEnvironment {
    pub mip_chain: SourceCubemapMipChain,
    pub irradiance_sh9: SourceCubemapIrradianceSh9,
    pub irradiance_cube: Option<SourceCubemapIrradianceCube>,
    /// Cached identity of the PMREM section supplied by a bake artifact.
    pub pmrem_hash: [u32; 4],
    pub bake_artifact_hash: [u32; 4],
    pub intensity: Real,
    pub rotation_radians: Real,
    pub source_revision: u64,
    pub source_hash: [u32; 4],
}

impl SourceCubemapEnvironment {
    pub fn new(
        mip_chain: SourceCubemapMipChain,
        source_revision: u64,
        source_hash: [u32; 4],
    ) -> Self {
        let irradiance_sh9 = *mip_chain.irradiance_sh9();
        Self {
            mip_chain,
            irradiance_sh9,
            irradiance_cube: None,
            pmrem_hash: [0; 4],
            bake_artifact_hash: [0; 4],
            intensity: 1.0,
            rotation_radians: 0.0,
            source_revision,
            source_hash,
        }
    }

    pub fn with_irradiance_cube(mut self, irradiance_cube: SourceCubemapIrradianceCube) -> Self {
        self.irradiance_cube = Some(irradiance_cube);
        self
    }

    /// Records artifact provenance without changing GPU texture content identity.
    pub fn with_bake_artifact_hash(mut self, bake_artifact_hash: [u32; 4]) -> Self {
        self.bake_artifact_hash = bake_artifact_hash;
        self
    }

    pub fn irradiance_cube(&self) -> Option<&SourceCubemapIrradianceCube> {
        self.irradiance_cube.as_ref()
    }

    pub fn texture_upload_key(&self) -> SourceCubemapUploadKey {
        SourceCubemapUploadKey {
            source_revision: self.source_revision,
            source_hash: self.source_hash,
            pmrem_hash: self.pmrem_hash,
            irradiance_cube_hash: self
                .irradiance_cube
                .as_ref()
                .map_or([0; 4], SourceCubemapIrradianceCube::content_hash),
        }
    }

    pub fn ibl_bake_key(&self) -> IblBakeKey {
        IblBakeKey::source_cubemap(self.source_revision, self.source_hash)
    }

    pub fn ibl_bake_artifact_request(
        &self,
        required_contents: IblBakeArtifactContents,
    ) -> IblBakeArtifactRequest {
        IblBakeArtifactRequest::new(
            self.ibl_bake_key(),
            self.mip_chain.source_face_size(),
            self.mip_chain.source_mip_count(),
        )
        .with_pmrem_layout(
            self.mip_chain.pmrem_face_size(),
            self.mip_chain.pmrem_mip_count(),
        )
        .with_required_contents(required_contents)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SkyboxSettings {
    pub mode: SkyboxMode,
    pub procedural: ProceduralSkyParams,
    pub source_cubemap: Option<SourceCubemapEnvironment>,
}

impl SkyboxSettings {
    pub fn none() -> Self {
        Self {
            mode: SkyboxMode::Disabled,
            procedural: ProceduralSkyParams::default_gradient(),
            source_cubemap: None,
        }
    }

    pub fn procedural_default() -> Self {
        Self {
            mode: SkyboxMode::ProceduralGradient,
            procedural: ProceduralSkyParams::default_gradient(),
            source_cubemap: None,
        }
    }

    pub fn source_cubemap(source_cubemap: SourceCubemapEnvironment) -> Self {
        Self {
            mode: SkyboxMode::SourceCubemap,
            procedural: ProceduralSkyParams::default_gradient(),
            source_cubemap: Some(source_cubemap),
        }
    }

    pub fn is_enabled(&self) -> bool {
        !matches!(self.mode, SkyboxMode::Disabled)
    }

    pub fn intensity(&self) -> Real {
        match self.mode {
            SkyboxMode::Disabled => 0.0,
            SkyboxMode::ProceduralGradient => self.procedural.intensity,
            SkyboxMode::SourceCubemap => self
                .source_cubemap
                .as_ref()
                .map(|environment| environment.intensity)
                .unwrap_or(0.0),
        }
    }

    pub fn rotation_radians(&self) -> Real {
        match self.mode {
            SkyboxMode::Disabled => 0.0,
            SkyboxMode::ProceduralGradient => self.procedural.rotation_radians,
            SkyboxMode::SourceCubemap => self
                .source_cubemap
                .as_ref()
                .map(|environment| environment.rotation_radians)
                .unwrap_or(0.0),
        }
    }

    pub fn ibl_bake_key(&self) -> Option<IblBakeKey> {
        match self.mode {
            SkyboxMode::Disabled => None,
            SkyboxMode::ProceduralGradient => Some(self.procedural.ibl_bake_key()),
            SkyboxMode::SourceCubemap => self
                .source_cubemap
                .as_ref()
                .map(SourceCubemapEnvironment::ibl_bake_key),
        }
    }

    pub fn source_cubemap_environment(&self) -> Option<&SourceCubemapEnvironment> {
        match self.mode {
            SkyboxMode::SourceCubemap => self.source_cubemap.as_ref(),
            SkyboxMode::Disabled | SkyboxMode::ProceduralGradient => None,
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

fn procedural_sun_hash(params: &ProceduralSkyParams, sun: ResolvedProceduralSun) -> [u32; 4] {
    if sun.direction.w < 0.5 {
        return [0; 4];
    }

    let mut hasher = blake3::Hasher::new();
    for value in [
        sun.direction.x,
        sun.direction.y,
        sun.direction.z,
        params.sun_color.x,
        params.sun_color.y,
        params.sun_color.z,
        sun.intensity_and_cosines.x,
        sun.intensity_and_cosines.y,
        sun.intensity_and_cosines.z,
    ] {
        hasher.update(&value.to_bits().to_le_bytes());
    }
    let hash = hasher.finalize();
    let bytes = hash.as_bytes();
    [
        u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
        u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]),
        u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]),
        u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::framework::render::build_source_cubemap_from_equirect;

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
    fn ibl_bake_key_tracks_effective_directional_sun_parameters() {
        let mut base = ProceduralSkyParams::default_gradient();
        base.sun_direction = Vec4::new(0.0, 2.0, 1.0, 0.0);
        base.sun_color = Vec4::new(1.0, 0.8, 0.6, 1.0);
        base.sun_intensity = 4.0;
        base.sun_angular_radius_radians = 0.04;
        let base_key = base.ibl_bake_key();

        let variants = [
            ProceduralSkyParams {
                sun_direction: Vec4::new(1.0, 2.0, 1.0, 0.0),
                ..base
            },
            ProceduralSkyParams {
                sun_color: Vec4::new(0.9, 0.8, 0.6, 1.0),
                ..base
            },
            ProceduralSkyParams {
                sun_intensity: 5.0,
                ..base
            },
            ProceduralSkyParams {
                sun_angular_radius_radians: 0.06,
                ..base
            },
        ];

        assert_ne!(base_key.source_hash, [0; 4]);
        for variant in variants {
            assert_ne!(base_key, variant.ibl_bake_key());
        }
    }

    #[test]
    fn ibl_bake_key_uses_normalized_sun_direction_and_ignores_disabled_sun() {
        let mut enabled = ProceduralSkyParams::default_gradient();
        enabled.sun_direction = Vec4::new(0.0, 2.0, 1.0, 0.0);
        enabled.sun_intensity = 4.0;
        let mut scaled = enabled;
        scaled.sun_direction *= 3.0;

        assert_eq!(enabled.ibl_bake_key(), scaled.ibl_bake_key());

        let disabled = ProceduralSkyParams::default_gradient();
        let changed_but_disabled = ProceduralSkyParams {
            sun_direction: Vec4::new(1.0, 0.0, 0.0, 0.0),
            sun_color: Vec4::new(0.25, 0.5, 0.75, 1.0),
            sun_angular_radius_radians: 0.2,
            ..disabled
        };
        assert_eq!(disabled.ibl_bake_key(), changed_but_disabled.ibl_bake_key());
        assert_eq!(disabled.ibl_bake_key().source_hash, [0; 4]);

        let invalid_direction = ProceduralSkyParams {
            sun_direction: Vec4::ZERO,
            sun_intensity: 4.0,
            ..disabled
        };
        assert_eq!(invalid_direction.ibl_bake_key().source_hash, [0; 4]);
    }

    #[test]
    fn resolved_sun_keeps_a_strict_cosine_interval_after_radius_clamping() {
        let mut sky = ProceduralSkyParams::default_gradient();
        sky.sun_intensity = 1.0;
        sky.sun_angular_radius_radians = 0.0;

        let sun = sky.resolved_sun();

        assert!(sun.intensity_and_cosines.y < sun.intensity_and_cosines.z);
    }

    #[test]
    fn ibl_bake_key_tracks_source_revision() {
        let first = ProceduralSkyParams::default_gradient();
        let mut second = first;
        second.source_revision += 1;

        assert_ne!(first.ibl_bake_key(), second.ibl_bake_key());
    }

    #[test]
    fn source_cubemap_bake_key_tracks_source_hash() {
        let first = SourceCubemapEnvironment::new(
            build_source_cubemap_from_equirect(1, |_, _| [0.25, 0.5, 0.75, 1.0]),
            2,
            [1, 2, 3, 4],
        );
        let second = SourceCubemapEnvironment::new(
            build_source_cubemap_from_equirect(1, |_, _| [0.25, 0.5, 0.75, 1.0]),
            2,
            [1, 2, 3, 5],
        );

        assert_ne!(first.ibl_bake_key(), second.ibl_bake_key());
        let skybox = SkyboxSettings::source_cubemap(first.clone());
        assert_eq!(skybox.ibl_bake_key(), Some(first.ibl_bake_key()));
        assert_eq!(skybox.source_cubemap_environment(), Some(&first));
    }

    #[test]
    fn source_cubemap_environment_can_carry_optional_iem_without_changing_bake_key() {
        let mip_chain = build_source_cubemap_from_equirect(1, |_, _| [0.25, 0.5, 0.75, 1.0]);
        let bake_key =
            SourceCubemapEnvironment::new(mip_chain.clone(), 3, [1, 2, 3, 4]).ibl_bake_key();
        let environment =
            SourceCubemapEnvironment::new(mip_chain, 3, [1, 2, 3, 4]).with_irradiance_cube(
                SourceCubemapIrradianceCube::new(1, vec![[0.25, 0.5, 0.75]; 6]),
            );

        assert_eq!(environment.ibl_bake_key(), bake_key);
        assert_eq!(
            environment
                .irradiance_cube()
                .map(SourceCubemapIrradianceCube::face_size),
            Some(1)
        );
    }

    #[test]
    fn source_cubemap_upload_key_tracks_optional_iem_content() {
        let environment = SourceCubemapEnvironment::new(
            build_source_cubemap_from_equirect(1, |_, _| [0.25, 0.5, 0.75, 1.0]),
            3,
            [1, 2, 3, 4],
        );
        let without_iem = environment.texture_upload_key();
        let first_iem = environment
            .clone()
            .with_irradiance_cube(SourceCubemapIrradianceCube::new(
                1,
                vec![[0.25, 0.5, 0.75]; 6],
            ))
            .texture_upload_key();
        let changed_iem = environment
            .with_irradiance_cube(SourceCubemapIrradianceCube::new(
                1,
                vec![[0.5, 0.25, 0.75]; 6],
            ))
            .texture_upload_key();

        assert_ne!(without_iem, first_iem);
        assert_ne!(first_iem, changed_iem);
    }

    #[test]
    fn source_cubemap_artifact_provenance_does_not_change_gpu_upload_identity() {
        let environment = SourceCubemapEnvironment::new(
            build_source_cubemap_from_equirect(1, |_, _| [0.25, 0.5, 0.75, 1.0]),
            3,
            [1, 2, 3, 4],
        );
        let upload_key = environment.texture_upload_key();
        let with_provenance = environment.with_bake_artifact_hash([9, 8, 7, 6]);

        assert_eq!(with_provenance.bake_artifact_hash, [9, 8, 7, 6]);
        assert_eq!(with_provenance.texture_upload_key(), upload_key);
    }

    #[test]
    fn source_cubemap_builds_ibl_bake_request_from_source_mip_chain_shape() {
        let environment = SourceCubemapEnvironment::new(
            build_source_cubemap_from_equirect(4, |_, _| [0.25, 0.5, 0.75, 1.0]),
            7,
            [9, 8, 7, 6],
        );

        let request = environment.ibl_bake_artifact_request(IblBakeArtifactContents::SH9);

        assert_eq!(request.bake_key(), environment.ibl_bake_key());
        assert_eq!(request.source_face_size(), 4);
        assert_eq!(request.source_mip_count(), 3);
        assert_eq!(request.required_contents(), IblBakeArtifactContents::SH9);
    }
}
