use std::collections::BTreeMap;

use crate::ui::surface::UiSurface;
use crate::ui::template::{
    UiAssetCompileCache, UiAssetSurfaceIndex, UiResourceResolver,
    UiResourceResolverCacheInvalidationReport,
};
use crate::ui::theme::{UiThemeRegistry, UiThemeReloadOutcome};
use zircon_runtime_interface::ui::event_ui::UiTreeId;
use zircon_runtime_interface::ui::style::UiThemeDocument;
use zircon_runtime_interface::ui::tree::UiTreeError;

use super::compiler::UiAssetCompileCacheEvictionReport;
use super::hot_reload_plan::UiAssetHotReloadPlan;
use super::surface_index::UiAssetSurfaceHotReloadApplyReport;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct UiAssetHotReloadExecutionReport {
    pub cache_eviction: UiAssetCompileCacheEvictionReport,
    pub resource_resolver_cache: Option<UiResourceResolverCacheInvalidationReport>,
    pub theme_reload: Option<UiThemeReloadOutcome>,
    pub surface_apply: UiAssetSurfaceHotReloadApplyReport,
    pub template_rebuild_targets: Vec<String>,
    pub removed_compiled_assets: Vec<String>,
    pub resource_refresh_assets: Vec<String>,
    pub unclassified_assets: Vec<String>,
}

#[derive(Debug)]
pub struct UiAssetHotReloadExecutor<'a> {
    pub cache: &'a mut UiAssetCompileCache,
    pub surface_index: &'a UiAssetSurfaceIndex,
    pub surfaces: &'a mut BTreeMap<UiTreeId, UiSurface>,
    pub resource_resolver: Option<&'a mut UiResourceResolver>,
    pub theme_registry: Option<&'a mut UiThemeRegistry>,
}

impl UiAssetHotReloadExecutionReport {
    pub fn is_empty(&self) -> bool {
        self.cache_eviction == UiAssetCompileCacheEvictionReport::default()
            && self.resource_resolver_cache.is_none()
            && self.theme_reload.is_none()
            && self.surface_apply == UiAssetSurfaceHotReloadApplyReport::default()
            && self.template_rebuild_targets.is_empty()
            && self.removed_compiled_assets.is_empty()
            && self.resource_refresh_assets.is_empty()
            && self.unclassified_assets.is_empty()
    }
}

impl UiAssetHotReloadPlan {
    pub fn execute_runtime_reload(
        &self,
        executor: UiAssetHotReloadExecutor<'_>,
        theme_document: Option<UiThemeDocument>,
    ) -> Result<UiAssetHotReloadExecutionReport, UiTreeError> {
        let cache_eviction = self.evict_compile_cache(executor.cache);
        let resource_resolver_cache = match executor.resource_resolver {
            Some(resolver) if !self.resource_refresh_assets.is_empty() => Some(
                resolver.invalidate_uris(self.resource_refresh_assets.iter().map(String::as_str)),
            ),
            _ => None,
        };
        let theme_reload = match (executor.theme_registry, theme_document) {
            (Some(registry), Some(document)) if !self.theme_restyle_assets.is_empty() => {
                Some(registry.apply_document(document))
            }
            _ => None,
        };
        let surface_apply = executor
            .surface_index
            .mark_target_surfaces_dirty(self, executor.surfaces)?;

        Ok(UiAssetHotReloadExecutionReport {
            cache_eviction,
            resource_resolver_cache,
            theme_reload,
            surface_apply,
            template_rebuild_targets: self.template_rebuild_targets.clone(),
            removed_compiled_assets: self.removed_compiled_assets.clone(),
            resource_refresh_assets: self.resource_refresh_assets.clone(),
            unclassified_assets: self.unclassified_assets.clone(),
        })
    }
}
