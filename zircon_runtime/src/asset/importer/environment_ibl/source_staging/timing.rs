use std::time::Duration;

/// Wall-time ownership for one environment source staging attempt.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EnvironmentIblSourceStagingTiming {
    pub(in crate::asset::importer::environment_ibl) source_decode: Duration,
    pub(in crate::asset::importer::environment_ibl) cubemap_build: Duration,
    pub(in crate::asset::importer::environment_ibl) equirect_projection: Duration,
    pub(in crate::asset::importer::environment_ibl) source_mip_build: Duration,
    pub(in crate::asset::importer::environment_ibl) pmrem_build: Duration,
    pub(in crate::asset::importer::environment_ibl) sh9_build: Duration,
    pub(in crate::asset::importer::environment_ibl) irradiance_cube_build: Duration,
    pub(in crate::asset::importer::environment_ibl) bundle_write: Duration,
}

impl EnvironmentIblSourceStagingTiming {
    pub const fn source_decode(&self) -> Duration {
        self.source_decode
    }

    pub const fn cubemap_build(&self) -> Duration {
        self.cubemap_build
    }

    pub const fn equirect_projection(&self) -> Duration {
        self.equirect_projection
    }

    pub const fn source_mip_build(&self) -> Duration {
        self.source_mip_build
    }

    pub const fn pmrem_build(&self) -> Duration {
        self.pmrem_build
    }

    pub const fn sh9_build(&self) -> Duration {
        self.sh9_build
    }

    pub const fn irradiance_cube_build(&self) -> Duration {
        self.irradiance_cube_build
    }

    pub const fn bundle_write(&self) -> Duration {
        self.bundle_write
    }

    pub const fn total(&self) -> Duration {
        self.source_decode
            .saturating_add(self.cubemap_build)
            .saturating_add(self.irradiance_cube_build)
            .saturating_add(self.bundle_write)
    }
}
