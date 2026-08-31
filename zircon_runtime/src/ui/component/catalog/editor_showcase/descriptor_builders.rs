use std::collections::BTreeMap;

use toml::Value;

use zircon_runtime_interface::ui::component::{
    UiComponentCategory, UiComponentDescriptor, UiComponentDescriptorKind, UiComponentEventKind,
    UiComponentLayoutRole, UiDefaultNodeTemplate, UiDragPayloadKind, UiDropPolicy,
    UiHostCapability, UiOptionDescriptor, UiPaletteMetadata, UiPropSchema, UiRenderCapability,
    UiSlotSchema, UiValue, UiValueKind, UiWidgetEditorFallback, UiWidgetFallbackPolicy,
    UiWidgetRuntimeFallback,
};

fn base_descriptor(
    id: &str,
    display_name: &str,
    category: UiComponentCategory,
    role: &str,
) -> UiComponentDescriptor {
    UiComponentDescriptor::new(id, display_name, category, role)
        .default_class(role.to_string())
        .requires_host_capability(UiHostCapability::Runtime)
        .requires_render_capability(UiRenderCapability::Primitive)
        .fallback_policy(UiWidgetFallbackPolicy::new(
            UiWidgetEditorFallback::Placeholder,
            UiWidgetRuntimeFallback::RejectNode,
        ))
}

pub(super) fn visual(id: &str, display_name: &str, role: &str) -> UiComponentDescriptor {
    base_descriptor(id, display_name, UiComponentCategory::Visual, role)
}

pub(super) fn input(id: &str, display_name: &str, role: &str) -> UiComponentDescriptor {
    base_descriptor(id, display_name, UiComponentCategory::Input, role)
        .requires_host_capability(UiHostCapability::PointerInput)
        .requires_host_capability(UiHostCapability::KeyboardNavigation)
}

pub(super) fn numeric(id: &str, display_name: &str, role: &str) -> UiComponentDescriptor {
    base_descriptor(id, display_name, UiComponentCategory::Numeric, role)
        .requires_host_capability(UiHostCapability::PointerInput)
        .requires_host_capability(UiHostCapability::KeyboardNavigation)
        .requires_host_capability(UiHostCapability::TextInput)
        .requires_render_capability(UiRenderCapability::Text)
}

pub(super) fn feedback(id: &str, display_name: &str, role: &str) -> UiComponentDescriptor {
    base_descriptor(id, display_name, UiComponentCategory::Feedback, role)
}

pub(super) fn collection(id: &str, display_name: &str, role: &str) -> UiComponentDescriptor {
    base_descriptor(id, display_name, UiComponentCategory::Collection, role)
        .requires_render_capability(UiRenderCapability::Scroll)
}

pub(super) fn editor_collection(id: &str, display_name: &str, role: &str) -> UiComponentDescriptor {
    collection(id, display_name, role)
        .descriptor_kind(UiComponentDescriptorKind::EditorOnly)
        .requires_host_capability(UiHostCapability::Editor)
}

pub(super) fn editor_feedback(id: &str, display_name: &str, role: &str) -> UiComponentDescriptor {
    feedback(id, display_name, role)
        .descriptor_kind(UiComponentDescriptorKind::EditorOnly)
        .requires_host_capability(UiHostCapability::Editor)
}

pub(super) fn container_descriptor(
    id: &str,
    display_name: &str,
    role: &str,
) -> UiComponentDescriptor {
    base_descriptor(id, display_name, UiComponentCategory::Container, role)
}

pub(super) fn layout_primitive(id: &str, display_name: &str, role: &str) -> UiComponentDescriptor {
    let descriptor = container_descriptor(id, display_name, role)
        .descriptor_kind(UiComponentDescriptorKind::Layout)
        .layout_role(layout_role_for(id))
        .default_node_template(layout_template(id));
    if id == "Space" {
        descriptor
    } else {
        descriptor.slot(UiSlotSchema::new("content").multiple(true))
    }
}

