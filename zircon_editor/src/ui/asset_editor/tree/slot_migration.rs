use std::collections::BTreeMap;

use toml::Value;
use zircon_runtime_interface::ui::{
    layout::UiSlotKind,
    template::{UiNodeDefinition, UiNodeDefinitionKind},
};

/// Edge fields cross a parent change only when both native slot families consume
/// them. A matching family retains its existing source extension fields as well.
pub(super) fn migrate_native_slot_attributes(
    source_parent: &UiNodeDefinition,
    target_parent: &UiNodeDefinition,
    attributes: &BTreeMap<String, Value>,
) -> BTreeMap<String, Value> {
    let source = declared_native_slot_kind(source_parent);
    let target = declared_native_slot_kind(target_parent);
    let mut migrated = if source.is_some() && source == target {
        attributes.clone()
    } else {
        let Some(layout) = attributes.get("layout").and_then(Value::as_table) else {
            return BTreeMap::new();
        };
        let mut portable = toml::map::Map::new();
        for key in ["padding", "alignment", "order"] {
            copy_field(layout, &mut portable, key);
        }
        if matches!(source, Some(UiSlotKind::Free | UiSlotKind::Canvas))
            && matches!(target, Some(UiSlotKind::Free | UiSlotKind::Canvas))
        {
            for key in [
                "anchor",
                "anchor_max",
                "pivot",
                "position",
                "offset",
                "auto_size",
            ] {
                copy_field(layout, &mut portable, key);
            }
        }
        if matches!(source, Some(UiSlotKind::Overlay | UiSlotKind::Canvas))
            && matches!(target, Some(UiSlotKind::Overlay | UiSlotKind::Canvas))
        {
            copy_field(layout, &mut portable, "z_order");
        }
        if portable.is_empty() {
            // An empty table would disable implicit Grid placement at the new
            // parent; keep absence when there are no transferable fields.
            BTreeMap::new()
        } else {
            BTreeMap::from([("layout".to_string(), Value::Table(portable))])
        }
    };
    // The v2 serializer uses entry.or_insert for the admitted new mount. Neither
    // old spelling may override it after a same-family move.
    migrated.remove("name");
    migrated.remove("slot_name");
    migrated
}

fn copy_field(
    source: &toml::map::Map<String, Value>,
    target: &mut toml::map::Map<String, Value>,
    key: &str,
) {
    if let Some(value) = source.get(key) {
        target.insert(key.to_string(), value.clone());
    }
}

fn declared_native_slot_kind(node: &UiNodeDefinition) -> Option<UiSlotKind> {
    if !matches!(node.kind, UiNodeDefinitionKind::Native) {
        return None;
    }
    let widget_type = node.widget_type.as_deref()?;
    let kind = if let Some(container) = node
        .layout
        .as_ref()
        .and_then(|layout| layout.get("container"))
    {
        container
            .as_table()?
            .get("kind")
            .and_then(Value::as_str)
            .unwrap_or(widget_type)
    } else {
        widget_type
    };
    // Source compatibility only, using v2 surface_tree/layout.rs names. Dynamic,
    // MUI-dependent and unknown declarations do not acquire a guessed family.
    Some(match kind {
        "Free" => UiSlotKind::Free,
        "Canvas" | "CanvasBox" => UiSlotKind::Canvas,
        "Container" | "Block" | "BlockBox" | "SizeBox" => UiSlotKind::Container,
        "Overlay" => UiSlotKind::Overlay,
        "HorizontalBox" | "HorizontalGroup" | "VerticalBox" | "VerticalGroup" | "ListView" => {
            UiSlotKind::Linear
        }
        "ScrollableBox" => UiSlotKind::Scrollable,
        "FlowBox" | "FlexBox" | "WrapBox" | "Masonry" | "MasonryBox" => UiSlotKind::Flow,
        "GridBox" | "GridGroup" => UiSlotKind::Grid,
        _ => return None,
    })
}
