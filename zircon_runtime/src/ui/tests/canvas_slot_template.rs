use crate::ui::template::{UiTemplateInstance, UiTemplateLoader, UiTemplateTreeBuilder};
use crate::ui::v2::{UiV2AssetLoader, UiV2DocumentCompiler, UiV2SurfaceBuilder};
use zircon_runtime_interface::ui::{
    event_ui::UiTreeId,
    layout::{Anchor, UiContainerKind, UiSlotKind},
};

#[test]
fn template_tree_builder_preserves_canvas_slot_stretch_anchor_contract() {
    let document = UiTemplateLoader::load_toml_str(
        r#"
version = 1

[root]
component = "Canvas"
control_id = "CanvasParent"
children = [
    { component = "Panel", control_id = "StretchChild", slot_attributes = { layout = { anchor = { x = 0.25, y = 0.0 }, anchor_max = { x = 0.75, y = 1.0 }, offset = { left = 8.0, top = 4.0, right = 12.0, bottom = 6.0 }, order = 7 } } }
]
"#,
    )
    .unwrap();
    let instance = UiTemplateInstance::from_document(&document).unwrap();

    let tree =
        UiTemplateTreeBuilder::build_tree(UiTreeId::new("canvas.slot.stretch.template"), &instance)
            .unwrap();
    let slot = tree.slots.first().expect("canvas child slot");
    let placement = slot.canvas_placement.expect("canvas placement");

    assert_eq!(slot.kind, UiSlotKind::Canvas);
    assert_eq!(slot.order, 7);
    assert_eq!(placement.anchor, Anchor::new(0.25, 0.0));
    assert_eq!(placement.anchor_max, Some(Anchor::new(0.75, 1.0)));
    assert_eq!(placement.resolved_anchor_max(), Anchor::new(0.75, 1.0));
}

#[test]
fn ui_v2_surface_builder_preserves_canvas_slot_stretch_anchor_contract() {
    let document = UiV2AssetLoader::load_toml_str(
        r#"
[asset]
kind = "view"
id = "asset://ui/tests/canvas_slot_stretch.v2.ui"
version = 2

[root]
node = "root"

[nodes.root]
component = "CanvasBox"
control_id = "CanvasParent"
children = [{ node = "stretch", slot = { layout = { anchor = { x = 0.1, y = 0.2 }, anchor_max = { x = 0.9, y = 0.8 }, offset = { left = 3.0, top = 5.0, right = 7.0, bottom = 11.0 } } } }]

[nodes.stretch]
component = "Panel"
control_id = "StretchChild"
"#,
    )
    .unwrap();
    let compiled = UiV2DocumentCompiler::compile(&document).unwrap();
    let surface = UiV2SurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("runtime.ui.v2.canvas_slot_stretch"),
        &document,
        &compiled,
    )
    .unwrap();
    let root = surface.tree.node(surface.tree.roots[0]).unwrap();
    let slot = surface.tree.slots.first().expect("v2 canvas child slot");
    let placement = slot.canvas_placement.expect("v2 canvas placement");

    assert_eq!(root.container, UiContainerKind::Canvas);
    assert_eq!(slot.kind, UiSlotKind::Canvas);
    assert_eq!(placement.anchor, Anchor::new(0.1, 0.2));
    assert_eq!(placement.anchor_max, Some(Anchor::new(0.9, 0.8)));
}
