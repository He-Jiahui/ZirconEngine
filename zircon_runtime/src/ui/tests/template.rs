use crate::ui::template::{UiTemplateInstance, UiTemplateSurfaceBuilder, UiTemplateTreeBuilder};
use serde::Deserialize;
use toml::Value;
use zircon_runtime_interface::ui::{
    event_ui::UiTreeId,
    layout::{
        AxisConstraint, StretchMode, UiAlignment, UiAxis, UiContainerKind, UiFrame,
        UiLinearBoxConfig, UiLinearSlotSizeRule, UiScrollState, UiScrollableBoxConfig,
        UiScrollbarVisibility, UiSize, UiSizeBoxConfig, UiSlotKind, UiVirtualListConfig,
        UiVirtualListWindow,
    },
    template::UiTemplateNode,
    tree::UiInputPolicy,
};

const WORKBENCH_TEMPLATE_TOML: &str = r#"
version = 1

[root]
component = "WorkbenchShell"
children = [
    { component = "UiHostToolbar", children = [
        { component = "IconButton", control_id = "OpenProject", bindings = [{ id = "WorkbenchMenuBar/OpenProject", event = "Click", route = "MenuAction.OpenProject" }], attributes = { icon = "folder-open-outline", label = "Open" } },
        { component = "IconButton", control_id = "SaveProject", bindings = [{ id = "WorkbenchMenuBar/SaveProject", event = "Click", route = "MenuAction.SaveProject" }], attributes = { icon = "save-outline", label = "Save" } }
    ] },
    { component = "ActivityRail", control_id = "ActivityRailRoot" },
    { component = "ToolWindowStack", control_id = "DocumentHost" }
]
"#;

const SHARED_CONTAINER_TEMPLATE_TOML: &str = r#"
version = 1

[root]
component = "ScrollableBox"
control_id = "ScrollRoot"
children = [
    { component = "HorizontalBox", control_id = "Row" },
    { component = "Space", control_id = "Gap" },
    { component = "IconButton", control_id = "InteractiveLeaf", bindings = [{ id = "Demo/Click", event = "Click", route = "Demo.Click" }], attributes = { label = "Demo" } }
]
"#;

const LAYOUT_CONTRACT_TEMPLATE_TOML: &str = r#"
version = 1

[root]
component = "WorkspaceShell"
control_id = "WorkspaceShellRoot"
attributes = { layout = { container = { kind = "VerticalBox", gap = 12.0 }, width = { stretch = "Stretch" }, height = { stretch = "Stretch" }, clip = true } }
children = [
    { component = "UiHostToolbar", control_id = "Toolbar", attributes = { layout = { container = { kind = "HorizontalBox", gap = 8.0 }, width = { stretch = "Stretch" }, height = { min = 48.0, preferred = 48.0, max = 48.0, stretch = "Fixed" } } }, children = [
        { component = "IconButton", control_id = "ToolbarAction", bindings = [{ id = "Toolbar/Action", event = "Click", route = "Toolbar.Action" }], attributes = { label = "Action", layout = { width = { min = 120.0, preferred = 120.0, max = 120.0, stretch = "Fixed" }, height = { min = 32.0, preferred = 32.0, max = 32.0, stretch = "Fixed" } } } }
    ] },
    { component = "ViewportHost", control_id = "ViewportHost", attributes = { layout = { container = { kind = "Overlay" }, width = { stretch = "Stretch" }, height = { stretch = "Stretch" } } }, children = [
        { component = "OverlayBadge", control_id = "OverlayBadge", attributes = { layout = { width = { min = 60.0, preferred = 60.0, max = 60.0, stretch = "Fixed" }, height = { min = 24.0, preferred = 24.0, max = 24.0, stretch = "Fixed" }, anchor = { x = 1.0, y = 0.0 }, pivot = { x = 1.0, y = 0.0 }, position = { x = -16.0, y = 12.0 }, z_index = 4 } } }
    ] },
    { component = "AssetList", control_id = "AssetList", attributes = { layout = { container = { kind = "ScrollableBox", axis = "Vertical", gap = 6.0, scrollbar_visibility = "Always", virtualization = { item_extent = 28.0, overscan = 2 } }, width = { stretch = "Stretch" }, height = { min = 120.0, preferred = 120.0, max = 120.0, stretch = "Fixed" }, clip = true } }, children = [
        { component = "AssetRow", control_id = "AssetRow0", attributes = { layout = { width = { stretch = "Stretch" }, height = { min = 28.0, preferred = 28.0, max = 28.0, stretch = "Fixed" } } } },
        { component = "AssetRow", control_id = "AssetRow1", attributes = { layout = { width = { stretch = "Stretch" }, height = { min = 28.0, preferred = 28.0, max = 28.0, stretch = "Fixed" } } } },
        { component = "AssetRow", control_id = "AssetRow2", attributes = { layout = { width = { stretch = "Stretch" }, height = { min = 28.0, preferred = 28.0, max = 28.0, stretch = "Fixed" } } } },
        { component = "AssetRow", control_id = "AssetRow3", attributes = { layout = { width = { stretch = "Stretch" }, height = { min = 28.0, preferred = 28.0, max = 28.0, stretch = "Fixed" } } } },
        { component = "AssetRow", control_id = "AssetRow4", attributes = { layout = { width = { stretch = "Stretch" }, height = { min = 28.0, preferred = 28.0, max = 28.0, stretch = "Fixed" } } } }
    ] }
]
"#;

