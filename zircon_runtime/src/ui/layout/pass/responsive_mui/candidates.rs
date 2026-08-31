use std::collections::{BTreeMap, BTreeSet};

use toml::Value;
use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    tree::{UiTemplateNodeMetadata, UiTree},
};

use super::{
    BREAKPOINTS, bool_attribute, breakpoint_range_attribute, breakpoint_width_attribute,
    has_explicit_layout_container, width_floor_attribute, width_query_threshold,
};

#[derive(Clone, Debug, Default, PartialEq)]
pub(in crate::ui::layout::pass) struct MuiResponsiveCandidates {
    pub(super) media_query_node_ids: BTreeSet<UiNodeId>,
    pub(super) visibility_node_ids: BTreeSet<UiNodeId>,
    pub(super) container_node_ids: BTreeSet<UiNodeId>,
    pub(super) implicit_grid_parent_ids: BTreeSet<UiNodeId>,
    width_thresholds_by_node: BTreeMap<UiNodeId, Vec<u32>>,
    width_threshold_counts: BTreeMap<u32, usize>,
    definitions_by_node: BTreeMap<UiNodeId, ResponsiveDefinition>,
    last_responsive_width: Option<f32>,
}

#[derive(Clone, Debug, PartialEq)]
struct ResponsiveDefinition {
    component: String,
    attributes: BTreeMap<String, Value>,
}

impl MuiResponsiveCandidates {
    pub(in crate::ui::layout::pass) fn for_tree(tree: &UiTree) -> Self {
        let mut candidates = Self::default();
        for node_id in tree.nodes.keys().copied() {
            candidates.patch_node(tree, node_id);
        }
        candidates
    }

    pub(in crate::ui::layout::pass) fn patch_nodes(
        &mut self,
        tree: &UiTree,
        node_ids: &BTreeSet<UiNodeId>,
    ) {
        for node_id in node_ids.iter().copied() {
            self.patch_node(tree, node_id);
        }
    }

    fn patch_node(&mut self, tree: &UiTree, node_id: UiNodeId) {
        let previous_thresholds = self.width_thresholds_by_node.remove(&node_id);
        if let Some(thresholds) = previous_thresholds.as_ref() {
            for threshold in thresholds {
                decrement_threshold(&mut self.width_threshold_counts, *threshold);
            }
        }
        let previous_definition = self.definitions_by_node.remove(&node_id);
        let previous_membership = self.node_is_candidate(node_id);
        self.media_query_node_ids.remove(&node_id);
        self.visibility_node_ids.remove(&node_id);
        self.container_node_ids.remove(&node_id);
        self.implicit_grid_parent_ids.remove(&node_id);

        let Some(metadata) = tree
            .node(node_id)
            .and_then(|node| node.template_metadata.as_ref())
        else {
            if previous_membership || previous_thresholds.is_some() || previous_definition.is_some()
            {
                self.last_responsive_width = None;
            }
            return;
        };
        let attributes = &metadata.attributes;
        if metadata.component == "UseMediaQuery" {
            self.media_query_node_ids.insert(node_id);
        }
        if attributes.contains_key("display")
            || attributes.contains_key("visibility")
            || attributes.contains_key("visible")
        {
            self.visibility_node_ids.insert(node_id);
        }
        if matches!(metadata.component.as_str(), "Grid" | "Stack" | "Masonry")
            && !has_explicit_layout_container(attributes)
        {
            self.container_node_ids.insert(node_id);
        }
        if metadata.component == "Grid"
            && bool_attribute(attributes, "container")
            && !has_explicit_layout_container(attributes)
        {
            self.implicit_grid_parent_ids.insert(node_id);
        }
        let is_candidate = self.node_is_candidate(node_id);
        let thresholds = width_thresholds_for_metadata(metadata, is_candidate);
        if !thresholds.is_empty() {
            for threshold in &thresholds {
                *self.width_threshold_counts.entry(*threshold).or_default() += 1;
            }
            self.width_thresholds_by_node.insert(node_id, thresholds);
        }
        let next_definition = responsive_definition_for_metadata(metadata, is_candidate);
        let definition_changed = previous_definition.as_ref() != next_definition.as_ref();
        if let Some(definition) = next_definition {
            self.definitions_by_node.insert(node_id, definition);
        }
        if previous_membership != is_candidate || definition_changed {
            // Candidate metadata changes must be resolved at the next pass,
            // even when their width thresholds remain unchanged.
            self.last_responsive_width = None;
        }
    }

    pub(super) fn responsive_layout_may_change(&mut self, width: f32) -> bool {
        let width = normalized_width(width);
        let changed = self.last_responsive_width.is_none_or(|previous| {
            self.width_threshold_counts
                .keys()
                .map(|bits| f32::from_bits(*bits))
                .any(|threshold| widths_crossed(previous, width, threshold))
        });
        self.last_responsive_width = Some(width);
        changed
    }

