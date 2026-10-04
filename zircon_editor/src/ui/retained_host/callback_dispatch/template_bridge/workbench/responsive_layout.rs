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
#[path = "tests/responsive_layout.rs"]
mod tests;
