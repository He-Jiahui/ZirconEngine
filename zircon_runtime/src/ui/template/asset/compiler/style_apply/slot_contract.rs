use std::collections::BTreeMap;

use toml::Value;
use zircon_runtime_interface::ui::template::UiTemplateNode;

use super::super::value_normalizer::merge_value_maps;
use super::{
    append_class, map_attribute, map_attribute_any, mui_collection_classes,
    mui_display_surface_classes, mui_form_classes, mui_layout_classes, mui_navigation_classes,
    mui_selection_classes, mui_surface_child_classes, mui_x_classes, nested_map_attribute_any,
    string_list_from_value, value_as_map,
};

pub(in super::super) fn apply_mui_root_slot_props_to_node(node: &mut UiTemplateNode) {
    let root_props = nested_map_attribute_any(node, &["mui_slot_props", "slotProps"], "root");
    if root_props.is_empty() {
        return;
    };
    merge_value_maps(&mut node.attributes, &root_props);
}

pub(in super::super) fn apply_mui_child_slot_props(node: &mut UiTemplateNode) {
    let slot_props = map_attribute_any(node, &["mui_slot_props", "slotProps"]);
    let slot_components = map_attribute_any(node, &["mui_slots", "slots"]);
    let slot_classes = map_attribute(node, "classes");
    let owner_prefix = node
        .component
        .as_deref()
        .filter(|component| !component.is_empty())
        .map(|component| format!("Mui{component}"));
    let owner_component = node.component.clone().unwrap_or_default();
    let owner_attributes = &node.attributes;
    if slot_props.is_empty()
        && slot_components.is_empty()
        && slot_classes.is_empty()
        && owner_prefix.is_none()
    {
        return;
    }

    for child in &mut node.children {
        if owner_component == "Skeleton" {
            mui_display_surface_classes::append_skeleton_child_metadata(child);
        }
        let Some(slot_name) = mui_slot_name(child).map(str::to_owned) else {
            continue;
        };
        apply_mui_slot_contract_to_child(
            child,
            &slot_name,
            &owner_component,
            owner_attributes,
            owner_prefix.as_deref(),
            &slot_props,
            &slot_components,
            &slot_classes,
        );
    }

    for (slot_name, children) in &mut node.slots {
        for child in children {
            if owner_component == "Skeleton" {
                mui_display_surface_classes::append_skeleton_child_metadata(child);
            }
            child
                .slot_attributes
                .entry("mui_slot".to_string())
                .or_insert_with(|| Value::String(slot_name.clone()));
            apply_mui_slot_contract_to_child(
                child,
                slot_name,
                &owner_component,
                owner_attributes,
                owner_prefix.as_deref(),
                &slot_props,
                &slot_components,
                &slot_classes,
            );
        }
    }
}

fn apply_mui_slot_contract_to_child(
    child: &mut UiTemplateNode,
    slot_name: &str,
    owner_component: &str,
    owner_attributes: &BTreeMap<String, Value>,
    owner_prefix: Option<&str>,
    slot_props: &BTreeMap<String, Value>,
    slot_components: &BTreeMap<String, Value>,
    slot_classes: &BTreeMap<String, Value>,
) {
    if let Some(prefix) = owner_prefix {
        append_class(&mut child.classes, format!("{prefix}-{slot_name}"));
    }
    append_mui_owner_slot_utility_classes(child, owner_component, owner_attributes, slot_name);
    if let Some(component) = slot_components
        .get(slot_name)
        .and_then(Value::as_str)
        .filter(|component| !component.trim().is_empty())
    {
        let _ = child.attributes.insert(
            "mui_slot_component".to_string(),
            Value::String(component.trim().to_string()),
        );
    }
    if let Some(props) = slot_props.get(slot_name).and_then(value_as_map) {
        merge_value_maps(&mut child.attributes, &props);
    }
    for class_name in string_list_from_value(slot_classes.get(slot_name)) {
        append_class(&mut child.classes, class_name);
    }
}

fn append_mui_owner_slot_utility_classes(
    child: &mut UiTemplateNode,
    owner_component: &str,
    owner_attributes: &BTreeMap<String, Value>,
    slot_name: &str,
) {
    if mui_layout_classes::append_layout_slot_classes(
        child,
        owner_component,
        owner_attributes,
        slot_name,
    ) {
        return;
    }
    if mui_form_classes::append_form_slot_classes(
        child,
        owner_component,
        owner_attributes,
        slot_name,
    ) {
        return;
    }
    if mui_selection_classes::append_slot_classes(
        child,
        owner_component,
        owner_attributes,
        slot_name,
    ) {
        return;
    }
    if mui_collection_classes::append_slot_classes(
        child,
        owner_component,
        owner_attributes,
        slot_name,
    ) {
        return;
    }
    if mui_x_classes::append_slot_classes(child, owner_component, owner_attributes, slot_name) {
        return;
    }
    if mui_surface_child_classes::append_slot_classes(
        child,
        owner_component,
        owner_attributes,
        slot_name,
    ) {
        return;
    }
    if mui_display_surface_classes::append_slot_classes(
        child,
        owner_component,
        owner_attributes,
        slot_name,
    ) {
        return;
    }

    match (owner_component, slot_name) {
        ("BottomNavigationAction", "label") => {
            mui_navigation_classes::append_bottom_navigation_action_label_slot_classes(
                child,
                owner_attributes,
            )
        }
        ("StepConnector", "line") => {
            mui_navigation_classes::append_step_connector_line_slot_classes(child, owner_attributes)
        }
        ("StepLabel", "label" | "iconContainer" | "labelContainer") => {
            mui_navigation_classes::append_step_label_slot_classes(
                child,
                owner_attributes,
                slot_name,
            )
        }
        ("Tabs", "list" | "scroller" | "scrollButtons") => {
            mui_navigation_classes::append_tabs_slot_classes(child, owner_attributes, slot_name)
        }
        ("TransferList", "source" | "target" | "actions") => {
            mui_navigation_classes::append_transfer_list_slot_classes(
                child,
                owner_attributes,
                slot_name,
            )
        }
        _ => {}
    }
}