    fn node_is_candidate(&self, node_id: UiNodeId) -> bool {
        self.media_query_node_ids.contains(&node_id)
            || self.visibility_node_ids.contains(&node_id)
            || self.container_node_ids.contains(&node_id)
            || self.implicit_grid_parent_ids.contains(&node_id)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::hint::black_box;
    use std::time::Instant;

    use toml::Value;
    use zircon_runtime_interface::ui::{
        event_ui::{UiNodeId, UiNodePath},
        tree::{UiTemplateNodeMetadata, UiTree, UiTreeNode},
    };

    use super::MuiResponsiveCandidates;

    fn media_query_node(query: &str) -> UiTreeNode {
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root/query")).with_template_metadata(
            UiTemplateNodeMetadata {
                component: "UseMediaQuery".to_string(),
                attributes: [("query".to_string(), Value::String(query.to_string()))]
                    .into_iter()
                    .collect(),
                ..UiTemplateNodeMetadata::default()
            },
        )
    }

    #[test]
    fn responsive_width_gate_reuses_candidates_inside_one_threshold_band() {
        let mut tree = UiTree::default();
        tree.nodes
            .insert(UiNodeId::new(1), media_query_node("(min-width: 600px)"));
        let mut candidates = MuiResponsiveCandidates::for_tree(&tree);

        assert!(candidates.responsive_layout_may_change(300.0));
        assert!(!candidates.responsive_layout_may_change(599.0));
        assert!(candidates.responsive_layout_may_change(600.0));
        assert!(!candidates.responsive_layout_may_change(899.0));
        assert!(candidates.responsive_layout_may_change(900.0));
    }

    #[test]
    fn responsive_candidate_mutation_reopens_gate_at_the_same_width() {
        let mut tree = UiTree::default();
        tree.nodes
            .insert(UiNodeId::new(1), media_query_node("(min-width: 600px)"));
        let mut candidates = MuiResponsiveCandidates::for_tree(&tree);
        assert!(candidates.responsive_layout_may_change(800.0));
        assert!(!candidates.responsive_layout_may_change(800.0));

        tree.nodes
            .insert(UiNodeId::new(1), media_query_node("(min-width: 900px)"));
        candidates.patch_nodes(&tree, &[UiNodeId::new(1)].into_iter().collect());

        assert!(candidates.responsive_layout_may_change(800.0));
    }

    #[test]
    fn responsive_grid_item_mutation_reopens_gate_at_the_same_width() {
        let node_id = UiNodeId::new(1);
        let mut tree = UiTree::default();
        tree.nodes.insert(
            node_id,
            UiTreeNode::new(node_id, UiNodePath::new("root/grid-item")).with_template_metadata(
                UiTemplateNodeMetadata {
                    component: "GridItem".to_string(),
                    attributes: [("size".to_string(), Value::Integer(6))]
                        .into_iter()
                        .collect(),
                    ..UiTemplateNodeMetadata::default()
                },
            ),
        );
        let mut candidates = MuiResponsiveCandidates::for_tree(&tree);
        assert!(candidates.responsive_layout_may_change(800.0));
        assert!(!candidates.responsive_layout_may_change(800.0));

        tree.nodes.insert(
            node_id,
            UiTreeNode::new(node_id, UiNodePath::new("root/grid-item")).with_template_metadata(
                UiTemplateNodeMetadata {
                    component: "GridItem".to_string(),
                    attributes: [("size".to_string(), Value::Integer(8))]
                        .into_iter()
                        .collect(),
                    ..UiTemplateNodeMetadata::default()
                },
            ),
        );
        candidates.patch_nodes(&tree, &[node_id].into_iter().collect());

        assert!(candidates.responsive_layout_may_change(800.0));
    }

    #[test]
    fn responsive_max_width_crossing_reopens_after_strict_boundary() {
        let mut tree = UiTree::default();
        tree.nodes
            .insert(UiNodeId::new(1), media_query_node("(max-width: 800px)"));
        let mut candidates = MuiResponsiveCandidates::for_tree(&tree);

        assert!(candidates.responsive_layout_may_change(800.0));
        assert!(candidates.responsive_layout_may_change(801.0));
    }

