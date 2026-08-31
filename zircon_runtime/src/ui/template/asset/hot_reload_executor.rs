use std::collections::{BTreeMap, BTreeSet};

use thiserror::Error;

use crate::ui::surface::UiSurface;
use crate::ui::template::{
    UiAssetCompileCache, UiAssetSurfaceIndex, UiResourceResolver,
    UiResourceResolverCacheInvalidationReport,
};
use crate::ui::theme::{UiThemeRegistry, UiThemeReloadOutcome};
use zircon_runtime_interface::ui::event_ui::UiTreeId;
use zircon_runtime_interface::ui::style::UiThemeDocument;
use zircon_runtime_interface::ui::tree::UiTreeError;

use super::binding_reload_transaction::{
    UiBindingQuiescenceReceipt, UiBindingReloadPrepareError, UiBindingReloadTransaction,
};
use super::compiler::UiAssetCompileCacheEvictionReport;
use super::hot_reload_plan::UiAssetHotReloadPlan;
use super::surface_index::UiAssetSurfaceHotReloadApplyReport;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct UiAssetHotReloadExecutionReport {
    pub cache_eviction: UiAssetCompileCacheEvictionReport,
    pub resource_resolver_cache: Option<UiResourceResolverCacheInvalidationReport>,
    pub theme_reload: Option<UiThemeReloadOutcome>,
    pub surface_apply: UiAssetSurfaceHotReloadApplyReport,
    pub template_rebuilds: Vec<UiAssetTemplateRebuildReceipt>,
    pub template_rebuild_targets: Vec<String>,
    pub removed_compiled_assets: Vec<String>,
    pub resource_refresh_assets: Vec<String>,
    pub unclassified_assets: Vec<String>,
}

pub struct UiAssetHotReloadExecutor<'a> {
    pub cache: &'a mut UiAssetCompileCache,
    pub surface_index: &'a mut UiAssetSurfaceIndex,
    pub surfaces: &'a mut BTreeMap<UiTreeId, UiSurface>,
    pub resource_resolver: Option<&'a mut UiResourceResolver>,
    pub theme_registry: Option<&'a mut UiThemeRegistry>,
    pub template_rebuilder: Option<&'a mut dyn UiAssetSurfaceRebuilder>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UiAssetTemplateRebuildReceipt {
    pub tree_id: UiTreeId,
    pub template_assets: Vec<String>,
    pub component_states_migrated: usize,
    pub component_states_reset: usize,
    pub transient_input_state_reset: bool,
    pub binding_reload: UiBindingQuiescenceReceipt,
}

#[derive(Clone, Copy)]
pub struct UiAssetSurfaceRebuildRequest<'a> {
    pub tree_id: &'a UiTreeId,
    pub template_assets: &'a [String],
    pub previous_surface: &'a UiSurface,
}

pub trait UiAssetSurfaceRebuilder {
    fn prepare_surface(
        &mut self,
        request: UiAssetSurfaceRebuildRequest<'_>,
    ) -> Result<UiSurface, String>;
}

impl<F> UiAssetSurfaceRebuilder for F
where
    F: for<'request> FnMut(UiAssetSurfaceRebuildRequest<'request>) -> Result<UiSurface, String>,
{
    fn prepare_surface(
        &mut self,
        request: UiAssetSurfaceRebuildRequest<'_>,
    ) -> Result<UiSurface, String> {
        self(request)
    }
}