pub(super) fn popup_descriptor() -> UiComponentDescriptor {
    container_descriptor("Popup", "Popup", "popup")
        .layout_role(UiComponentLayoutRole::Popup)
        .with_prop(bool_prop("popup_open", false))
        .with_prop(UiPropSchema::new("popup_anchor_x", UiValueKind::Float))
        .with_prop(UiPropSchema::new("popup_anchor_y", UiValueKind::Float))
        .state(state_bool_prop("popup_open", false))
        .state(UiPropSchema::new("popup_anchor_x", UiValueKind::Float))
        .state(UiPropSchema::new("popup_anchor_y", UiValueKind::Float))
        .slot(UiSlotSchema::new("content").multiple(true))
        .events([
            UiComponentEventKind::OpenPopup,
            UiComponentEventKind::OpenPopupAt,
            UiComponentEventKind::ClosePopup,
        ])
}

fn layout_role_for(id: &str) -> UiComponentLayoutRole {
    match id {
        "Overlay" => UiComponentLayoutRole::Overlay,
        "HorizontalBox" | "VerticalBox" | "HorizontalGroup" | "VerticalGroup" | "ListView"
        | "FlexBox" | "FlowBox" | "ScrollableBox" => UiComponentLayoutRole::Flex,
        "GridBox" | "GridGroup" => UiComponentLayoutRole::Grid,
        "CanvasBox" => UiComponentLayoutRole::Canvas,
        "SizeBox" => UiComponentLayoutRole::Size,
        _ => UiComponentLayoutRole::Leaf,
    }
}

fn layout_template(widget_type: &str) -> UiDefaultNodeTemplate {
    let template = UiDefaultNodeTemplate::native(widget_type);
    match widget_type {
        "Container" | "Overlay" | "FlowBox" | "GridBox" => {
            template.with_layout(container_layout(widget_type))
        }
        "FlexBox" => template.with_layout(container_layout("FlowBox")),
        "GridGroup" => template.with_layout(container_layout("GridBox")),
        "HorizontalBox" | "VerticalBox" => template.with_layout(box_layout(widget_type)),
        "HorizontalGroup" => template.with_layout(box_layout("HorizontalBox")),
        "VerticalGroup" | "ListView" => template.with_layout(box_layout("VerticalBox")),
        "CanvasBox" => template.with_layout(container_layout("Free")),
        "SizeBox" => template.with_layout(container_layout("SizeBox")),
        "ScrollableBox" => template.with_layout(scrollable_layout()),
        _ => template,
    }
}

fn container_layout(kind: &str) -> BTreeMap<String, Value> {
    BTreeMap::from([(
        "container".to_string(),
        table_value([("kind", Value::String(kind.to_string()))]),
    )])
}

fn box_layout(kind: &str) -> BTreeMap<String, Value> {
    BTreeMap::from([(
        "container".to_string(),
        table_value([
            ("kind", Value::String(kind.to_string())),
            ("gap", Value::Integer(0)),
        ]),
    )])
}

fn scrollable_layout() -> BTreeMap<String, Value> {
    BTreeMap::from([(
        "container".to_string(),
        table_value([
            ("kind", Value::String("ScrollableBox".to_string())),
            ("axis", Value::String("Vertical".to_string())),
            ("gap", Value::Integer(0)),
            ("scrollbar_visibility", Value::String("Auto".to_string())),
        ]),
    )])
}

fn table_value<const N: usize>(entries: [(&str, Value); N]) -> Value {
    Value::Table(
        entries
            .into_iter()
            .map(|(key, value)| (key.to_string(), value))
            .collect(),
    )
}

pub(super) fn reference(
    id: &str,
    display_name: &str,
    role: &str,
    accepts: impl IntoIterator<Item = UiDragPayloadKind>,
) -> UiComponentDescriptor {
    base_descriptor(id, display_name, UiComponentCategory::Reference, role)
        .descriptor_kind(UiComponentDescriptorKind::EditorOnly)
        .requires_host_capability(UiHostCapability::Editor)
        .requires_host_capability(UiHostCapability::PointerInput)
        .with_prop(bool_prop("drop_hovered", false))
        .with_prop(bool_prop("active_drag_target", false))
        .drop_policy(UiDropPolicy::new(accepts))
        .event(UiComponentEventKind::DropHover)
        .event(UiComponentEventKind::ActiveDragTarget)
        .event(UiComponentEventKind::DropReference)
        .event(UiComponentEventKind::ClearReference)
        .event(UiComponentEventKind::LocateReference)
        .event(UiComponentEventKind::OpenReference)
}

