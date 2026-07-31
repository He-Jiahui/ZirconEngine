use crate::ui::surface::UiSurface;
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiTreeId},
    layout::{AxisConstraint, BoxConstraints, DesiredSize, LayoutBoundary, StretchMode, UiSize},
    surface::{UiRenderCommandKind, UiVisualAssetRef},
    tree::{UiTemplateNodeMetadata, UiTreeNode},
};

mod asset_icon_roles;
mod button_icon_metrics;
mod constraints_children;
mod field_values;
mod row_label_metrics;

#[test]
fn material_layout_resolves_metrics_without_preflight_attribute_rescan() {
    let source = include_str!("../layout/pass/material.rs");

    assert!(!source.contains("has_layout_attribute"));
    assert!(source.contains("let Some(metrics) = MaterialLayoutMetrics::resolve(metadata) else"));
}

fn measure_material_leaf(
    component: &str,
    attributes: &str,
    constraints: BoxConstraints,
) -> DesiredSize {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.material_layout"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_layout_boundary(LayoutBoundary::ContentDriven),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/control"))
                .with_layout_boundary(LayoutBoundary::ContentDriven)
                .with_constraints(constraints)
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: component.to_string(),
                    attributes: toml::from_str(attributes).unwrap(),
                    ..Default::default()
                }),
        )
        .unwrap();

    surface.compute_layout(UiSize::new(240.0, 120.0)).unwrap();
    surface
        .tree
        .node(UiNodeId::new(2))
        .unwrap()
        .layout_cache
        .desired_size
}

fn render_material_leaf_command(
    component: &str,
    attributes: &str,
) -> zircon_runtime_interface::ui::surface::UiRenderCommand {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.material_layout.render"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_layout_boundary(LayoutBoundary::ContentDriven),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/control"))
                .with_layout_boundary(LayoutBoundary::ContentDriven)
                .with_constraints(intrinsic_constraints())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: component.to_string(),
                    attributes: toml::from_str(attributes).unwrap(),
                    ..Default::default()
                }),
        )
        .unwrap();

    surface.compute_layout(UiSize::new(240.0, 120.0)).unwrap();
    surface
        .render_extract
        .list
        .commands
        .into_iter()
        .find(|command| command.node_id == UiNodeId::new(2))
        .unwrap_or_else(|| panic!("render extract should include `{component}` command"))
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

fn intrinsic_constraints() -> BoxConstraints {
    BoxConstraints {
        width: intrinsic_axis_constraint(),
        height: intrinsic_axis_constraint(),
    }
}

fn intrinsic_axis_constraint() -> AxisConstraint {
    AxisConstraint {
        min: 0.0,
        max: -1.0,
        preferred: 0.0,
        priority: 100,
        weight: 1.0,
        stretch_mode: StretchMode::Fixed,
    }
}
