use crate::ui::{
    surface::{UiPropertyMutationRequest, UiPropertyMutationStatus, UiSurface},
    tree::{UiRuntimeTreeAccessExt, UiRuntimeTreeLayoutExt},
};
use zircon_runtime_interface::ui::{
    component::UiValue,
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::{AxisConstraint, BoxConstraints, StretchMode, UiSize},
    tree::{UiDirtyFlags, UiInputPolicy, UiTemplateNodeMetadata, UiTreeNode},
};

#[test]
fn surface_dirty_mui_virtual_range_aliases_rebuild_layout_and_visible_range() {
    let mut surface = test_surface("runtime.ui.dirty_mui.virtual_range");
    attach_component_metadata(
        &mut surface,
        "DataGrid",
        r#"
rowCount = 200
rowHeight = 46.0
overscanCount = 5
"#,
    );
    surface.clear_dirty_flags();

    let mutation = surface
        .mutate_property(UiPropertyMutationRequest::new(
            button_id(),
            "rowHeight",
            UiValue::Float(52.0),
        ))
        .unwrap();

    assert_eq!(mutation.status, UiPropertyMutationStatus::Accepted);
    assert_mui_dirty(
        mutation.invalidation.dirty,
        UiDirtyFlags {
            layout: true,
            hit_test: true,
            render: true,
            input: true,
            visible_range: true,
            ..Default::default()
        },
    );
    assert_mui_dirty(
        surface.dirty_flags(),
        UiDirtyFlags {
            layout: true,
            hit_test: true,
            render: true,
            input: true,
            visible_range: true,
            ..Default::default()
        },
    );
}

#[test]
fn surface_dirty_mui_customization_aliases_rebuild_layout_style_and_input() {
    let mut surface = test_surface("runtime.ui.dirty_mui.customization");
    attach_component_metadata(
        &mut surface,
        "Button",
        r#"
sx = { border_width = 2.0 }
slotProps = { root = { disabled = false } }
className = "before"
"#,
    );
    surface.clear_dirty_flags();

    let mutation = surface
        .mutate_property(UiPropertyMutationRequest::new(
            button_id(),
            "slotProps",
            UiValue::Map(Default::default()),
        ))
        .unwrap();

    assert_eq!(mutation.status, UiPropertyMutationStatus::Accepted);
    assert_mui_dirty(mutation.invalidation.dirty, style_input_dirty());
    assert_mui_dirty(surface.dirty_flags(), style_input_dirty());
}

#[test]
fn surface_dirty_mui_feedback_metadata_rebuilds_layout_text_and_input() {
    let mut surface = test_surface("runtime.ui.dirty_mui.feedback");
    attach_component_metadata(
        &mut surface,
        "Snackbar",
        r#"
message = "Saved"
autoHideDuration = 4000
anchorOrigin = { vertical = "bottom", horizontal = "center" }
"#,
    );
    surface.clear_dirty_flags();

    let mutation = surface
        .mutate_property(UiPropertyMutationRequest::new(
            button_id(),
            "autoHideDuration",
            UiValue::Int(2500),
        ))
        .unwrap();

    assert_eq!(mutation.status, UiPropertyMutationStatus::Accepted);
    assert_mui_dirty(mutation.invalidation.dirty, style_input_dirty());
    assert_mui_dirty(surface.dirty_flags(), style_input_dirty());
}

fn attach_component_metadata(surface: &mut UiSurface, component: &str, attributes: &str) {
    let metadata = UiTemplateNodeMetadata {
        component: component.to_string(),
        control_id: Some(format!("DirtyDomain{component}")),
        attributes: toml::from_str(attributes).unwrap(),
        ..Default::default()
    };
    surface
        .tree
        .node_mut(button_id())
        .expect("button node should exist")
        .template_metadata = Some(metadata);
}

fn assert_mui_dirty(actual: UiDirtyFlags, expected: UiDirtyFlags) {
    assert_eq!(actual, expected);
}

fn style_input_dirty() -> UiDirtyFlags {
    UiDirtyFlags {
        layout: true,
        hit_test: true,
        render: true,
        text: true,
        input: true,
        ..Default::default()
    }
}

fn test_surface(tree_name: &str) -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new(tree_name));
    surface.tree.insert_root(
        UiTreeNode::new(root_id(), UiNodePath::new("root")).with_constraints(BoxConstraints {
            width: fixed_constraint(120.0),
            height: fixed_constraint(60.0),
        }),
    );
    surface
        .tree
        .insert_child(
            root_id(),
            UiTreeNode::new(button_id(), UiNodePath::new("root/button"))
                .with_constraints(BoxConstraints {
                    width: fixed_constraint(80.0),
                    height: fixed_constraint(24.0),
                })
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(pointer_state()),
        )
        .unwrap();
    surface.compute_layout(root_size()).unwrap();
    surface.clear_dirty_flags();
    surface
}

fn fixed_constraint(size: f32) -> AxisConstraint {
    AxisConstraint {
        min: size,
        max: size,
        preferred: size,
        priority: 100,
        weight: 1.0,
        stretch_mode: StretchMode::Fixed,
    }
}

fn pointer_state() -> UiStateFlags {
    UiStateFlags {
        visible: true,
        enabled: true,
        clickable: true,
        hoverable: true,
        focusable: true,
        pressed: false,
        checked: false,
        dirty: false,
    }
}

fn root_id() -> UiNodeId {
    UiNodeId::new(1)
}

fn button_id() -> UiNodeId {
    UiNodeId::new(2)
}

fn root_size() -> UiSize {
    UiSize::new(120.0, 60.0)
}
