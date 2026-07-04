use super::{IblBakeKey, SampledEquirectangularEnvironment, SkyboxSettings};

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

    pub fn sampled_equirectangular(sampled: SampledEquirectangularEnvironment) -> Self {
        Self {
            skybox: SkyboxSettings::sampled_equirectangular(sampled),
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
}
