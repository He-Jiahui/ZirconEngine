use std::time::Duration;

/// Wall-time ownership for one environment source staging attempt.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EnvironmentIblSourceStagingTiming {
    pub(in crate::asset::importer::environment_ibl) source_classify: Duration,
    pub(in crate::asset::importer::environment_ibl) source_identity: Duration,
    pub(in crate::asset::importer::environment_ibl) cache_probe: Duration,
    pub(in crate::asset::importer::environment_ibl) source_decode: Duration,
    pub(in crate::asset::importer::environment_ibl) cubemap_build: Duration,
    pub(in crate::asset::importer::environment_ibl) equirect_projection: Duration,
    pub(in crate::asset::importer::environment_ibl) source_mip_build: Duration,
    pub(in crate::asset::importer::environment_ibl) pmrem_build: Duration,
    pub(in crate::asset::importer::environment_ibl) sh9_build: Duration,
    pub(in crate::asset::importer::environment_ibl) irradiance_cube_build: Duration,
    pub(in crate::asset::importer::environment_ibl) bundle_encode: Duration,
    pub(in crate::asset::importer::environment_ibl) bundle_commit: Duration,
}

impl EnvironmentIblSourceStagingTiming {
    pub const fn source_classify(&self) -> Duration {
        self.source_classify
    }

    pub const fn source_identity(&self) -> Duration {
        self.source_identity
    }

    pub const fn cache_probe(&self) -> Duration {
        self.cache_probe
    }

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
        self.bundle_encode.saturating_add(self.bundle_commit)
    }

    pub const fn bundle_encode(&self) -> Duration {
        self.bundle_encode
    }

    pub const fn bundle_commit(&self) -> Duration {
        self.bundle_commit
    }

    pub const fn total(&self) -> Duration {
        self.source_classify
            .saturating_add(self.source_identity)
            .saturating_add(self.cache_probe)
            .saturating_add(self.source_decode)
            .saturating_add(self.cubemap_build)
            .saturating_add(self.irradiance_cube_build)
            .saturating_add(self.bundle_encode)
            .saturating_add(self.bundle_commit)
    }
}
