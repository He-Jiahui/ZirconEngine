use super::{
    IblBakeArtifactContents, IblBakeArtifactRequest, IblBakeKey, SkyboxSettings,
    SourceCubemapEnvironment,
};

#[derive(Clone, Debug, PartialEq)]
pub struct EnvironmentExtract {
    pub skybox: SkyboxSettings,
}

impl EnvironmentExtract {
    pub fn disabled() -> Self {
        Self {
            skybox: SkyboxSettings::none(),
        }
    }

    pub fn procedural_default() -> Self {
        Self {
            skybox: SkyboxSettings::procedural_default(),
        }
    }

    pub fn source_cubemap(source_cubemap: SourceCubemapEnvironment) -> Self {
        Self {
            skybox: SkyboxSettings::source_cubemap(source_cubemap),
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

        assert_eq!(request.face_size(), 4);
        assert_eq!(request.mip_count(), 3);
        assert_eq!(request.required_contents(), IblBakeArtifactContents::IEM);
    }
}
