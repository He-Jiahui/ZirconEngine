use super::{
    IblBakeArtifactContents, IblBakeArtifactRequest, IblBakeKey, LightProbeGridData,
    LightmapConsumeContract, LightmapContractValidationError, ReflectionProbeData,
    SOURCE_CUBEMAP_PMREM_FACE_SIZE, SOURCE_CUBEMAP_PMREM_MIP_COUNT, SkyboxSettings,
    SourceCubemapEnvironment,
};

#[derive(Clone, Debug, PartialEq)]
pub struct EnvironmentExtract {
    pub skybox: SkyboxSettings,
    pub probes: Vec<ReflectionProbeData>,
    pub baked_lighting: Option<LightmapConsumeContract>,
    pub probe_grid: Option<LightProbeGridData>,
}

impl EnvironmentExtract {
    pub fn disabled() -> Self {
        Self {
            skybox: SkyboxSettings::none(),
            probes: Vec::new(),
            baked_lighting: None,
            probe_grid: None,
        }
    }

    pub fn procedural_default() -> Self {
        Self {
            skybox: SkyboxSettings::procedural_default(),
            probes: Vec::new(),
            baked_lighting: None,
            probe_grid: None,
        }
    }

    pub fn source_cubemap(source_cubemap: SourceCubemapEnvironment) -> Self {
        Self {
            skybox: SkyboxSettings::source_cubemap(source_cubemap),
            probes: Vec::new(),
            baked_lighting: None,
            probe_grid: None,
        }
    }

    pub fn from_preview_skybox_enabled(enabled: bool) -> Self {
        if enabled {
            Self::procedural_default()
        } else {
            Self::disabled()
        }
    }

    pub fn skybox_enabled(&self) -> bool {
        self.skybox.is_enabled()
    }

    pub fn with_reflection_probes(mut self, probes: Vec<ReflectionProbeData>) -> Self {
        self.probes = probes;
        self
    }

    pub fn reflection_probes(&self) -> &[ReflectionProbeData] {
        &self.probes
    }

    pub fn try_with_baked_lighting(
        mut self,
        baked_lighting: LightmapConsumeContract,
        probe_grid: Option<LightProbeGridData>,
    ) -> Result<Self, LightmapContractValidationError> {
        baked_lighting.validate()?;
        if let Some(grid) = &probe_grid {
            grid.validate()?;
            if grid.light_set_generation != baked_lighting.light_set_generation {
                return Err(LightmapContractValidationError::GenerationMismatch);
            }
        }
        self.baked_lighting = Some(baked_lighting);
        self.probe_grid = probe_grid;
        Ok(self)
    }

    pub fn baked_lighting(&self) -> Option<&LightmapConsumeContract> {
        self.baked_lighting.as_ref()
    }

    pub fn light_probe_grid(&self) -> Option<&LightProbeGridData> {
        self.probe_grid.as_ref()
    }

    pub fn ibl_bake_key(&self) -> Option<IblBakeKey> {
        self.skybox.ibl_bake_key()
    }

    pub fn source_cubemap_ibl_bake_request(
        &self,
        required_contents: IblBakeArtifactContents,
    ) -> Option<IblBakeArtifactRequest> {
        self.skybox
            .source_cubemap_environment()
            .map(|environment| environment.ibl_bake_artifact_request(required_contents))
    }
}

impl Default for EnvironmentExtract {
    fn default() -> Self {
        Self::disabled()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::math::{Vec3, Vec4};
    use crate::core::resource::ResourceId;

    #[test]
    fn default_environment_is_disabled() {
        let environment = EnvironmentExtract::default();

        assert!(!environment.skybox_enabled());
        assert!(environment.ibl_bake_key().is_none());
    }

    #[test]
    fn preview_skybox_flag_maps_to_procedural_environment() {
        let environment = EnvironmentExtract::from_preview_skybox_enabled(true);

        assert!(environment.skybox_enabled());
        assert!(environment.ibl_bake_key().is_some());
    }

    #[test]
    fn source_cubemap_extract_supplies_ibl_bake_request_shape() {
        let environment = EnvironmentExtract::source_cubemap(SourceCubemapEnvironment::new(
            crate::core::framework::render::build_source_cubemap_from_equirect(4, |_, _| {
                [0.25, 0.5, 0.75, 1.0]
            }),
            2,
            [1, 2, 3, 4],
        ));

        let request = environment
            .source_cubemap_ibl_bake_request(IblBakeArtifactContents::IEM)
            .expect("source cubemap environment should produce an IBL bake request");

        assert_eq!(request.source_face_size(), 4);
        assert_eq!(request.source_mip_count(), 3);
        assert_eq!(request.pmrem_face_size(), SOURCE_CUBEMAP_PMREM_FACE_SIZE);
        assert_eq!(request.pmrem_mip_count(), SOURCE_CUBEMAP_PMREM_MIP_COUNT);
        assert_eq!(request.required_contents(), IblBakeArtifactContents::IEM);
    }

    #[test]
    fn baked_environment_requires_one_light_set_generation() {
        let lightmaps = LightmapConsumeContract::new(
            3,
            ResourceId::from_stable_label("res://lighting/test.lightmap-array"),
            super::super::LightmapAtlasDescriptor {
                page_size: 4,
                page_count: 1,
                format: super::super::LightmapAtlasFormat::Rgba16Float,
            },
            vec![(
                11,
                super::super::LightmapInstanceSlot {
                    atlas_page: 0,
                    uv_rect: Vec4::new(1.0, 1.0, 0.0, 0.0),
                },
            )],
        );
        let mismatched_grid = LightProbeGridData {
            light_set_generation: 4,
            bounds_min: Vec3::ZERO,
            cell_size: Vec3::ONE,
            dims: [1, 1, 1],
            sh: vec![super::super::ShL2Rgb::default()],
        };

        assert_eq!(
            EnvironmentExtract::disabled()
                .try_with_baked_lighting(lightmaps.clone(), Some(mismatched_grid)),
            Err(LightmapContractValidationError::GenerationMismatch)
        );

        let environment = EnvironmentExtract::disabled()
            .try_with_baked_lighting(lightmaps, None)
            .expect("matching baked contract should be accepted");
        assert_eq!(
            environment
                .baked_lighting()
                .expect("lightmap contract should be stored")
                .light_set_generation,
            3
        );
        assert!(environment.light_probe_grid().is_none());
    }
}
