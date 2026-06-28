use std::collections::BTreeSet;

use crate::ui::surface::{UiSurface, UiSurfaceRebuildReport};
use crate::ui::template::UiAssetCompileCache;
use zircon_runtime_interface::ui::tree::UiDirtyFlags;
use zircon_runtime_interface::ui::tree::UiTreeError;

use super::compiler::UiAssetCompileCacheEvictionReport;
use super::watch_invalidation::UiAssetWatchInvalidationReport;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum UiHotReloadAssetKind {
    Template,
    Theme,
    Icon,
    Font,
    Texture,
    Other,
}

/// Ordered runtime work derived from a folded asset-watch batch.
///
/// The plan is intentionally pure: it classifies URI strings and describes cache,
/// restyle, rebuild, and resource-damage actions without reading files or mutating
/// theme/resource registries.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UiAssetHotReloadPlan {
    pub changed_assets: Vec<String>,
    pub removed_assets: Vec<String>,
    pub template_rebuild_targets: Vec<String>,
    pub removed_compiled_assets: Vec<String>,
    pub theme_restyle_assets: Vec<String>,
    pub theme_restyle_targets: Vec<String>,
    pub resource_refresh_assets: Vec<String>,
    pub resource_damage_targets: Vec<String>,
    pub unclassified_assets: Vec<String>,
    pub dirty: UiDirtyFlags,
    pub rebuild_required: bool,
}

/// Coarse surface damage produced by applying a hot-reload plan to surface roots.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct UiAssetHotReloadSurfaceDirtyReport {
    pub roots_marked: usize,
    pub dirty: UiDirtyFlags,
}

impl UiAssetHotReloadPlan {
    pub fn from_watch_report(report: &UiAssetWatchInvalidationReport) -> Self {
        let mut builder = UiAssetHotReloadPlanBuilder::from_report(report);
        let removed_assets = report
            .removed_assets
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();

        for asset in &report.changed_assets {
            match classify_ui_hot_reload_asset(asset) {
                UiHotReloadAssetKind::Template if removed_assets.contains(asset.as_str()) => {
                    builder.add_removed_template(asset);
                }
                UiHotReloadAssetKind::Template => builder.add_changed_template(asset),
                UiHotReloadAssetKind::Theme => builder.add_changed_theme(asset),
                UiHotReloadAssetKind::Icon | UiHotReloadAssetKind::Texture => {
                    builder.add_changed_render_resource(asset);
                }
                UiHotReloadAssetKind::Font => builder.add_changed_font_resource(asset),
                UiHotReloadAssetKind::Other => builder.add_changed_unclassified_resource(asset),
            }
        }

        for asset in &report.removed_assets {
            if classify_ui_hot_reload_asset(asset) == UiHotReloadAssetKind::Template {
                builder.add_removed_template(asset);
            }
        }

        for target in &report.rebuild_targets {
            if builder.has_template_change {
                builder.add_template_rebuild_target(target);
            }
            if builder.has_theme_change {
                builder.add_theme_restyle_target(target);
            }
            if builder.has_resource_change {
                builder.add_resource_damage_target(target);
            }
        }

        builder.finish()
    }

    pub fn is_empty(&self) -> bool {
        self.changed_assets.is_empty()
            && self.removed_assets.is_empty()
            && self.template_rebuild_targets.is_empty()
            && self.removed_compiled_assets.is_empty()
            && self.theme_restyle_assets.is_empty()
            && self.theme_restyle_targets.is_empty()
            && self.resource_refresh_assets.is_empty()
            && self.resource_damage_targets.is_empty()
            && self.unclassified_assets.is_empty()
            && !self.dirty.any()
            && !self.rebuild_required
    }

    pub fn evict_compile_cache(
        &self,
        cache: &mut UiAssetCompileCache,
    ) -> UiAssetCompileCacheEvictionReport {
        let mut targets = Vec::new();
        targets.extend(self.template_rebuild_targets.iter().map(String::as_str));
        targets.extend(self.removed_compiled_assets.iter().map(String::as_str));
        cache.evict_assets(targets)
    }