#[derive(Debug, Error)]
pub enum UiAssetHotReloadExecutionError {
    #[error("template hot reload requires a rebuilder for active surfaces {surface_ids:?}")]
    RebuilderRequired { surface_ids: Vec<UiTreeId> },
    #[error("failed to prepare template hot reload for {tree_id:?}: {detail}")]
    PrepareFailed { tree_id: UiTreeId, detail: String },
    #[error("template hot reload for {requested_tree_id:?} returned surface {actual_tree_id:?}")]
    TreeIdMismatch {
        requested_tree_id: UiTreeId,
        actual_tree_id: UiTreeId,
    },
    #[error(transparent)]
    BindingPrepare(#[from] UiBindingReloadPrepareError),
    #[error(transparent)]
    Tree(#[from] UiTreeError),
}

impl UiAssetHotReloadExecutionReport {
    pub fn is_empty(&self) -> bool {
        self.cache_eviction == UiAssetCompileCacheEvictionReport::default()
            && self.resource_resolver_cache.is_none()
            && self.theme_reload.is_none()
            && self.surface_apply == UiAssetSurfaceHotReloadApplyReport::default()
            && self.template_rebuilds.is_empty()
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
    ) -> Result<UiAssetHotReloadExecutionReport, UiAssetHotReloadExecutionError> {
        let UiAssetHotReloadExecutor {
            cache,
            surface_index,
            surfaces,
            resource_resolver,
            theme_registry,
            template_rebuilder,
        } = executor;
        let prepared =
            prepare_surface_publication(self, &*surface_index, surfaces, template_rebuilder)?;

        let cache_eviction = self.evict_compile_cache(cache);
        let resource_resolver_cache = match resource_resolver {
            Some(resolver) if !self.resource_refresh_assets.is_empty() => Some(
                resolver.invalidate_uris(self.resource_refresh_assets.iter().map(String::as_str)),
            ),
            _ => None,
        };
        let theme_reload = match (theme_registry, theme_document) {
            (Some(registry), Some(document)) if !self.theme_restyle_assets.is_empty() => {
                Some(registry.apply_document(document))
            }
            _ => None,
        };
        let UiAssetPreparedSurfacePublication {
            staged_surfaces,
            surface_apply,
            mut template_rebuilds,
        } = prepared;
        let mut rebuild_receipts = Vec::with_capacity(template_rebuilds.len());
        for (tree_id, surface) in staged_surfaces {
            let retired_surface = surfaces.insert(tree_id.clone(), surface);
            drop(retired_surface);

            let Some(rebuild) = template_rebuilds.remove(&tree_id) else {
                continue;
            };
            let published = surfaces
                .get(&tree_id)
                .expect("a staged surface was just published");
            surface_index.record_binding_program(tree_id.clone(), published.binding_program());
            let binding_reload = rebuild.binding_reload.publish(
                published.binding_program(),
                rebuild.component_states_migrated,
                rebuild.component_states_reset,
            );
            rebuild_receipts.push(UiAssetTemplateRebuildReceipt {
                tree_id,
                template_assets: rebuild.template_assets,
                component_states_migrated: rebuild.component_states_migrated,
                component_states_reset: rebuild.component_states_reset,
                transient_input_state_reset: true,
                binding_reload,
            });
        }

        Ok(UiAssetHotReloadExecutionReport {
            cache_eviction,
            resource_resolver_cache,
            theme_reload,
            surface_apply,
            template_rebuilds: rebuild_receipts,
            template_rebuild_targets: self.template_rebuild_targets.clone(),
            removed_compiled_assets: self.removed_compiled_assets.clone(),
            resource_refresh_assets: self.resource_refresh_assets.clone(),
            unclassified_assets: self.unclassified_assets.clone(),
        })
    }
}

struct UiAssetPreparedSurfacePublication {
    staged_surfaces: BTreeMap<UiTreeId, UiSurface>,
    surface_apply: UiAssetSurfaceHotReloadApplyReport,
    template_rebuilds: BTreeMap<UiTreeId, UiAssetPreparedTemplateRebuild>,
}

struct UiAssetPreparedTemplateRebuild {
    template_assets: Vec<String>,
    component_states_migrated: usize,
    component_states_reset: usize,
    binding_reload: UiBindingReloadTransaction,
}

fn prepare_surface_publication(
    plan: &UiAssetHotReloadPlan,
    surface_index: &UiAssetSurfaceIndex,
    surfaces: &BTreeMap<UiTreeId, UiSurface>,
    template_rebuilder: Option<&mut dyn UiAssetSurfaceRebuilder>,
) -> Result<UiAssetPreparedSurfacePublication, UiAssetHotReloadExecutionError> {
    let targets = surface_index.target_surfaces_for_plan(plan);
    let target_surface_ids = targets.all_target_surfaces();
    let mut staged_surfaces = target_surface_ids
        .iter()
        .filter_map(|tree_id| {
            surfaces
                .get(tree_id)
                .cloned()
                .map(|surface| (tree_id.clone(), surface))
        })
        .collect::<BTreeMap<_, _>>();
    let active_rebuild_surface_ids = targets
        .template_rebuild_surfaces
        .iter()
        .chain(&targets.removed_compiled_surfaces)
        .filter(|tree_id| surfaces.contains_key(*tree_id))
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    let mut template_rebuilds = BTreeMap::new();
    if !active_rebuild_surface_ids.is_empty() {
        let rebuilder = template_rebuilder.ok_or_else(|| {
            UiAssetHotReloadExecutionError::RebuilderRequired {
                surface_ids: active_rebuild_surface_ids.clone(),
            }
        })?;
        for tree_id in active_rebuild_surface_ids {
            let previous_surface = surfaces
                .get(&tree_id)
                .expect("active rebuild ids come from the surface map");
            let template_assets = template_assets_for_surface(plan, surface_index, &tree_id);
            let mut replacement = rebuilder
                .prepare_surface(UiAssetSurfaceRebuildRequest {
                    tree_id: &tree_id,
                    template_assets: &template_assets,
                    previous_surface,
                })
                .map_err(|detail| UiAssetHotReloadExecutionError::PrepareFailed {
                    tree_id: tree_id.clone(),
                    detail,
                })?;
            if replacement.tree.tree_id != tree_id {
                return Err(UiAssetHotReloadExecutionError::TreeIdMismatch {
                    requested_tree_id: tree_id,
                    actual_tree_id: replacement.tree.tree_id.clone(),
                });
            }
            let binding_reload = UiBindingReloadTransaction::prepare(
                tree_id.clone(),
                previous_surface.binding_program(),
                replacement.binding_program(),
            )?;
            let migration = replacement.adopt_hot_reload_state_from(previous_surface);
            staged_surfaces.insert(tree_id.clone(), replacement);
            template_rebuilds.insert(
                tree_id,
                UiAssetPreparedTemplateRebuild {
                    template_assets,
                    component_states_migrated: migration.migrated,
                    component_states_reset: migration.reset,
                    binding_reload,
                },
            );
        }
    }

    let surface_apply = surface_index.mark_target_surfaces_dirty(plan, &mut staged_surfaces)?;
    Ok(UiAssetPreparedSurfacePublication {
        staged_surfaces,
        surface_apply,
        template_rebuilds,
    })
}

fn template_assets_for_surface(
    plan: &UiAssetHotReloadPlan,
    surface_index: &UiAssetSurfaceIndex,
    tree_id: &UiTreeId,
) -> Vec<String> {
    plan.template_rebuild_targets
        .iter()
        .chain(&plan.removed_compiled_assets)
        .filter(|asset_id| {
            surface_index
                .surfaces_for_asset(asset_id)
                .any(|surface_id| surface_id == tree_id)
        })
        .cloned()
        .collect()
}
