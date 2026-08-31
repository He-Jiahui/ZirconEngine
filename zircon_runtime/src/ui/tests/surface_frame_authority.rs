use crate::ui::{
    dispatch::UiPointerDispatcher,
    surface::{hit_test_surface_frame, UiSurface},
};
use zircon_runtime_interface::ui::{
    dispatch::{UiPointerDispatchEffect, UiPointerEvent},
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::{
        AxisConstraint, BoxConstraints, StretchMode, UiAlignment, UiAlignment2D, UiContainerKind,
        UiFrame, UiGridBoxConfig, UiGridSlotPlacement, UiLayoutEngineBackend,
        UiLayoutEngineFallbackReason, UiLayoutEngineFamily, UiLayoutEngineSupport,
        UiLinearBoxConfig, UiLinearSlotSizeRule, UiLinearSlotSizing, UiMargin, UiPoint, UiSize,
        UiSizeBoxConfig, UiSlot, UiSlotKind, UiWrapBoxConfig,
    },
    surface::{UiPointerButton, UiPointerEventKind, UI_RENDER_FRAME_COMMAND_SEGMENT_SIZE},
    tree::{UiInputPolicy, UiTemplateNodeMetadata, UiTreeNode},
};

const ROOT_ID: UiNodeId = UiNodeId::new(1);
const BACK_ID: UiNodeId = UiNodeId::new(2);
const FRONT_ID: UiNodeId = UiNodeId::new(3);

mod arranged_authority;
mod taffy_flex;
mod taffy_wrap_grid;
mod zircon_fallback;

fn overlapping_button_surface() -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("surface.frame.authority"));
    surface.tree.insert_root(
        UiTreeNode::new(ROOT_ID, UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 180.0, 120.0))
            .with_input_policy(UiInputPolicy::Ignore)
            .with_state_flags(root_state()),
    );
    surface
        .tree
        .insert_child(
            ROOT_ID,
            button_node(
                BACK_ID,
                "root/back",
                "back.button",
                UiFrame::new(16.0, 16.0, 96.0, 56.0),
                0,
            ),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            ROOT_ID,
            button_node(
                FRONT_ID,
                "root/front",
                "front.button",
                UiFrame::new(32.0, 24.0, 96.0, 56.0),
                10,
            ),
        )
        .unwrap();
    surface.rebuild();
    surface
}

fn taffy_flex_button_surface() -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("surface.frame.authority.taffy"));
    surface.tree.insert_root(
        UiTreeNode::new(ROOT_ID, UiNodePath::new("root"))
            .with_container(UiContainerKind::HorizontalBox(UiLinearBoxConfig {
                gap: 4.0,
            }))
            .with_input_policy(UiInputPolicy::Ignore)
            .with_state_flags(root_state()),
    );
    surface
        .tree
        .insert_child(
            ROOT_ID,
            layout_button_node(BACK_ID, "root/back", "back.button", 40.0, 0),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            ROOT_ID,
            layout_button_node(FRONT_ID, "root/front", "front.button", 80.0, 10),
        )
        .unwrap();
    surface
}

fn taffy_flex_linear_slot_sizing_button_surface() -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new(
        "surface.frame.authority.taffy.flex.slot_sizing",
    ));
    surface.tree.insert_root(
        UiTreeNode::new(ROOT_ID, UiNodePath::new("root"))
            .with_container(UiContainerKind::HorizontalBox(UiLinearBoxConfig {
                gap: 0.0,
            }))
            .with_input_policy(UiInputPolicy::Ignore)
            .with_state_flags(root_state()),
    );
    surface
        .tree
        .insert_child(
            ROOT_ID,
            button_node(
                BACK_ID,
                "root/back",
                "back.button",
                UiFrame::new(0.0, 0.0, 0.0, 0.0),
                0,
            ),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            ROOT_ID,
            button_node(
                FRONT_ID,
                "root/front",
                "front.button",
                UiFrame::new(0.0, 0.0, 0.0, 0.0),
                10,
            ),
        )
        .unwrap();
    surface.tree.push_layout_slot(
        UiSlot::new(ROOT_ID, BACK_ID, UiSlotKind::Linear).with_linear_sizing(
            UiLinearSlotSizing::new(UiLinearSlotSizeRule::Stretch).with_value(2.0),
        ),
    );
    surface.tree.push_layout_slot(
        UiSlot::new(ROOT_ID, FRONT_ID, UiSlotKind::Linear).with_linear_sizing(
            UiLinearSlotSizing::new(UiLinearSlotSizeRule::Stretch).with_value(1.0),
        ),
    );
    surface
}

