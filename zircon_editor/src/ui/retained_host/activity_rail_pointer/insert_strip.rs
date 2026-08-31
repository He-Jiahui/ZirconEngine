use zircon_runtime::ui::{dispatch::UiPointerDispatcher, surface::UiSurface};
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath},
    layout::UiFrame,
    tree::{UiInputPolicy, UiTreeNode},
};

use super::base_state::base_state;
use super::constants::{BUTTON_EXTENT, BUTTON_GAP, STRIP_X_INSET, STRIP_Y_INSET};
use super::host_activity_rail_pointer_item::HostActivityRailPointerItem;
use super::host_activity_rail_pointer_route::HostActivityRailPointerRoute;
use super::host_activity_rail_pointer_side::HostActivityRailPointerSide;
use super::register_handled_pointer_node::register_handled_pointer_node;
use super::strip_button_node_id::{strip_button_node_id, strip_button_route_id, strip_route_id};
use crate::ui::retained_host::route_intent::{EditorRouteIntent, EditorRouteIntentMap};

pub(super) fn insert_strip(
    surface: &mut UiSurface,
    dispatcher: &mut UiPointerDispatcher,
    route_intents: &mut EditorRouteIntentMap,
    root_node_id: UiNodeId,
    strip_node_id: UiNodeId,
    path: &str,
    frame: UiFrame,
    tabs: &[HostActivityRailPointerItem],
    side: HostActivityRailPointerSide,
) {
    if frame.width <= 0.0 || frame.height <= 0.0 {
        return;
    }

    surface
        .tree
        .insert_child(
            root_node_id,
            UiTreeNode::new(strip_node_id, UiNodePath::new(path))
                .with_frame(frame)
                .with_z_index(10)
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(base_state(true)),
        )
        .expect("activity rail root must exist");
    register_handled_pointer_node(dispatcher, strip_node_id);
    route_intents.bind_node(
        strip_node_id,
        strip_route_id(side),
        EditorRouteIntent::ActivityRail(HostActivityRailPointerRoute::Strip(side)),
    );

    for (item_index, _) in tabs.iter().enumerate() {
        let node_id = strip_button_node_id(side, item_index);
        surface
            .tree
            .insert_child(
                strip_node_id,
                UiTreeNode::new(
                    node_id,
                    UiNodePath::new(format!("{path}/button_{item_index}")),
                )
                .with_frame(UiFrame::new(
                    frame.x + STRIP_X_INSET,
                    frame.y + STRIP_Y_INSET + item_index as f32 * (BUTTON_EXTENT + BUTTON_GAP),
                    BUTTON_EXTENT,
                    BUTTON_EXTENT,
                ))
                .with_z_index(20 + item_index as i32)
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(base_state(true)),
            )
            .expect("activity rail strip must exist");
        register_handled_pointer_node(dispatcher, node_id);
        route_intents.bind_node(
            node_id,
            strip_button_route_id(side, item_index),
            EditorRouteIntent::ActivityRail(HostActivityRailPointerRoute::Button {
                side,
                item_index,
            }),
        );
    }
}
