use zircon_runtime::ui::{surface::UiSurface, tree::UiRuntimeTreeLayoutExt};
use zircon_runtime_interface::ui::{event_ui::UiNodeId, layout::UiSize, tree::UiVisibility};

use crate::ui::workbench::autolayout::{
    workbench_layout_tier_for_physical_width, WorkbenchLayoutTier,
};

use super::error::BuiltinHostWindowTemplateBridgeError;

const RESPONSIVE_MIN_TIER_ATTRIBUTE: &str = "responsive_min_tier";
const RESPONSIVE_MAX_TIER_ATTRIBUTE: &str = "responsive_max_tier";
const RESPONSIVE_COMPACT_DRAWER_ATTRIBUTE: &str = "responsive_compact_drawer";
const MODULE_DETAILS_DRAWER_ROLE: &str = "module_details";

pub(super) fn apply_workbench_responsive_layout(
    surface: &mut UiSurface,
    physical_shell_size: UiSize,
    scale_factor: f32,
    compact_module_details_drawer_open: bool,
) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
    let tier = workbench_layout_tier_for_physical_width(physical_shell_size.width, scale_factor);
    let responsive_nodes = surface
        .tree
        .nodes
        .values()
        .filter_map(|node| {
            let metadata = node.template_metadata.as_ref()?;
            let minimum = metadata
                .attributes
                .get(RESPONSIVE_MIN_TIER_ATTRIBUTE)
                .and_then(|value| value.as_str())
                .and_then(parse_layout_tier);
            let maximum = metadata
                .attributes
                .get(RESPONSIVE_MAX_TIER_ATTRIBUTE)
                .and_then(|value| value.as_str())
                .and_then(parse_layout_tier);
            let compact_drawer = metadata
                .attributes
                .get(RESPONSIVE_COMPACT_DRAWER_ATTRIBUTE)
                .and_then(|value| value.as_str())
                .is_some_and(|role| role == MODULE_DETAILS_DRAWER_ROLE);
            if minimum.is_none() && maximum.is_none() && !compact_drawer {
                return None;
            }
            Some((
                node.node_id,
                responsive_node_visible(
                    tier,
                    minimum,
                    maximum,
                    compact_drawer,
                    compact_module_details_drawer_open,
                ),
            ))
        })
        .collect::<Vec<_>>();

    for (node_id, visible) in responsive_nodes {
        apply_responsive_visibility(surface, node_id, visible)?;
    }
    Ok(())
}

fn responsive_node_visible(
    tier: WorkbenchLayoutTier,
    minimum: Option<WorkbenchLayoutTier>,
    maximum: Option<WorkbenchLayoutTier>,
    compact_drawer: bool,
    compact_drawer_open: bool,
) -> bool {
    let rank = tier_rank(tier);
    let within_authored_bounds = minimum.map_or(true, |minimum| rank >= tier_rank(minimum))
        && maximum.map_or(true, |maximum| rank <= tier_rank(maximum));
    let compact_drawer_override = compact_drawer
        && compact_drawer_open
        && rank >= tier_rank(WorkbenchLayoutTier::Narrow)
        && rank <= tier_rank(WorkbenchLayoutTier::Regular);

    within_authored_bounds || compact_drawer_override
}

fn apply_responsive_visibility(
    surface: &mut UiSurface,
    node_id: UiNodeId,
    visible: bool,
) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
    let next = if visible {
        UiVisibility::Visible
    } else {
        UiVisibility::Collapsed
    };
    let changed = surface
        .tree
        .node_mut(node_id)
        .map(|node| {
            let changed = node.visibility != next;
            node.visibility = next;
            changed
        })
        .unwrap_or(false);
    if changed {
        surface.tree.mark_layout_dirty(node_id)?;
    }
    Ok(())
}

fn parse_layout_tier(value: &str) -> Option<WorkbenchLayoutTier> {
    let value = value.trim();
    if value.eq_ignore_ascii_case("ultra") {
        Some(WorkbenchLayoutTier::Ultra)
    } else if value.eq_ignore_ascii_case("narrow") {
        Some(WorkbenchLayoutTier::Narrow)
    } else if value.eq_ignore_ascii_case("regular") {
        Some(WorkbenchLayoutTier::Regular)
    } else if value.eq_ignore_ascii_case("wide") {
        Some(WorkbenchLayoutTier::Wide)
    } else {
        None
    }
}

fn tier_rank(tier: WorkbenchLayoutTier) -> u8 {
    match tier {
        WorkbenchLayoutTier::Ultra => 0,
        WorkbenchLayoutTier::Narrow => 1,
        WorkbenchLayoutTier::Regular => 2,
        WorkbenchLayoutTier::Wide => 3,
    }
}