    pub fn mark_surface_roots_dirty(
        &self,
        surface: &mut UiSurface,
    ) -> Result<UiAssetHotReloadSurfaceDirtyReport, UiTreeError> {
        if !self.dirty.any() {
            return Ok(UiAssetHotReloadSurfaceDirtyReport::default());
        }

        let roots = surface.tree.roots.clone();
        // Until runtime surfaces retain asset-to-node ownership, hot reload marks
        // roots with the aggregate dirty domains instead of guessing a subtree.
        for root in &roots {
            surface.mark_node_dirty(*root, self.dirty)?;
        }
        Ok(UiAssetHotReloadSurfaceDirtyReport {
            roots_marked: roots.len(),
            dirty: self.dirty,
        })
    }

    pub fn rebuild_dirty_surface(
        &self,
        surface: &mut UiSurface,
        root_size: zircon_runtime_interface::ui::layout::UiSize,
    ) -> Result<UiSurfaceRebuildReport, UiTreeError> {
        self.mark_surface_roots_dirty(surface)?;
        surface.rebuild_dirty(root_size)
    }
}

pub fn classify_ui_hot_reload_asset(asset_id: &str) -> UiHotReloadAssetKind {
    let path = normalized_asset_path(asset_id);
    if path.ends_with(".theme.toml") {
        return UiHotReloadAssetKind::Theme;
    }
    if path.ends_with(".icon.toml") || path.ends_with(".svg") {
        return UiHotReloadAssetKind::Icon;
    }
    if path.ends_with(".font.toml") || has_any_suffix(&path, FONT_SOURCE_SUFFIXES) {
        return UiHotReloadAssetKind::Font;
    }
    if path.ends_with(".zui") {
        return UiHotReloadAssetKind::Template;
    }
    if has_any_suffix(&path, TEXTURE_SOURCE_SUFFIXES) {
        return UiHotReloadAssetKind::Texture;
    }
    UiHotReloadAssetKind::Other
}

const FONT_SOURCE_SUFFIXES: &[&str] = &[".ttf", ".otf", ".ttc", ".woff", ".woff2"];

const TEXTURE_SOURCE_SUFFIXES: &[&str] = &[
    ".png", ".jpg", ".jpeg", ".bmp", ".tga", ".tiff", ".tif", ".gif", ".webp", ".hdr", ".exr",
    ".qoi", ".pnm", ".pbm", ".pgm", ".ppm", ".dds", ".ktx", ".ktx2", ".astc", ".cube", ".psd",
];

struct UiAssetHotReloadPlanBuilder {
    plan: UiAssetHotReloadPlan,
    seen_template_targets: BTreeSet<String>,
    seen_removed_compiled_assets: BTreeSet<String>,
    seen_theme_assets: BTreeSet<String>,
    seen_theme_targets: BTreeSet<String>,
    seen_resource_assets: BTreeSet<String>,
    seen_resource_targets: BTreeSet<String>,
    seen_unclassified_assets: BTreeSet<String>,
    has_template_change: bool,
    has_theme_change: bool,
    has_resource_change: bool,
}

impl UiAssetHotReloadPlanBuilder {
    fn from_report(report: &UiAssetWatchInvalidationReport) -> Self {
        Self {
            plan: UiAssetHotReloadPlan {
                changed_assets: report.changed_assets.clone(),
                removed_assets: report.removed_assets.clone(),
                ..UiAssetHotReloadPlan::default()
            },
            seen_template_targets: BTreeSet::new(),
            seen_removed_compiled_assets: BTreeSet::new(),
            seen_theme_assets: BTreeSet::new(),
            seen_theme_targets: BTreeSet::new(),
            seen_resource_assets: BTreeSet::new(),
            seen_resource_targets: BTreeSet::new(),
            seen_unclassified_assets: BTreeSet::new(),
            has_template_change: false,
            has_theme_change: false,
            has_resource_change: false,
        }
    }

