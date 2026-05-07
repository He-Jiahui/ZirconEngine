use crate::ui::{
    dispatch::UiPointerDispatcher,
    surface::{hit_test_surface_frame, UiSurface},
    tree::UiRuntimeTreeAccessExt,
};
use zircon_runtime_interface::ui::{
    dispatch::{UiPointerDispatchEffect, UiPointerEvent},
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::{UiFrame, UiPoint},
    surface::{UiPointerButton, UiPointerEventKind},
    tree::{UiInputPolicy, UiTemplateNodeMetadata, UiTreeNode},
};

const ROOT_ID: UiNodeId = UiNodeId::new(1);
const BACK_ID: UiNodeId = UiNodeId::new(2);
const FRONT_ID: UiNodeId = UiNodeId::new(3);

#[test]
fn surface_frame_render_hit_and_pointer_dispatch_share_arranged_authority() {
    let mut surface = overlapping_button_surface();
    let point = UiPoint::new(48.0, 36.0);
    let frame = surface.surface_frame();

    assert_eq!(frame.tree_id, UiTreeId::new("surface.frame.authority"));
    assert_eq!(frame.arranged_tree.tree_id, frame.tree_id);
    assert_eq!(frame.render_extract.tree_id, frame.tree_id);

    let arranged_front = frame
        .arranged_tree
        .get(FRONT_ID)
        .expect("front control should be arranged");
    let render_front = frame
        .render_extract
        .list
        .commands
        .iter()
        .find(|command| command.node_id == FRONT_ID)
        .expect("front control should be rendered from the arranged tree");
    let hit_front = frame
        .hit_grid
        .entries
        .iter()
        .find(|entry| entry.node_id == FRONT_ID)
        .expect("front control should be entered into the hit grid");

    assert_eq!(render_front.frame, arranged_front.frame);
    assert_eq!(render_front.clip_frame, Some(arranged_front.clip_frame));
    assert_eq!(render_front.z_index, arranged_front.z_index);
    assert_eq!(hit_front.frame, arranged_front.frame);
    assert_eq!(
        hit_front.clip_frame,
        arranged_front
            .frame
            .intersection(arranged_front.clip_frame)
            .expect("front arranged frame should intersect its clip")
    );
    assert_eq!(hit_front.z_index, arranged_front.z_index);
    assert_eq!(hit_front.paint_order, arranged_front.paint_order);
    assert_eq!(hit_front.control_id.as_deref(), Some("front.button"));

    let frame_hit = hit_test_surface_frame(&frame, point);
    assert_eq!(surface.hit_test(point), frame_hit);
    assert_eq!(frame_hit.top_hit, Some(FRONT_ID));
    assert_eq!(frame_hit.stacked, vec![FRONT_ID, BACK_ID]);
    assert_eq!(frame_hit.path.root_to_leaf, vec![ROOT_ID, FRONT_ID]);
    assert_eq!(frame_hit.path.bubble_route, vec![FRONT_ID, ROOT_ID]);

    let mut dispatcher = UiPointerDispatcher::default();
    dispatcher.register(FRONT_ID, UiPointerEventKind::Down, |context| {
        assert_eq!(context.route.hit_path.target, Some(FRONT_ID));
        assert_eq!(context.route.hit_path.bubble_route, vec![FRONT_ID, ROOT_ID]);
        UiPointerDispatchEffect::handled()
    });

    let dispatch = surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Down, point)
                .with_button(UiPointerButton::Primary),
        )
        .expect("pointer dispatch should route through the same surface hit path");

    assert_eq!(dispatch.handled_by, Some(FRONT_ID));
    assert_eq!(dispatch.route.target, frame_hit.path.target);
    assert_eq!(dispatch.route.hit_path, frame_hit.path);
    assert_eq!(dispatch.route.stacked, frame_hit.stacked);
}

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

fn button_node(
    node_id: UiNodeId,
    node_path: &'static str,
    control_id: &'static str,
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