pub(in super::super) fn mui_slot_name(node: &UiTemplateNode) -> Option<&str> {
    borrowed_string_from_map(&node.slot_attributes, "mui_slot")
        .or_else(|| borrowed_string_from_map(&node.attributes, "mui_slot"))
}

fn borrowed_string_from_map<'value>(
    values: &'value BTreeMap<String, Value>,
    name: &str,
) -> Option<&'value str> {
    values
        .get(name)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    #[test]
    fn optimization_batch_dy_borrowed_mui_slot_name_preserves_precedence_and_trimming() {
        let mut node = UiTemplateNode::default();
        node.attributes.insert(
            "mui_slot".to_owned(),
            Value::String(" attribute-slot ".to_owned()),
        );
        node.slot_attributes.insert(
            "mui_slot".to_owned(),
            Value::String("  primary-slot  ".to_owned()),
        );

        let slot = mui_slot_name(&node).expect("slot name");
        let stored = node
            .slot_attributes
            .get("mui_slot")
            .and_then(Value::as_str)
            .expect("stored slot")
            .trim();

        assert_eq!(slot, "primary-slot");
        assert_eq!(slot.as_ptr(), stored.as_ptr());
    }

    #[test]
    fn optimization_batch_dy_mui_slot_name_uses_borrowed_storage() {
        let production = include_str!("slot_contract.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("slot contract production source");
        let lookup = production
            .split("fn mui_slot_name")
            .nth(1)
            .expect("MUI slot-name lookup");

        assert!(lookup.contains("Option<&str>"));
        assert!(lookup.contains("borrowed_string_from_map"));
        assert!(!lookup.contains("Option<String>"));
        assert!(!lookup.contains("str::to_string"));
    }

    #[test]
    #[ignore = "release-only alternating p95 performance gate"]
    fn optimization_batch_dy_borrowed_mui_slot_name_p95() {
        const SAMPLE_PAIRS: usize = 17;
        const LOOKUPS_PER_SAMPLE: usize = 16;
        const NODE_COUNT: usize = 2_048;

        let nodes = (0..NODE_COUNT)
            .map(|index| {
                let mut node = UiTemplateNode::default();
                node.slot_attributes.insert(
                    "mui_slot".to_owned(),
                    Value::String(format!(
                        "  slot-{index:04}-{}  ",
                        "long_component_slot_name/".repeat(16)
                    )),
                );
                node
            })
            .collect::<Vec<_>>();
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for sample_index in 0..SAMPLE_PAIRS {
            if sample_index % 2 == 0 {
                legacy_samples.push(measure_slot_lookups(&nodes, LOOKUPS_PER_SAMPLE, false));
                optimized_samples.push(measure_slot_lookups(&nodes, LOOKUPS_PER_SAMPLE, true));
            } else {
                optimized_samples.push(measure_slot_lookups(&nodes, LOOKUPS_PER_SAMPLE, true));
                legacy_samples.push(measure_slot_lookups(&nodes, LOOKUPS_PER_SAMPLE, false));
            }
        }

        let legacy_p95 = p95(&mut legacy_samples);
        let optimized_p95 = p95(&mut optimized_samples);
        println!(
            "RUNTIME433_BORROWED_MUI_SLOT_NAME_BENCH_V1 lookups_per_sample={LOOKUPS_PER_SAMPLE} node_count={NODE_COUNT} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} ratio={:.4}",
            optimized_p95 as f64 / legacy_p95.max(1) as f64
        );
        assert!(
            optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(70),
            "borrowed MUI slot-name p95 {optimized_p95}ns exceeded 70% of legacy {legacy_p95}ns"
        );
    }

    fn measure_slot_lookups(
        nodes: &[UiTemplateNode],
        lookup_count: usize,
        optimized: bool,
    ) -> u128 {
        let started_at = Instant::now();
        let mut checksum = 0_usize;
        for _ in 0..lookup_count {
            for node in nodes {
                if optimized {
                    let slot = mui_slot_name(node).expect("optimized slot name");
                    checksum = checksum.wrapping_add(black_box(slot).len());
                } else {
                    let slot = legacy_mui_slot_name(node).expect("legacy slot name");
                    checksum = checksum.wrapping_add(black_box(slot).len());
                }
            }
        }
        black_box(checksum);
        started_at.elapsed().as_nanos()
    }

    fn legacy_mui_slot_name(node: &UiTemplateNode) -> Option<String> {
        legacy_string_from_map(&node.slot_attributes, "mui_slot")
            .or_else(|| legacy_string_from_map(&node.attributes, "mui_slot"))
    }

    fn legacy_string_from_map(values: &BTreeMap<String, Value>, name: &str) -> Option<String> {
        values
            .get(name)
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
    }

    fn p95(samples: &mut [u128]) -> u128 {
        samples.sort_unstable();
        samples[(samples.len() * 95).div_ceil(100).saturating_sub(1)]
    }
}