    fn add_changed_template(&mut self, asset: &str) {
        self.has_template_change = true;
        self.plan.rebuild_required = true;
        mark_full_rebuild_dirty(&mut self.plan.dirty);
        self.add_template_rebuild_target(asset);
    }

    fn add_removed_template(&mut self, asset: &str) {
        self.has_template_change = true;
        self.plan.rebuild_required = true;
        mark_full_rebuild_dirty(&mut self.plan.dirty);
        push_unique(
            &mut self.plan.removed_compiled_assets,
            &mut self.seen_removed_compiled_assets,
            asset,
        );
    }

    fn add_changed_theme(&mut self, asset: &str) {
        self.has_theme_change = true;
        mark_restyle_dirty(&mut self.plan.dirty);
        push_unique(
            &mut self.plan.theme_restyle_assets,
            &mut self.seen_theme_assets,
            asset,
        );
    }

    fn add_changed_render_resource(&mut self, asset: &str) {
        self.has_resource_change = true;
        mark_render_resource_dirty(&mut self.plan.dirty);
        push_unique(
            &mut self.plan.resource_refresh_assets,
            &mut self.seen_resource_assets,
            asset,
        );
    }

    fn add_changed_font_resource(&mut self, asset: &str) {
        self.has_resource_change = true;
        mark_font_resource_dirty(&mut self.plan.dirty);
        push_unique(
            &mut self.plan.resource_refresh_assets,
            &mut self.seen_resource_assets,
            asset,
        );
    }

    fn add_changed_unclassified_resource(&mut self, asset: &str) {
        self.has_resource_change = true;
        mark_render_resource_dirty(&mut self.plan.dirty);
        push_unique(
            &mut self.plan.resource_refresh_assets,
            &mut self.seen_resource_assets,
            asset,
        );
        push_unique(
            &mut self.plan.unclassified_assets,
            &mut self.seen_unclassified_assets,
            asset,
        );
    }

    fn add_template_rebuild_target(&mut self, target: &str) {
        push_unique(
            &mut self.plan.template_rebuild_targets,
            &mut self.seen_template_targets,
            target,
        );
    }

    fn add_theme_restyle_target(&mut self, target: &str) {
        push_unique(
            &mut self.plan.theme_restyle_targets,
            &mut self.seen_theme_targets,
            target,
        );
    }

    fn add_resource_damage_target(&mut self, target: &str) {
        push_unique(
            &mut self.plan.resource_damage_targets,
            &mut self.seen_resource_targets,
            target,
        );
    }

    fn finish(self) -> UiAssetHotReloadPlan {
        self.plan
    }
}

fn push_unique(targets: &mut Vec<String>, seen: &mut BTreeSet<String>, value: &str) {
    if seen.insert(value.to_string()) {
        targets.push(value.to_string());
    }
}

fn mark_full_rebuild_dirty(dirty: &mut UiDirtyFlags) {
    dirty.layout = true;
    dirty.hit_test = true;
    dirty.render = true;
    dirty.style = true;
    dirty.text = true;
    dirty.input = true;
    dirty.visible_range = true;
}

fn mark_restyle_dirty(dirty: &mut UiDirtyFlags) {
    dirty.style = true;
    dirty.layout = true;
    dirty.hit_test = true;
    dirty.render = true;
    dirty.text = true;
}

fn mark_render_resource_dirty(dirty: &mut UiDirtyFlags) {
    dirty.render = true;
}

fn mark_font_resource_dirty(dirty: &mut UiDirtyFlags) {
    dirty.text = true;
    dirty.layout = true;
    dirty.hit_test = true;
    dirty.render = true;
}

fn normalized_asset_path(asset_id: &str) -> String {
    asset_id
        .split_once('#')
        .map(|(path, _)| path)
        .unwrap_or(asset_id)
        .trim()
        .to_ascii_lowercase()
}

fn has_any_suffix(path: &str, suffixes: &[&str]) -> bool {
    suffixes.iter().any(|suffix| path.ends_with(suffix))
}