pub(super) fn selection(
    id: &str,
    display_name: &str,
    role: &str,
    value_kind: UiValueKind,
) -> UiComponentDescriptor {
    base_descriptor(id, display_name, UiComponentCategory::Selection, role)
        .requires_host_capability(UiHostCapability::PointerInput)
        .requires_host_capability(UiHostCapability::KeyboardNavigation)
        .with_prop(UiPropSchema::new("value", value_kind))
        .with_prop(options_prop())
        .with_prop(value_text_prop())
        .with_prop(
            UiPropSchema::new("multiple", UiValueKind::Bool).default_value(UiValue::Bool(false)),
        )
        .with_prop(selection_state_prop())
        .with_prop(validation_level_prop())
        .with_prop(bool_prop("popup_open", false))
        .with_prop(option_ids_prop("disabled_options"))
        .with_prop(option_ids_prop("special_options"))
        .with_prop(option_ids_prop("focused_options"))
        .with_prop(option_ids_prop("hovered_options"))
        .with_prop(option_ids_prop("pressed_options"))
        .event(UiComponentEventKind::Focus)
        .state(state_bool_prop("focused", false))
        .state(state_bool_prop("popup_open", false))
        .state(state_bool_prop("selected", false))
        .events([
            UiComponentEventKind::OpenPopup,
            UiComponentEventKind::ClosePopup,
            UiComponentEventKind::SelectOption,
            UiComponentEventKind::ValueChanged,
        ])
}

pub(super) fn input_field(id: &str, display_name: &str) -> UiComponentDescriptor {
    input(id, display_name, "input-field")
        .requires_host_capability(UiHostCapability::TextInput)
        .requires_render_capability(UiRenderCapability::Text)
        .with_prop(
            UiPropSchema::new("value", UiValueKind::String)
                .default_value(UiValue::String(String::new())),
        )
        .with_prop(UiPropSchema::new("placeholder", UiValueKind::String))
        .with_prop(validation_level_prop())
        .state(state_string_prop("value"))
        .state(state_bool_prop("focused", false))
        .state(state_bool_prop("disabled", false))
        .events([
            UiComponentEventKind::ValueChanged,
            UiComponentEventKind::Commit,
            UiComponentEventKind::Focus,
        ])
}

pub(super) fn with_palette_metadata(descriptor: UiComponentDescriptor) -> UiComponentDescriptor {
    let descriptor = with_runtime_projection_metadata_props(descriptor);
    let template = if descriptor.default_node_template.is_empty() {
        default_template_from_descriptor(&descriptor)
    } else {
        descriptor.default_node_template.clone()
    };
    let sort_key = format!(
        "{:02}.{}",
        category_sort_key(descriptor.category),
        descriptor.display_name
    );
    let palette = UiPaletteMetadata::new(
        descriptor.display_name.clone(),
        descriptor.category,
        sort_key,
        template.clone(),
    );
    descriptor.default_node_template(template).palette(palette)
}

fn with_runtime_projection_metadata_props(
    descriptor: UiComponentDescriptor,
) -> UiComponentDescriptor {
    descriptor
        .with_prop(UiPropSchema::new("surface_variant", UiValueKind::String))
        .with_prop(UiPropSchema::new("button_variant", UiValueKind::String))
        .with_prop(UiPropSchema::new("font_size", UiValueKind::Float))
        .with_prop(UiPropSchema::new("font_weight", UiValueKind::Int))
        .with_prop(UiPropSchema::new("height", UiValueKind::Float))
        .with_prop(UiPropSchema::new("layout_padding_left", UiValueKind::Float))
        .with_prop(UiPropSchema::new(
            "layout_padding_right",
            UiValueKind::Float,
        ))
        .with_prop(UiPropSchema::new("layout_padding_top", UiValueKind::Float))
        .with_prop(UiPropSchema::new(
            "layout_padding_bottom",
            UiValueKind::Float,
        ))
        .with_prop(UiPropSchema::new("layout_spacing", UiValueKind::Float))
        .with_prop(UiPropSchema::new("layout_min_width", UiValueKind::Float))
        .with_prop(UiPropSchema::new("layout_min_height", UiValueKind::Float))
        .with_prop(UiPropSchema::new("layout_icon_size", UiValueKind::Float))
        .with_prop(UiPropSchema::new("input_interactive", UiValueKind::Bool))
        .with_prop(UiPropSchema::new("input_clickable", UiValueKind::Bool))
        .with_prop(UiPropSchema::new("input_hoverable", UiValueKind::Bool))
        .with_prop(UiPropSchema::new("input_focusable", UiValueKind::Bool))
}

