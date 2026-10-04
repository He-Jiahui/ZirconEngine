mod standalone;

#[cfg(test)]
#[path = "projection/tests/metrics.rs"]
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

/// 向行验证提供相同的重复查询接口，避免包内每个功能重新构建独立索引。
/// 查询中的行号必须来自建索引时的清单；投影建成后不得换序或改用另一份功能。
pub(super) struct RuntimePluginFeatureValidationProjection<'projection, 'manifest> {
    source: ProjectionSource<'projection, 'manifest>,
}

// 独立来源只保存重复位置，拥有自身数据；内嵌来源借用包级索引并保留列表种类和行号。
enum ProjectionSource<'projection, 'manifest> {
    Standalone(StandaloneFeatureValidationProjection),
    Embedded {
        package: &'projection RuntimePluginPackageValidationProjection<'manifest>,
        kind: EmbeddedFeatureKind,
        feature: usize,
    },
}

impl RuntimePluginFeatureValidationProjection<'static, 'static> {
    /// 为脱离包清单注册的功能创建局部索引；结果不借用功能清单本身。
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
    /// 借用本次包审查已建立的索引；种类和功能行号应由所属包列表的枚举传入。
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
