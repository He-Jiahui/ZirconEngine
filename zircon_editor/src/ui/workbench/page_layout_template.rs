use serde::{Deserialize, Serialize};

use super::autolayout::{
    EditorRegion, EditorRegionRole, RegionBinding, WorkbenchConstraintTokenName,
};
use super::layout::{ActivityDrawerMode, ActivityDrawerSlot, MainPageId, SplitAxis};
use super::{CenterSplitLayout, LayoutPresetDrawerState, LayoutPresetName};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PageLayoutTemplate {
    pub page: MainPageId,
    pub region_fills: Vec<RegionBinding>,
    pub default_preset: LayoutPresetName,
    pub default_drawer_states: Vec<LayoutPresetDrawerState>,
    pub default_center_split: CenterSplitLayout,
}

impl PageLayoutTemplate {
    pub fn builtin_templates() -> Vec<Self> {
        vec![
            Self::scene(),
            Self::game(),
            Self::material(),
            Self::material_preview(),
            Self::inspector(),
            Self::prefab(),
            Self::ui_designer(),
            Self::ui_source(),
            Self::animation_timeline(),
            Self::animation_graph(),
            Self::asset_browser(),
            Self::console(),
            Self::runtime_diagnostics(),
        ]
    }

    pub fn scene() -> Self {
        workbench_page(
            "scene",
            "res://ui/editor/host/editor_main_frame.v2.ui.toml",
            LayoutPresetName::Authoring,
            pinned_drawers(),
            CenterSplitLayout::SingleDocument,
        )
    }

    pub fn game() -> Self {
        workbench_page(
            "game",
            "res://ui/editor/host/editor_main_frame.v2.ui.toml",
            LayoutPresetName::Focus,
            collapsed_drawers(),
            CenterSplitLayout::SingleDocument,
        )
    }

    pub fn material() -> Self {
        workbench_page(
            "material",
            "res://ui/editor/components/workbench/modules/core/rendering/workbench_material_workspace.zui",
            LayoutPresetName::Authoring,
            pinned_drawers(),
            horizontal_split(),
        )
    }

    pub fn material_preview() -> Self {
        workbench_page(
            "material_preview",
            "res://ui/editor/components/workbench/modules/core/rendering/workbench_material_workspace.zui",
            LayoutPresetName::Review,
            right_review_drawers(),
            CenterSplitLayout::SingleDocument,
        )
    }

    pub fn inspector() -> Self {
        Self {
            page: MainPageId::new("inspector"),
            region_fills: vec![
                binding(
                    EditorRegion::Center,
                    EditorRegionRole::CenterDocument,
                    "res://ui/editor/host/inspector_body.v2.ui.toml",
                    None,
                ),
                binding(
                    EditorRegion::RightTop,
                    EditorRegionRole::HierarchyStructure,
                    "res://ui/editor/host/hierarchy_body.v2.ui.toml",
                    Some("--right-drawer-width"),
                ),
                binding(
                    EditorRegion::RightBottom,
                    EditorRegionRole::DetailInspector,
                    "res://ui/editor/host/inspector_body.v2.ui.toml",
                    Some("--right-drawer-width"),
                ),
            ],
            default_preset: LayoutPresetName::Review,
            default_drawer_states: pinned_drawers(),
            default_center_split: CenterSplitLayout::SingleDocument,
        }
    }

    pub fn prefab() -> Self {
        workbench_page(
            "prefab",
            "res://ui/editor/components/workbench/modules/extensions/world/workbench_extension_prefab_editor_workspace.zui",
            LayoutPresetName::Authoring,
            pinned_drawers(),
            CenterSplitLayout::SingleDocument,
        )
    }

    pub fn ui_designer() -> Self {
        workbench_page(
            "ui_designer",
            "res://ui/editor/components/workbench/modules/core/ui/workbench_hud_workspace.zui",
            LayoutPresetName::Authoring,
            pinned_drawers(),
            horizontal_split(),
        )
    }

    pub fn ui_source() -> Self {
        workbench_page(
            "ui_source",
            "res://ui/editor/components/workbench/modules/core/ui/workbench_hud_workspace.zui",
            LayoutPresetName::Focus,
            collapsed_drawers(),
            CenterSplitLayout::SingleDocument,
        )
    }

    pub fn animation_timeline() -> Self {
        workbench_page(
            "animation_timeline",
            "res://ui/editor/host/animation_sequence_body.v2.ui.toml",
            LayoutPresetName::Debug,
            bottom_debug_drawers(),
            horizontal_split(),
        )
    }