#[cfg(test)]
mod tests {
    use super::super::componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge;
    use super::*;

    #[test]
    fn responsive_min_tier_order_matches_workbench_breakpoint_order() {
        assert!(tier_rank(WorkbenchLayoutTier::Ultra) < tier_rank(WorkbenchLayoutTier::Narrow));
        assert!(tier_rank(WorkbenchLayoutTier::Narrow) < tier_rank(WorkbenchLayoutTier::Regular));
        assert!(tier_rank(WorkbenchLayoutTier::Regular) < tier_rank(WorkbenchLayoutTier::Wide));
        assert_eq!(
            parse_layout_tier(" regular "),
            Some(WorkbenchLayoutTier::Regular)
        );
        assert_eq!(parse_layout_tier("unsupported"), None);
    }

    #[test]
    fn responsive_tier_parsing_avoids_per_node_lowercase_allocation() {
        let source = include_str!("responsive_layout.rs");
        let forbidden = ["to_ascii", "_lowercase"].concat();

        assert!(!source.contains(&forbidden));
    }

    #[test]
    fn responsive_visibility_honors_both_tier_bounds() {
        let visible = |tier| {
            responsive_node_visible(
                tier,
                Some(WorkbenchLayoutTier::Narrow),
                Some(WorkbenchLayoutTier::Regular),
                false,
                false,
            )
        };

        assert!(!visible(WorkbenchLayoutTier::Ultra));
        assert!(visible(WorkbenchLayoutTier::Narrow));
        assert!(visible(WorkbenchLayoutTier::Regular));
        assert!(!visible(WorkbenchLayoutTier::Wide));
    }

    #[test]
    fn compact_details_drawer_overrides_wide_minimum_only_when_open() {
        let visible = |tier, open| {
            responsive_node_visible(tier, Some(WorkbenchLayoutTier::Wide), None, true, open)
        };

        assert!(visible(WorkbenchLayoutTier::Wide, false));
        assert!(!visible(WorkbenchLayoutTier::Regular, false));
        assert!(visible(WorkbenchLayoutTier::Regular, true));
        assert!(visible(WorkbenchLayoutTier::Narrow, true));
        assert!(!visible(WorkbenchLayoutTier::Ultra, true));
    }

    #[test]
    fn compact_module_details_drawer_is_reachable_without_reducing_regular_center_budget() {
        let mut regular =
            BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(900.0, 620.0))
                .expect("regular workbench should build");
        assert!(regular
            .control_frame("WorkbenchModuleDetailsDrawerToggle")
            .is_some());
        assert!(regular.control_frame("WorkbenchEffectRightPanel").is_none());
        let center_before_drawer = regular
            .control_frame("WorkbenchEffectCenterPanel")
            .expect("regular effect center should remain visible");

        regular
            .dispatch_binding_state_for_control(
                "WorkbenchModuleDetailsDrawerToggle",
                "Workbench/ToggleModuleDetailsDrawer",
            )
            .expect("details drawer should open");
        assert!(regular.control_frame("WorkbenchEffectRightPanel").is_some());
        assert_eq!(
            regular.control_frame("WorkbenchEffectCenterPanel"),
            Some(center_before_drawer),
            "overlay details must not consume the regular center budget"
        );

        regular
            .dispatch_binding_state_for_control(
                "WorkbenchModuleDetailsDrawerToggle",
                "Workbench/ToggleModuleDetailsDrawer",
            )
            .expect("details drawer should close");
        assert!(regular.control_frame("WorkbenchEffectRightPanel").is_none());

        regular
            .dispatch_binding_state_for_control("WorkbenchModuleScene", "WorkbenchModule/Scene")
            .expect("scene workspace should activate");
        assert!(regular
            .control_frame("WorkbenchModuleDetailsDrawerToggle")
            .is_none());
        regular
            .dispatch_binding_state_for_control("WorkbenchModuleEffect", "WorkbenchModule/Effect")
            .expect("effect workspace should reactivate");
        assert!(regular
            .control_frame("WorkbenchModuleDetailsDrawerToggle")
            .is_some());

        let wide = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0))
            .expect("wide workbench should build");
        assert!(wide.control_frame("WorkbenchEffectRightPanel").is_some());
        assert!(wide
            .control_frame("WorkbenchModuleDetailsDrawerToggle")
            .is_none());

        let ultra = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(420.0, 520.0))
            .expect("ultra workbench should build");
        assert!(ultra.control_frame("WorkbenchEffectRightPanel").is_none());
        assert!(ultra
            .control_frame("WorkbenchModuleDetailsDrawerToggle")
            .is_none());
    }
}
