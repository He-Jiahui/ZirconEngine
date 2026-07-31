mod standalone;

#[cfg(test)]
mod metrics;

use crate::plugin::PluginFeatureBundleManifest;

use super::super::package_validation::{
    EmbeddedFeatureKind, RuntimePluginPackageValidationProjection,
};
use standalone::StandaloneFeatureValidationProjection;

#[cfg(test)]
pub(in crate::plugin::runtime_plugin) use metrics::{
    begin_feature_projection_build_observation, observed_embedded_feature_projection_views,
    observed_standalone_feature_projection_builds,
};

pub(super) struct RuntimePluginFeatureValidationProjection<'projection, 'manifest> {
    source: ProjectionSource<'projection, 'manifest>,
}

enum ProjectionSource<'projection, 'manifest> {
    Standalone(StandaloneFeatureValidationProjection),
    Embedded {
        package: &'projection RuntimePluginPackageValidationProjection<'manifest>,
        kind: EmbeddedFeatureKind,
        feature: usize,
    },
}

impl RuntimePluginFeatureValidationProjection<'static, 'static> {
    pub(super) fn standalone(feature: &PluginFeatureBundleManifest) -> Self {
        #[cfg(test)]
        metrics::observe_standalone_feature_projection_build();

        Self {
            source: ProjectionSource::Standalone(StandaloneFeatureValidationProjection::build(
                feature,
            )),
        }
    }
}

impl<'projection, 'manifest> RuntimePluginFeatureValidationProjection<'projection, 'manifest> {
    pub(super) fn embedded(
        package: &'projection RuntimePluginPackageValidationProjection<'manifest>,
        kind: EmbeddedFeatureKind,
        feature: usize,
    ) -> Self {
        #[cfg(test)]
        metrics::observe_embedded_feature_projection_view();

        Self {
            source: ProjectionSource::Embedded {
                package,
                kind,
                feature,
            },
        }
    }

    pub(super) fn capability_is_duplicate(&self, capability: usize) -> bool {
        match &self.source {
            ProjectionSource::Standalone(projection) => {
                projection.capability_is_duplicate(capability)
            }
            ProjectionSource::Embedded {
                package,
                kind,
                feature,
            } => package.feature_capability_is_duplicate(*kind, *feature, capability),
        }
    }

    pub(super) fn dependency_is_duplicate(&self, dependency: usize) -> bool {
        match &self.source {
            ProjectionSource::Standalone(projection) => {
                projection.dependency_is_duplicate(dependency)
            }
            ProjectionSource::Embedded {
                package,
                kind,
                feature,
            } => package.feature_dependency_is_duplicate(*kind, *feature, dependency),
        }
    }

    pub(super) fn module_name_is_duplicate(&self, module: usize) -> bool {
        match &self.source {
            ProjectionSource::Standalone(projection) => projection.module_name_is_duplicate(module),
            ProjectionSource::Embedded {
                package,
                kind,
                feature,
            } => package.feature_module_name_is_duplicate(*kind, *feature, module),
        }
    }

    pub(super) fn module_capability_is_duplicate(&self, module: usize, capability: usize) -> bool {
        match &self.source {
            ProjectionSource::Standalone(projection) => {
                projection.module_capability_is_duplicate(module, capability)
            }
            ProjectionSource::Embedded {
                package,
                kind,
                feature,
            } => {
                package.feature_module_capability_is_duplicate(*kind, *feature, module, capability)
            }
        }
    }
}
