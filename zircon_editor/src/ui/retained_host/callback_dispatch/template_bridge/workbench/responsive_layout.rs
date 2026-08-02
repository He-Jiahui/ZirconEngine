use zircon_runtime::ui::{surface::UiSurface, tree::UiRuntimeTreeLayoutExt};
use zircon_runtime_interface::ui::{event_ui::UiNodeId, layout::UiSize, tree::UiVisibility};

use crate::ui::workbench::autolayout::{
    WorkbenchLayoutTier, workbench_layout_tier_for_physical_width,
};

use super::error::BuiltinHostWindowTemplateBridgeError;

const RESPONSIVE_MIN_TIER_ATTRIBUTE: &str = "responsive_min_tier";

pub(super) fn apply_workbench_responsive_layout(
    surface: &mut UiSurface,
    shell_size: UiSize,
    scale_factor: f32,
) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
    let tier = workbench_layout_tier_for_physical_width(shell_size.width, scale_factor);
    let responsive_nodes = surface
        .tree
        .nodes
        .values()
        .filter_map(|node| {
            let minimum = node
                .template_metadata
                .as_ref()?
                .attributes
                .get(RESPONSIVE_MIN_TIER_ATTRIBUTE)?
                .as_str()
                .and_then(parse_layout_tier)?;
            Some((node.node_id, tier_rank(tier) >= tier_rank(minimum)))
        })
        .collect::<Vec<_>>();

    for (node_id, visible) in responsive_nodes {
        apply_responsive_visibility(surface, node_id, visible)?;
    }
    Ok(())
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
}