fn taffy_vertical_flex_linear_slot_sizing_button_surface() -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new(
        "surface.frame.authority.taffy.vertical_flex.slot_sizing",
    ));
    surface.tree.insert_root(
        UiTreeNode::new(ROOT_ID, UiNodePath::new("root"))
            .with_container(UiContainerKind::VerticalBox(UiLinearBoxConfig { gap: 0.0 }))
            .with_input_policy(UiInputPolicy::Ignore)
            .with_state_flags(root_state()),
    );
    surface
        .tree
        .insert_child(
            ROOT_ID,
            button_node(
                BACK_ID,
                "root/back",
                "back.button",
                UiFrame::new(0.0, 0.0, 0.0, 0.0),
                0,
            ),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            ROOT_ID,
            button_node(
                FRONT_ID,
                "root/front",
                "front.button",
                UiFrame::new(0.0, 0.0, 0.0, 0.0),
                10,
            ),
        )
        .unwrap();
    surface.tree.push_layout_slot(
        UiSlot::new(ROOT_ID, BACK_ID, UiSlotKind::Linear).with_linear_sizing(
            UiLinearSlotSizing::new(UiLinearSlotSizeRule::Stretch).with_value(2.0),
        ),
    );
    surface.tree.push_layout_slot(
        UiSlot::new(ROOT_ID, FRONT_ID, UiSlotKind::Linear).with_linear_sizing(
            UiLinearSlotSizing::new(UiLinearSlotSizeRule::Stretch).with_value(1.0),
        ),
    );
    surface
}

fn taffy_flex_slot_policy_fallback_button_surface() -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new(
        "surface.frame.authority.taffy.flex.slot_policy_fallback",
    ));
    surface.tree.insert_root(
        UiTreeNode::new(ROOT_ID, UiNodePath::new("root"))
            .with_container(UiContainerKind::HorizontalBox(UiLinearBoxConfig {
                gap: 0.0,
            }))
            .with_input_policy(UiInputPolicy::Ignore)
            .with_state_flags(root_state()),
    );
    surface
        .tree
        .insert_child(
            ROOT_ID,
            layout_button_node_with_size(FRONT_ID, "root/front", "front.button", 40.0, 16.0, 10),
        )
        .unwrap();
    surface.tree.push_layout_slot(
        UiSlot::new(ROOT_ID, FRONT_ID, UiSlotKind::Linear)
            .with_padding(UiMargin::new(10.0, 5.0, 10.0, 5.0))
            .with_alignment(UiAlignment2D::new(UiAlignment::Center, UiAlignment::End)),
    );
    surface
}

fn taffy_wrap_button_surface() -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("surface.frame.authority.taffy.wrap"));
    surface.tree.insert_root(
        UiTreeNode::new(ROOT_ID, UiNodePath::new("root"))
            .with_container(UiContainerKind::WrapBox(UiWrapBoxConfig {
                horizontal_gap: 4.0,
                vertical_gap: 6.0,
                item_min_width: 1.0,
            }))
            .with_input_policy(UiInputPolicy::Ignore)
            .with_state_flags(root_state()),
    );
    surface
        .tree
        .insert_child(
            ROOT_ID,
            layout_button_node_with_size(BACK_ID, "root/back", "back.button", 40.0, 16.0, 0),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            ROOT_ID,
            layout_button_node_with_size(FRONT_ID, "root/front", "front.button", 50.0, 16.0, 10),
        )
        .unwrap();
    surface
}