fn default_template_from_descriptor(descriptor: &UiComponentDescriptor) -> UiDefaultNodeTemplate {
    let mut props = descriptor
        .prop_schema
        .iter()
        .filter_map(|schema| {
            schema
                .default_value
                .as_ref()
                .map(|value| (schema.name.clone(), value.to_toml()))
        })
        .collect::<BTreeMap<_, _>>();
    for (name, value) in &descriptor.default_props {
        let _ = props.insert(name.clone(), value.to_toml());
    }
    UiDefaultNodeTemplate::native(descriptor.id.as_str())
        .with_node_id_prefix(descriptor.role.as_str())
        .with_props(props)
}

fn category_sort_key(category: UiComponentCategory) -> u8 {
    match category {
        UiComponentCategory::Container => 0,
        UiComponentCategory::Visual => 1,
        UiComponentCategory::Input => 2,
        UiComponentCategory::Numeric => 3,
        UiComponentCategory::Selection => 4,
        UiComponentCategory::Reference => 5,
        UiComponentCategory::Collection => 6,
        UiComponentCategory::Feedback => 7,
    }
}

pub(super) fn text_prop() -> UiPropSchema {
    UiPropSchema::new("text", UiValueKind::String).default_value(UiValue::String(String::new()))
}

pub(super) fn state_text_prop() -> UiPropSchema {
    UiPropSchema::new("text", UiValueKind::String).default_value(UiValue::String(String::new()))
}

pub(super) fn state_string_prop(name: &str) -> UiPropSchema {
    UiPropSchema::new(name, UiValueKind::String)
}

pub(super) fn bool_value_prop(default: bool) -> UiPropSchema {
    UiPropSchema::new("value", UiValueKind::Bool).default_value(UiValue::Bool(default))
}

pub(super) fn bool_prop(name: &str, default: bool) -> UiPropSchema {
    UiPropSchema::new(name, UiValueKind::Bool).default_value(UiValue::Bool(default))
}

pub(super) fn int_prop(name: &str, default: i64) -> UiPropSchema {
    UiPropSchema::new(name, UiValueKind::Int).default_value(UiValue::Int(default))
}

pub(super) fn number_value_prop() -> UiPropSchema {
    UiPropSchema::new("value", UiValueKind::Float)
        .default_value(UiValue::Float(0.0))
        .range(0.0, 100.0)
        .step(1.0)
}

pub(super) fn value_text_prop() -> UiPropSchema {
    UiPropSchema::new("value_text", UiValueKind::String)
        .default_value(UiValue::String(String::new()))
}

pub(super) fn validation_level_prop() -> UiPropSchema {
    UiPropSchema::new("validation_level", UiValueKind::String)
}

pub(super) fn validation_message_prop() -> UiPropSchema {
    UiPropSchema::new("validation_message", UiValueKind::String)
}

pub(super) fn options_prop() -> UiPropSchema {
    UiPropSchema::new("options", UiValueKind::Array)
        .default_value(UiValue::Array(vec![
            UiValue::Enum("primary".to_string()),
            UiValue::Enum("secondary".to_string()),
            UiValue::Enum("tertiary".to_string()),
        ]))
        .with_options([
            UiOptionDescriptor::new("primary", "Primary", UiValue::Enum("primary".to_string())),
            UiOptionDescriptor::new(
                "secondary",
                "Secondary",
                UiValue::Enum("secondary".to_string()),
            )
            .disabled(true),
            UiOptionDescriptor::new(
                "tertiary",
                "Tertiary",
                UiValue::Enum("tertiary".to_string()),
            )
            .special_condition("mixed"),
        ])
}

pub(super) fn option_ids_prop(name: &str) -> UiPropSchema {
    UiPropSchema::new(name, UiValueKind::Array).default_value(UiValue::Array(Vec::new()))
}

pub(super) fn selection_state_prop() -> UiPropSchema {
    UiPropSchema::new("selection_state", UiValueKind::String)
}

pub(super) fn state_bool_prop(name: &str, default: bool) -> UiPropSchema {
    UiPropSchema::new(name, UiValueKind::Bool).default_value(UiValue::Bool(default))
}

pub(super) fn state_float_prop(name: &str, default: f64) -> UiPropSchema {
    UiPropSchema::new(name, UiValueKind::Float).default_value(UiValue::Float(default))
}