const SLOT_CONTRACT_TEMPLATE_TOML: &str = r#"
version = 1

[root]
component = "HorizontalBox"
control_id = "SlotParent"
children = [
    { component = "IconButton", control_id = "PrimaryAction", slot_attributes = { layout = { width = { min = 96.0, preferred = 96.0, max = 96.0, stretch = "Fixed" }, linear_size = { rule = "StretchContent", value = 2.0, shrink_value = 0.5, min = 48.0, max = 160.0 }, padding = { left = 4.0, top = 6.0, right = 8.0, bottom = 10.0 }, alignment = { horizontal = "Fill", vertical = "Center" }, order = 3, z_order = 21 } }, attributes = { layout = { height = { min = 32.0, preferred = 32.0, max = 32.0, stretch = "Fixed" } } } }
]
"#;

const OVERLAY_SLOT_CONTRACT_TEMPLATE_TOML: &str = r#"
version = 1

[root]
component = "ViewportHost"
control_id = "OverlayParent"
attributes = { layout = { container = { kind = "Overlay" } } }
children = [
    { component = "OverlayPanel", control_id = "BackgroundLayer", slot_attributes = { layout = { z_order = -4, order = 2, alignment = { horizontal = "Fill", vertical = "Fill" } } } },
    { component = "OverlayBadge", control_id = "ForegroundLayer", slot_attributes = { layout = { z_order = 16, order = 1, padding = { left = 4.0, top = 6.0 } } }, attributes = { layout = { z_index = 99 } } }
]
"#;

const CANVAS_FREE_SLOT_CONTRACT_TEMPLATE_TOML: &str = r#"
version = 1

[root]
component = "Canvas"
control_id = "CanvasParent"
children = [
    { component = "CanvasBadge", control_id = "FreePlaced", slot_attributes = { layout = { anchor = { x = 1.0, y = 0.25 }, pivot = { x = 1.0, y = 0.5 }, position = { x = -24.0, y = 16.0 }, offset = { left = 2.0, top = 4.0, right = 120.0, bottom = 40.0 }, auto_size = true, order = 4 } }, attributes = { layout = { width = { min = 60.0, preferred = 60.0, max = 60.0, stretch = "Fixed" }, height = { min = 20.0, preferred = 20.0, max = 20.0, stretch = "Fixed" } } } }
]
"#;

const NON_CANVAS_FREE_SLOT_PLACEMENT_TEMPLATE_TOML: &str = r#"
version = 1

[root]
component = "HorizontalBox"
control_id = "LinearParent"
children = [
    { component = "ToolbarAction", control_id = "LinearChild", slot_attributes = { layout = { anchor = { x = 1.0, y = 0.25 }, pivot = { x = 1.0, y = 0.5 }, position = { x = -24.0, y = 16.0 }, offset = { left = 2.0, top = 4.0, right = 120.0, bottom = 40.0 }, auto_size = true, order = 4 } } }
]
"#;

const SPACE_SLOT_PLACEMENT_TEMPLATE_TOML: &str = r#"
version = 1

[root]
component = "Space"
control_id = "SpaceParent"
children = [
    { component = "Decorative", control_id = "SpaceChild", slot_attributes = { layout = { anchor = { x = 0.5, y = 0.5 }, position = { x = 8.0, y = 12.0 }, offset = { left = 1.0, top = 2.0, right = 3.0, bottom = 4.0 }, auto_size = true } } }
]
"#;

mod interaction_bindings;
mod layout_compute;
mod slot_contracts;
mod surface_containers;

fn tree_from_root_toml(root: String) -> zircon_runtime_interface::ui::tree::UiTree {
    let instance = compiled_instance_from_toml(&format!("root = {root}"));
    UiTemplateTreeBuilder::build_tree(UiTreeId::new("interaction.metadata"), &instance).unwrap()
}

fn root_with_inline_node(node: &str) -> String {
    node.to_string()
}

#[derive(Deserialize)]
struct CompiledTemplateFixture {
    root: UiTemplateNode,
}

pub(super) fn compiled_instance_from_toml(source: &str) -> UiTemplateInstance {
    let fixture: CompiledTemplateFixture = toml::from_str(source).unwrap();
    assert_compiled_node(&fixture.root);
    UiTemplateInstance::new(fixture.root)
}

fn assert_compiled_node(node: &UiTemplateNode) {
    assert!(
        node.component.is_some() && node.template.is_none() && node.slot.is_none(),
        "compiled template fixtures must contain native component nodes only"
    );
    assert!(
        node.slots.is_empty(),
        "compiled template fixtures cannot contain unresolved slot fills"
    );
    for child in &node.children {
        assert_compiled_node(child);
    }
}

fn only_root_node(
    tree: &zircon_runtime_interface::ui::tree::UiTree,
) -> &zircon_runtime_interface::ui::tree::UiTreeNode {
    assert_eq!(tree.roots.len(), 1);
    tree.node(tree.roots[0]).unwrap()
}