    pub fn animation_graph() -> Self {
        workbench_page(
            "animation_graph",
            "res://ui/editor/host/animation_graph_body.v2.ui.toml",
            LayoutPresetName::Review,
            right_review_drawers(),
            horizontal_split(),
        )
    }

    pub fn asset_browser() -> Self {
        workbench_page(
            "asset_browser",
            "res://ui/editor/components/workbench/modules/core/assets/workbench_assets_workspace.zui",
            LayoutPresetName::Review,
            right_review_drawers(),
            CenterSplitLayout::SingleDocument,
        )
    }

    pub fn console() -> Self {
        workbench_page(
            "console",
            "res://ui/editor/host/console_body.v2.ui.toml",
            LayoutPresetName::Debug,
            bottom_debug_drawers(),
            CenterSplitLayout::SingleDocument,
        )
    }

    pub fn runtime_diagnostics() -> Self {
        workbench_page(
            "runtime_diagnostics",
            "res://ui/editor/components/workbench/modules/extensions/diagnostics/workbench_extension_runtime_diagnostics_workspace.zui",
            LayoutPresetName::Debug,
            bottom_debug_drawers(),
            horizontal_split(),
        )
    }

    pub fn has_region_role(&self, region: EditorRegion, role: EditorRegionRole) -> bool {
        self.region_fills
            .iter()
            .any(|binding| binding.region == region && binding.role == role)
    }
}

fn workbench_page(
    page: &'static str,
    center_asset: &'static str,
    default_preset: LayoutPresetName,
    default_drawer_states: Vec<LayoutPresetDrawerState>,
    default_center_split: CenterSplitLayout,
) -> PageLayoutTemplate {
    PageLayoutTemplate {
        page: MainPageId::new(page),
        region_fills: standard_region_fills(center_asset),
        default_preset,
        default_drawer_states,
        default_center_split,
    }
}

fn standard_region_fills(center_asset: &'static str) -> Vec<RegionBinding> {
    vec![
        binding(
            EditorRegion::LeftTop,
            EditorRegionRole::PlacementTools,
            "res://ui/editor/host/asset_surface_controls.v2.ui.toml",
            Some("--left-drawer-width"),
        ),
        binding(
            EditorRegion::LeftBottom,
            EditorRegionRole::ProjectTree,
            "res://ui/editor/asset_browser.v2.ui.toml",
            Some("--left-drawer-width"),
        ),
        binding(
            EditorRegion::Center,
            EditorRegionRole::CenterDocument,
            center_asset,
            None,
        ),
        binding(
            EditorRegion::RightTop,
            EditorRegionRole::HierarchyStructure,
            "res://ui/editor/host/hierarchy_body.v2.ui.toml",
            Some("--right-drawer-width"),
        ),
        binding(
            EditorRegion::RightBottom,
            EditorRegionRole::DetailInspector,
            "res://ui/editor/host/inspector_body.v2.ui.toml",
            Some("--right-drawer-width"),
        ),
        binding(
            EditorRegion::Bottom,
            EditorRegionRole::ConsoleDiagnosticsTimeline,
            "res://ui/editor/host/console_body.v2.ui.toml",
            Some("--bottom-output-height"),
        ),
    ]
}

fn binding(
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

fn pinned_drawers() -> Vec<LayoutPresetDrawerState> {
    drawer_states(|_| ActivityDrawerMode::Pinned)
}

fn collapsed_drawers() -> Vec<LayoutPresetDrawerState> {
    drawer_states(|_| ActivityDrawerMode::Collapsed)
}

fn right_review_drawers() -> Vec<LayoutPresetDrawerState> {
    drawer_states(|slot| match slot {
        ActivityDrawerSlot::RightTop | ActivityDrawerSlot::RightBottom => {
            ActivityDrawerMode::Pinned
        }
        _ => ActivityDrawerMode::Collapsed,
    })
}

fn bottom_debug_drawers() -> Vec<LayoutPresetDrawerState> {
    drawer_states(|slot| {
        if slot == ActivityDrawerSlot::Bottom {
            ActivityDrawerMode::Pinned
        } else {
            ActivityDrawerMode::Collapsed
        }
    })
}

fn drawer_states(
    mode: impl Fn(ActivityDrawerSlot) -> ActivityDrawerMode,
) -> Vec<LayoutPresetDrawerState> {
    ActivityDrawerSlot::ALL
        .into_iter()
        .map(|slot| LayoutPresetDrawerState {
            slot,
            mode: mode(slot),
        })
        .collect()
}

fn horizontal_split() -> CenterSplitLayout {
    CenterSplitLayout::Split {
        axis: SplitAxis::Horizontal,
        panes: 2,
    }
}