pub(super) fn state_int_prop(name: &str, default: i64) -> UiPropSchema {
    UiPropSchema::new(name, UiValueKind::Int).default_value(UiValue::Int(default))
}

pub(super) fn state_array_prop(name: &str) -> UiPropSchema {
    UiPropSchema::new(name, UiValueKind::Array)
}

pub(super) fn state_map_prop(name: &str) -> UiPropSchema {
    UiPropSchema::new(name, UiValueKind::Map)
}

pub(super) fn expanded_prop() -> UiPropSchema {
    UiPropSchema::new("expanded", UiValueKind::Bool).default_value(UiValue::Bool(true))
}

#[cfg(test)]
mod optimization_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use toml::Value;

    use super::table_value;

    #[test]
    fn optimization_batch_20260831gs_runtime574_table_value_preserves_entries() {
        let value = table_value([
            ("kind", Value::String("ScrollableBox".to_owned())),
            ("axis", Value::String("Vertical".to_owned())),
            ("gap", Value::Integer(0)),
            ("scrollbar_visibility", Value::String("Auto".to_owned())),
        ]);
        let table = value.as_table().expect("table value");
        assert_eq!(table["kind"].as_str(), Some("ScrollableBox"));
        assert_eq!(table["axis"].as_str(), Some("Vertical"));
        assert_eq!(table["gap"].as_integer(), Some(0));
        assert_eq!(table["scrollbar_visibility"].as_str(), Some("Auto"));
    }

    #[test]
    #[ignore = "release performance evidence"]
    fn optimization_batch_20260831gs_runtime574_table_value_move_benchmark() {
        const SAMPLE_PAIRS: usize = 21;
        const ITERATIONS: usize = 120_000;
        let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
        let mut checksum = 0usize;
        for sample_index in 0..SAMPLE_PAIRS {
            if sample_index % 2 == 0 {
                let (elapsed, value) = measure(ITERATIONS, legacy_table_len);
                legacy_ns.push(elapsed);
                checksum ^= value;
                let (elapsed, value) = measure(ITERATIONS, optimized_table_len);
                optimized_ns.push(elapsed);
                checksum ^= value;
            } else {
                let (elapsed, value) = measure(ITERATIONS, optimized_table_len);
                optimized_ns.push(elapsed);
                checksum ^= value;
                let (elapsed, value) = measure(ITERATIONS, legacy_table_len);
                legacy_ns.push(elapsed);
                checksum ^= value;
            }
        }
        let legacy_p95_ns = nearest_rank(&legacy_ns, 95);
        let optimized_p95_ns = nearest_rank(&optimized_ns, 95);
        assert!(
            optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(85),
            "moved table values P95 must be at least 15% below cloned values: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
        println!(
            "RUNTIME574_TABLE_VALUE_MOVE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} iterations={ITERATIONS} checksum={checksum} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_ns={} optimized_ns={}",
            join_samples(&legacy_ns),
            join_samples(&optimized_ns),
        );

        fn measure(iterations: usize, operation: fn() -> usize) -> (u128, usize) {
            let started = Instant::now();
            let mut checksum = 0usize;
            for _ in 0..iterations {
                checksum = checksum.wrapping_add(operation());
            }
            (started.elapsed().as_nanos(), black_box(checksum))
        }

        fn entries() -> [(&'static str, Value); 4] {
            [
                ("kind", Value::String("ScrollableBox".to_owned())),
                ("axis", Value::String("Vertical".to_owned())),
                ("gap", Value::Integer(0)),
                ("scrollbar_visibility", Value::String("Auto".to_owned())),
            ]
        }

        fn legacy_table_len() -> usize {
            let entries = entries();
            let value = Value::Table(
                entries
                    .iter()
                    .map(|(key, value)| ((*key).to_owned(), value.clone()))
                    .collect(),
            );
            black_box(value.as_table().map_or(0, |table| table.len()))
        }

        fn optimized_table_len() -> usize {
            let value = table_value(entries());
            black_box(value.as_table().map_or(0, |table| table.len()))
        }

        fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
            let mut ordered = samples.to_vec();
            ordered.sort_unstable();
            let rank = (ordered.len() * percentile).div_ceil(100).max(1);
            ordered[rank - 1]
        }

        fn join_samples(samples: &[u128]) -> String {
            samples
                .iter()
                .map(u128::to_string)
                .collect::<Vec<_>>()
                .join(",")
        }
    }
}