fn taffy_grid_slot_button_surface() -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("surface.frame.authority.taffy.grid"));
    surface.tree.insert_root(
        UiTreeNode::new(ROOT_ID, UiNodePath::new("root"))
            .with_container(UiContainerKind::GridBox(UiGridBoxConfig {
                columns: 2,
                rows: 2,
                column_gap: 4.0,
                row_gap: 6.0,
            }))
            .with_input_policy(UiInputPolicy::Ignore)
            .with_state_flags(root_state()),
    );
    surface
        .tree
        .insert_child(
            ROOT_ID,
            layout_button_node_with_size(FRONT_ID, "root/front", "front.button", 40.0, 16.0, 10),
        )
        .unwrap();
    surface.tree.push_layout_slot(
        UiSlot::new(ROOT_ID, FRONT_ID, UiSlotKind::Grid)
            .with_grid_placement(UiGridSlotPlacement::new(1, 1))
            .with_padding(UiMargin::new(2.0, 3.0, 4.0, 5.0))
            .with_alignment(UiAlignment2D::new(UiAlignment::Center, UiAlignment::End)),
    );
    surface
}

fn zircon_size_box_button_surface() -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("surface.frame.authority.size_box"));
    surface.tree.insert_root(
        UiTreeNode::new(ROOT_ID, UiNodePath::new("root"))
            .with_container(UiContainerKind::SizeBox(UiSizeBoxConfig {
                aspect_ratio: 2.0,
            }))
            .with_input_policy(UiInputPolicy::Ignore)
            .with_state_flags(root_state()),
    );
    surface
        .tree
        .insert_child(
            ROOT_ID,
            layout_button_node_with_size(FRONT_ID, "root/front", "front.button", 40.0, 16.0, 10),
        )
        .unwrap();
    surface.tree.push_layout_slot(
        UiSlot::new(ROOT_ID, FRONT_ID, UiSlotKind::Container)
            .with_padding(UiMargin::new(10.0, 5.0, 10.0, 5.0))
            .with_alignment(UiAlignment2D::new(UiAlignment::Center, UiAlignment::End)),
    );
    surface
}

fn button_node(
    node_id: UiNodeId,
    node_path: &str,
    control_id: &str,
    frame: UiFrame,
    z_index: i32,
) -> UiTreeNode {
    UiTreeNode::new(node_id, UiNodePath::new(node_path))
        .with_frame(frame)
        .with_z_index(z_index)
        .with_input_policy(UiInputPolicy::Receive)
        .with_state_flags(pointer_state())
        .with_template_metadata(UiTemplateNodeMetadata {
            component: "MaterialButton".to_string(),
            control_id: Some(control_id.to_string()),
            attributes: toml::from_str(
                r##"
text = "Run"
opacity = 1.0

[background]
color = "#2f6f5e"
"##,
            )
            .unwrap(),
            ..Default::default()
        })
}

fn layout_button_node(
    node_id: UiNodeId,
    node_path: &'static str,
    control_id: &'static str,
    width: f32,
    z_index: i32,
) -> UiTreeNode {
    button_node(
        node_id,
        node_path,
        control_id,
        UiFrame::new(0.0, 0.0, width, 0.0),
        z_index,
    )
    .with_constraints(BoxConstraints {
        width: fixed_axis(width),
        height: AxisConstraint::default(),
    })
}

fn layout_button_node_with_size(
    node_id: UiNodeId,
    node_path: &'static str,
    control_id: &'static str,
    width: f32,
    height: f32,
    z_index: i32,
) -> UiTreeNode {
    button_node(
        node_id,
        node_path,
        control_id,
        UiFrame::new(0.0, 0.0, width, height),
        z_index,
    )
    .with_constraints(BoxConstraints {
        width: fixed_axis(width),
        height: fixed_axis(height),
    })
}

fn fixed_axis(value: f32) -> AxisConstraint {
    AxisConstraint {
        min: 0.0,
        max: value,
        preferred: value,
        priority: 0,
        weight: 1.0,
        stretch_mode: StretchMode::Fixed,
    }
}

fn root_state() -> UiStateFlags {
    UiStateFlags {
        visible: true,
        enabled: true,
        clickable: false,
        hoverable: false,
        focusable: false,
        pressed: false,
        checked: false,
        dirty: false,
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
