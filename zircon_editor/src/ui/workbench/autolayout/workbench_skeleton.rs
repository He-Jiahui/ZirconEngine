use std::{collections::BTreeMap, sync::OnceLock};

use serde::{Deserialize, Serialize};
use zircon_runtime_interface::ui::{design_tokens::EditorDesignTokens, layout::UiDimension};

use crate::ui::workbench::layout::ActivityDrawerMode;

use super::{
    CssLikeConstraint, CssLikeDimension, CssLikeSize, EditorRegion, EditorRegionRole,
    RegionBinding, ShellRegionId, WorkbenchConstraintTokenName,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 壳资源与语义区域绑定基线；尺寸令牌在使用时按当前主题解析，和具体layout用户状态分开。
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
    /// 缓存内建绑定结构，每次重新读取调用方主题值；host可按主题快照身份缓存结果。
    pub fn default_region_extents_from_tokens(
        tokens: &EditorDesignTokens,
    ) -> BTreeMap<ShellRegionId, f32> {
        static DEFAULT_WORKBENCH_SKELETON: OnceLock<WorkbenchSkeleton> = OnceLock::new();

        DEFAULT_WORKBENCH_SKELETON
            .get_or_init(Self::jetbrains_default)
            .preferred_region_extents_from_tokens(tokens)
    }

    /// 内建六区域与chrome资源基线，供正常host及作者资产加载后的其余壳字段复用。
    pub fn jetbrains_default() -> Self {
        let regions = vec![
            region(
                EditorRegion::LeftTop,
                EditorRegionRole::PlacementTools,
                "res://ui/editor/host/asset_surface_controls.zui",
                Some("--left-drawer-width"),
            ),
            region(
                EditorRegion::LeftBottom,
                EditorRegionRole::ProjectTree,
                "res://ui/editor/host/hierarchy_body.zui",
                Some("--left-drawer-width"),
            ),
            region(
                EditorRegion::RightTop,
                EditorRegionRole::HierarchyStructure,
                "res://ui/editor/host/hierarchy_body.zui",
                Some("--right-drawer-width"),
            ),
            region(
                EditorRegion::RightBottom,
                EditorRegionRole::DetailInspector,
                "res://ui/editor/host/inspector_body.zui",
                Some("--right-drawer-width"),
            ),
            region(
                EditorRegion::Bottom,
                EditorRegionRole::ConsoleDiagnosticsTimeline,
                "res://ui/editor/host/console_body.zui",
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
            bottom_output_asset: "res://ui/editor/host/console_body.zui".to_string(),
            status_bar_asset: "res://ui/editor/components/workbench/shell/workbench_status_bar.zui"
                .to_string(),
            default_drawer_modes: EditorRegion::ALL
                .into_iter()
                .filter(|region| region.drawer_slot().is_some())
                .map(|region| (region, ActivityDrawerMode::Pinned))
                .collect(),
        }
    }

    /// 取语义位置绑定；标准顺序走固定slot，作者数据重排时仍按身份查找。
    pub fn region(&self, region: EditorRegion) -> Option<&RegionBinding> {
        self.regions
            .get(expected_region_index(region))
            .filter(|binding| binding.region == region)
            .or_else(|| self.regions.iter().find(|binding| binding.region == region))
    }

    pub fn default_drawer_mode(&self, region: EditorRegion) -> Option<ActivityDrawerMode> {
        self.default_drawer_modes.get(&region).copied()
    }

    /// 形成logical壳主轴默认；同侧多个语义绑定取最大令牌值，无法解析者不提供偏好。
    pub fn preferred_region_extents_from_tokens(
        &self,
        tokens: &EditorDesignTokens,
    ) -> BTreeMap<ShellRegionId, f32> {
        let mut extents = BTreeMap::new();
        for binding in &self.regions {
            let Some(size_token) = binding.size_token.as_ref() else {
                continue;
            };
            let Some(value) = preferred_extent_from_constraint(
                tokens,
                binding.shell_region(),
                size_token.clone(),
            ) else {
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

const fn expected_region_index(region: EditorRegion) -> usize {
    match region {
        EditorRegion::LeftTop => 0,
        EditorRegion::LeftBottom => 1,
        EditorRegion::RightTop => 2,
        EditorRegion::RightBottom => 3,
        EditorRegion::Bottom => 4,
        EditorRegion::Center => 5,
    }
}

/// 沿作者约束到共享DTO的同一令牌解释路径，只提取可作为绝对主轴偏好的值。
fn preferred_extent_from_constraint(
    tokens: &EditorDesignTokens,
    region: ShellRegionId,
    size_token: WorkbenchConstraintTokenName,
) -> Option<f32> {
    let size = match region {
        ShellRegionId::Bottom => CssLikeSize {
            width: CssLikeDimension::Auto,
            height: CssLikeDimension::Token(size_token),
        },
        ShellRegionId::Left | ShellRegionId::Right | ShellRegionId::Document => CssLikeSize {
            width: CssLikeDimension::Token(size_token),
            height: CssLikeDimension::Auto,
        },
    };
    let constraint = CssLikeConstraint {
        size,
        ..CssLikeConstraint::default()
    };
    if !constraint.family().is_taffy_owned() {
        return None;
    }
    let style = constraint.into_layout_style(tokens).ok()?;
    match region {
        ShellRegionId::Bottom => layout_extent(style.size.height),
        ShellRegionId::Left | ShellRegionId::Right | ShellRegionId::Document => {
            layout_extent(style.size.width)
        }
    }
}

/// 区域默认需要可直接比较的绝对长度；auto、百分比与无效值保留为缺省。
fn layout_extent(dimension: UiDimension) -> Option<f32> {
    match dimension {
        UiDimension::Px(value) if value.is_finite() && value >= 0.0 => Some(value),
        UiDimension::Auto | UiDimension::Px(_) | UiDimension::Percent(_) => None,
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

#[cfg(test)]
#[path = "tests/workbench_skeleton.rs"]
mod tests;
