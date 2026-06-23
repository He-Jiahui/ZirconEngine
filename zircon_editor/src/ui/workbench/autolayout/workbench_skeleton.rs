use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use zircon_runtime_interface::ui::design_tokens::EditorDesignTokens;

use crate::ui::workbench::layout::ActivityDrawerMode;

use super::{
    EditorRegion, EditorRegionRole, RegionBinding, ShellRegionId, WorkbenchConstraintTokenName,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkbenchSkeleton {
    pub top_toolbar_asset: String,
    pub activity_rail_left_asset: String,
    pub activity_rail_right_asset: Option<String>,
    pub document_tabs_asset: String,
    pub regions: Vec<RegionBinding>,
    pub bottom_output_asset: String,
    pub status_bar_asset: String,
    pub default_drawer_modes: BTreeMap<EditorRegion, ActivityDrawerMode>,
}

impl WorkbenchSkeleton {
    pub fn jetbrains_default() -> Self {
        let regions = vec![
            region(
                EditorRegion::LeftTop,
                EditorRegionRole::PlacementTools,
                "res://ui/editor/host/asset_surface_controls.v2.ui.toml",
                Some("--left-drawer-width"),
            ),
            region(
                EditorRegion::LeftBottom,
                EditorRegionRole::ProjectTree,
                "res://ui/editor/host/hierarchy_body.v2.ui.toml",
                Some("--left-drawer-width"),
            ),
            region(
                EditorRegion::RightTop,
                EditorRegionRole::HierarchyStructure,
                "res://ui/editor/host/hierarchy_body.v2.ui.toml",
                Some("--right-drawer-width"),
            ),
            region(
                EditorRegion::RightBottom,
                EditorRegionRole::DetailInspector,
                "res://ui/editor/host/inspector_body.v2.ui.toml",
                Some("--right-drawer-width"),
            ),
            region(
                EditorRegion::Bottom,
                EditorRegionRole::ConsoleDiagnosticsTimeline,
                "res://ui/editor/host/console_body.v2.ui.toml",
                Some("--bottom-output-height"),
            ),
            region(
                EditorRegion::Center,
                EditorRegionRole::CenterDocument,
                "res://ui/editor/components/workbench/shell/workbench_main_band.zui",
                None,
            ),
        ];

        Self {
            top_toolbar_asset:
                "res://ui/editor/components/workbench/shell/workbench_top_toolbar.zui".to_string(),
            activity_rail_left_asset:
                "res://ui/editor/components/workbench/shell/workbench_activity_rail.zui".to_string(),
            activity_rail_right_asset: None,
            document_tabs_asset:
                "res://ui/editor/components/workbench/primitives/inputs/workbench_tab_strip.zui"
                    .to_string(),
            regions,
            bottom_output_asset: "res://ui/editor/host/console_body.v2.ui.toml".to_string(),
            status_bar_asset: "res://ui/editor/components/workbench/shell/workbench_status_bar.zui"
                .to_string(),
            default_drawer_modes: EditorRegion::ALL
                .into_iter()
                .filter(|region| region.drawer_slot().is_some())
                .map(|region| (region, ActivityDrawerMode::Pinned))
                .collect(),
        }
    }

    pub fn region(&self, region: EditorRegion) -> Option<&RegionBinding> {
        self.regions.iter().find(|binding| binding.region == region)
    }

    pub fn default_drawer_mode(&self, region: EditorRegion) -> Option<ActivityDrawerMode> {
        self.default_drawer_modes.get(&region).copied()
    }

    pub fn preferred_region_extents_from_tokens(
        &self,
        tokens: &EditorDesignTokens,
    ) -> BTreeMap<ShellRegionId, f32> {
        let mut extents = BTreeMap::new();
        for binding in &self.regions {
            let Some(size_token) = binding.size_token.as_ref() else {
                continue;
            };
            let Some(value) = tokens.density_value_for_token_name(size_token.as_str()) else {
                continue;
            };
            extents
                .entry(binding.shell_region())
                .and_modify(|existing: &mut f32| *existing = existing.max(value))
                .or_insert(value);
        }
        extents
    }
}

fn region(
    region: EditorRegion,
    role: EditorRegionRole,
    panel_asset: &'static str,
    size_token: Option<&'static str>,
) -> RegionBinding {
    RegionBinding::from_trusted_parts(
        region,
        role,
        panel_asset,
        size_token.map(WorkbenchConstraintTokenName::new),
    )
}