    #[test]
    fn optimization_batch_gy_runtime580_definition_is_moved_into_the_index() {
        let source = include_str!("candidates.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("responsive candidate production source");

        assert!(production.contains("if let Some(definition) = next_definition"));
        assert!(production.contains("insert(node_id, definition)"));
        assert!(!production.contains("insert(node_id, definition.clone())"));
    }

    #[test]
    #[ignore = "release performance evidence; run through the validation coordinator"]
    fn optimization_batch_gy_runtime580_definition_move_performance_evidence() {
        fn definition(index: usize) -> super::ResponsiveDefinition {
            super::ResponsiveDefinition {
                component: format!("ResponsivePanel{index}"),
                attributes: (0..24)
                    .map(|attribute| {
                        (
                            format!("attribute-{attribute}"),
                            Value::String(format!("value-{index}-{attribute}")),
                        )
                    })
                    .collect(),
            }
        }

        const DEFINITION_COUNT: usize = 4_096;
        let definitions = (0..DEFINITION_COUNT).map(definition).collect::<Vec<_>>();
        let mut legacy_samples = Vec::with_capacity(17);
        let mut optimized_samples = Vec::with_capacity(17);
        for _ in 0..17 {
            let legacy_input = definitions.clone();
            let started = Instant::now();
            let mut legacy = BTreeMap::new();
            for (index, definition) in legacy_input.iter().enumerate() {
                legacy.insert(UiNodeId::new(index as u64), definition.clone());
            }
            black_box(legacy);
            legacy_samples.push(started.elapsed().as_nanos());

            let optimized_input = definitions.clone();
            let started = Instant::now();
            let mut optimized = BTreeMap::new();
            for (index, definition) in optimized_input.into_iter().enumerate() {
                optimized.insert(UiNodeId::new(index as u64), definition);
            }
            black_box(optimized);
            optimized_samples.push(started.elapsed().as_nanos());
        }

        legacy_samples.sort_unstable();
        optimized_samples.sort_unstable();
        let legacy_p95 = legacy_samples[16];
        let optimized_p95 = optimized_samples[16];
        println!(
            "RUNTIME580_RESPONSIVE_DEFINITION_MOVE_BENCH_V1 definitions={DEFINITION_COUNT} attributes_per_definition=24 legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} target_ratio_bp=7000"
        );
        assert!(
            optimized_p95.saturating_mul(10_000) <= legacy_p95.saturating_mul(7_000),
            "responsive definition move P95 {optimized_p95} ns exceeded 70% of legacy {legacy_p95} ns"
        );
    }
}

fn width_thresholds_for_metadata(
    metadata: &UiTemplateNodeMetadata,
    is_candidate: bool,
) -> Vec<u32> {
    if !is_candidate {
        return Vec::new();
    }
    let attributes = &metadata.attributes;
    let mut thresholds = BREAKPOINTS
        .iter()
        .map(|(_, width)| width.to_bits())
        .collect::<Vec<_>>();
    for names in [
        &["min_width", "minWidth"][..],
        &["max_width", "maxWidth"][..],
    ] {
        if let Some(width) = width_floor_attribute(attributes, names) {
            thresholds.push(width.to_bits());
        }
    }
    for names in [&["up", "breakpoint"][..], &["down"][..]] {
        if let Some(width) = breakpoint_width_attribute(attributes, names) {
            thresholds.push(width.to_bits());
        }
    }
    if let Some((start, end)) = breakpoint_range_attribute(attributes, "between") {
        thresholds.extend([start.to_bits(), end.to_bits()]);
    }
    if let Some(query) = attributes.get("query").and_then(|value| value.as_str()) {
        let normalized = super::normalize_media_query(query);
        for prefix in ["minwidth:", "maxwidth:"] {
            if let Some(width) = width_query_threshold(&normalized, prefix) {
                thresholds.push(width.to_bits());
            }
        }
    }
    thresholds.sort_unstable();
    thresholds.dedup();
    thresholds
}

const RESPONSIVE_ATTRIBUTE_NAMES: &[&str] = &[
    "query",
    "matches",
    "defaultMatches",
    "default_matches",
    "display",
    "visibility",
    "visible",
    "container",
    "layout",
    "columns",
    "spacing",
    "columnSpacing",
    "column_spacing",
    "rowSpacing",
    "row_spacing",
    "direction",
    "sequential",
    "size",
    "offset",
    "min_width",
    "minWidth",
    "max_width",
    "maxWidth",
    "up",
    "breakpoint",
    "down",
    "between",
];

fn responsive_definition_for_metadata(
    metadata: &UiTemplateNodeMetadata,
    is_candidate: bool,
) -> Option<ResponsiveDefinition> {
    let attributes = RESPONSIVE_ATTRIBUTE_NAMES
        .iter()
        .filter_map(|name| {
            metadata
                .attributes
                .get(*name)
                .map(|value| ((*name).to_string(), value.clone()))
        })
        .collect::<BTreeMap<_, _>>();
    (is_candidate || !attributes.is_empty()).then_some(ResponsiveDefinition {
        component: metadata.component.clone(),
        attributes,
    })
}

fn decrement_threshold(counts: &mut BTreeMap<u32, usize>, threshold: u32) {
    if let Some(count) = counts.get_mut(&threshold) {
        *count = count.saturating_sub(1);
        if *count == 0 {
            counts.remove(&threshold);
        }
    }
}

fn normalized_width(width: f32) -> f32 {
    width.is_finite().then_some(width.max(0.0)).unwrap_or(0.0)
}

fn widths_crossed(previous: f32, next: f32, threshold: f32) -> bool {
    (previous < threshold && next >= threshold)
        || (previous >= threshold && next < threshold)
        || (previous <= threshold && next > threshold)
        || (previous > threshold && next <= threshold)
}
