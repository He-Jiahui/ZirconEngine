use crate::ui::template::{UiTemplateInstance, UiTemplateLoader, UiTemplateSurfaceBuilder};
use zircon_runtime_interface::ui::{
    event_ui::UiTreeId,
    layout::{
        UiAlignment, UiAlignment2D, UiContainerKind, UiGridBoxConfig, UiGridSlotPlacement,
        UiSlotKind, UiWrapBoxConfig,
    },
};

const GRID_FLOW_TEMPLATE_TOML: &str = r#"
version = 1

[root]
component = "GridBox"
control_id = "GridRoot"
attributes = { layout = { container = { kind = "GridBox", columns = 2, rows = 2, gap = 4.0 } } }
children = [
    { component = "IconButton", control_id = "GridChild", slot_attributes = { layout = { column = 1, row = 1, column_span = 1, row_span = 1, padding = { left = 4.0, top = 2.0, right = 6.0, bottom = 8.0 }, alignment = { horizontal = "End", vertical = "Center" } } }, attributes = { layout = { width = { min = 20.0, preferred = 20.0, max = 20.0, stretch = "Fixed" }, height = { min = 12.0, preferred = 12.0, max = 12.0, stretch = "Fixed" } } } },
    { component = "FlowBox", control_id = "FlowChild", attributes = { layout = { container = { kind = "FlowBox", gap = 3.0, item_min_width = 24.0 } } }, children = [
        { component = "IconButton", control_id = "FlowItem", slot_attributes = { layout = { order = 2, padding = { left = 1.0, top = 2.0, right = 3.0, bottom = 4.0 } } } }
    ] }
]
"#;

#[test]
fn template_builder_maps_grid_and_flow_slots_into_shared_runtime_layout_contract() {
    let document = UiTemplateLoader::load_toml_str(GRID_FLOW_TEMPLATE_TOML).unwrap();
    let instance = UiTemplateInstance::from_document(&document).unwrap();
    let surface = UiTemplateSurfaceBuilder::build_surface(
        UiTreeId::new("runtime.ui.template.grid"),
        &instance,
    )
    .unwrap();

    let root = surface
        .tree
        .nodes
        .values()
        .find(|node| {
            node.template_metadata
                .as_ref()
                .and_then(|metadata| metadata.control_id.as_deref())
                == Some("GridRoot")
        })
        .expect("grid root should be built");
    let grid_child = surface
        .tree
        .nodes
        .values()
        .find(|node| {
            node.template_metadata
                .as_ref()
                .and_then(|metadata| metadata.control_id.as_deref())
                == Some("GridChild")
        })
        .expect("grid child should be built");
    let flow_child = surface
        .tree
        .nodes
        .values()
        .find(|node| {
            node.template_metadata
                .as_ref()
                .and_then(|metadata| metadata.control_id.as_deref())
                == Some("FlowChild")
        })
        .expect("flow child should be built");
    let flow_item = surface
        .tree
        .nodes
        .values()
        .find(|node| {
            node.template_metadata
                .as_ref()
                .and_then(|metadata| metadata.control_id.as_deref())
                == Some("FlowItem")
        })
        .expect("flow item should be built");

    assert_eq!(
        root.container,
        UiContainerKind::GridBox(UiGridBoxConfig {
            columns: 2,
            rows: 2,
            column_gap: 4.0,
            row_gap: 4.0,
        })
    );
    assert_eq!(
        flow_child.container,
        UiContainerKind::WrapBox(UiWrapBoxConfig {
            horizontal_gap: 3.0,
            vertical_gap: 3.0,
            item_min_width: 24.0,
        })
    );

    let grid_slot = surface
        .tree
        .slots
        .iter()
        .find(|slot| slot.child_id == grid_child.node_id)
        .expect("grid child should carry a parent slot");
    let grid_flow_slot = surface
        .tree
        .slots
        .iter()
        .find(|slot| slot.child_id == flow_child.node_id)
        .expect("flow child should carry a parent slot");
    let flow_item_slot = surface
        .tree
        .slots
        .iter()
        .find(|slot| slot.child_id == flow_item.node_id)
        .expect("flow item should carry a parent slot");

    assert_eq!(grid_slot.kind, UiSlotKind::Grid);
    assert_eq!(
        grid_slot.grid_placement,
        Some(UiGridSlotPlacement::new(1, 1).with_span(1, 1))
    );
    assert_eq!(grid_slot.padding.left, 4.0);
    assert_eq!(
        grid_slot.alignment,
        UiAlignment2D::new(UiAlignment::End, UiAlignment::Center)
    );
    assert_eq!(grid_flow_slot.kind, UiSlotKind::Grid);
    assert_eq!(flow_item_slot.kind, UiSlotKind::Flow);
    assert_eq!(flow_item_slot.order, 2);
    assert_eq!(flow_item_slot.padding.bottom, 4.0);
}
