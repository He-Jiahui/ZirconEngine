#[cfg(feature = "profiling")]
const PROFILE_STREAM: &str = "asset";
#[cfg(feature = "profiling")]
const PROFILE_CATEGORY: &str = "environment_ibl.stage";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::asset::importer::environment_ibl) enum EnvironmentIblStagingPhase {
    SourceClassify,
    SourceIdentity,
    CacheProbe,
    SourceDecode,
    CubemapBuild,
    IrradianceCubeBuild,
    BundleEncode,
    BundleCommit,
}

impl EnvironmentIblStagingPhase {
    pub(in crate::asset::importer::environment_ibl) fn enter(
        self,
    ) -> EnvironmentIblStagingPhaseScope {
        #[cfg(feature = "profiling")]
        let scope = crate::core::runtime::diagnostics::profiling::ProfileScope::enter(
            PROFILE_STREAM,
            PROFILE_CATEGORY,
            self.name(),
        );
        #[cfg(not(feature = "profiling"))]
        let _ = self;
        EnvironmentIblStagingPhaseScope {
            #[cfg(feature = "profiling")]
            _scope: scope,
        }
    }

    #[cfg(feature = "profiling")]
    const fn name(self) -> &'static str {
        match self {
            Self::SourceClassify => "source_classify",
            Self::SourceIdentity => "source_identity",
            Self::CacheProbe => "cache_probe",
            Self::SourceDecode => "source_decode",
            Self::CubemapBuild => "cubemap_build",
            Self::IrradianceCubeBuild => "irradiance_cube_build",
            Self::BundleEncode => "bundle_encode",
            Self::BundleCommit => "bundle_commit",
        }
    }
}

#[must_use]
pub(in crate::asset::importer::environment_ibl) struct EnvironmentIblStagingPhaseScope {
    #[cfg(feature = "profiling")]
    _scope: crate::core::runtime::diagnostics::profiling::ProfileScope,
}
