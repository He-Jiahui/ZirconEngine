use super::{EditorRouteIntent, EditorRouteIntentMap};
use crate::ui::retained_host::menu_pointer::HostMenuPointerRouteIntent;
use zircon_runtime_interface::ui::{
    dispatch::UiPointerDispatchResult,
    event_ui::{UiNodeId, UiRouteId},
    layout::UiPoint,
    surface::{UiPointerActivationPhase, UiPointerEventKind, UiPointerRoute, UiPointerRoutingPath},
};

fn menu_intent(action_id: &str) -> EditorRouteIntent {
    EditorRouteIntent::Menu(HostMenuPointerRouteIntent::MenuItem {
        menu_index: 0,
        item_index: 1,
        item_path: vec![2, 1],
        action_id: action_id.to_owned(),
    })
}

fn pointer_dispatch(target: Option<UiNodeId>) -> UiPointerDispatchResult {
    UiPointerDispatchResult::new(UiPointerRoute {
        kind: UiPointerEventKind::Move,
        button: None,
        modifiers: Default::default(),
        activation_phase: UiPointerActivationPhase::Hover,
        point: UiPoint::default(),
        scroll_delta: 0.0,
        target,
        hit_path: Default::default(),
        routing_path: UiPointerRoutingPath::default(),
        stacked: Vec::new(),
        entered: Vec::new(),
        left: Vec::new(),
        captured: None,
        pressed: None,
        click_target: None,
        release_inside_pressed: false,
        focused: None,
        fallback_to_root: false,
        root_targets: Vec::new(),
    })
}

#[test]
fn repeated_route_handle_lookup_borrows_one_retained_payload() {
    let node = UiNodeId::new(41);
    let route = UiRouteId::new(73);
    let mut map = EditorRouteIntentMap::default();
    map.bind_node(node, route, menu_intent("menu.action"));
    let retained = map.intent_for_node(node).expect("bound route");

    for _ in 0..1_000 {
        let handle = map.handle_for_node(node).expect("route handle");
        assert_eq!(handle.node_id, node);
        assert_eq!(handle.route_id, route);
        assert!(std::ptr::eq(
            map.resolve_handle(handle).expect("retained payload"),
            retained,
        ));
    }
}

#[test]
fn pointer_handle_prefers_handler_then_falls_back_to_target() {
    let handled = UiNodeId::new(41);
    let target = UiNodeId::new(42);
    let mut map = EditorRouteIntentMap::default();
    map.bind_node(handled, UiRouteId::new(71), menu_intent("handled"));
    map.bind_node(target, UiRouteId::new(72), menu_intent("target"));
    let mut dispatch = pointer_dispatch(Some(target));

    dispatch.handled_by = Some(handled);
    assert_eq!(
        map.handle_for_pointer_dispatch(&dispatch).unwrap().node_id,
        handled
    );
    dispatch.handled_by = Some(UiNodeId::new(43));
    assert!(map.handle_for_pointer_dispatch(&dispatch).is_none());
    assert!(map.intent_for_pointer_dispatch(&dispatch).is_none());
    dispatch.handled_by = None;
    assert_eq!(
        map.handle_for_pointer_dispatch(&dispatch).unwrap().node_id,
        target
    );
    dispatch.route.target = None;
    assert!(map.handle_for_pointer_dispatch(&dispatch).is_none());
}

#[test]
fn stale_handle_cannot_resolve_after_rebind_or_map_replacement() {
    let node = UiNodeId::new(41);
    let route = UiRouteId::new(73);
    let mut map = EditorRouteIntentMap::default();
    map.bind_node(node, route, menu_intent("first"));
    let first = map.handle_for_node(node).unwrap();

    map.bind_node(node, route, menu_intent("second"));
    assert!(map.resolve_handle(first).is_none());
    let second = map.handle_for_node(node).unwrap();
    assert!(map.resolve_handle(second).is_some());

    let cloned_map = map.clone();
    assert!(cloned_map.resolve_handle(second).is_none());

    let mut replacement = EditorRouteIntentMap::default();
    replacement.bind_node(node, route, menu_intent("third"));
    assert!(replacement.resolve_handle(second).is_none());
    assert!(replacement.resolve_handle(first).is_none());
    assert!(replacement
        .resolve_handle(replacement.handle_for_node(node).unwrap())
        .is_some());
}

#[test]
fn route_intent_map_uses_one_hash_index_for_hot_pointer_lookup() {
    let source = include_str!("../map.rs");
    let implementation = source.split("#[cfg(test)]").next().expect("implementation");
    assert!(implementation.contains("HashMap<UiNodeId, EditorRouteBinding>"));
    assert_eq!(implementation.matches("HashMap<").count(), 1);
    assert!(!implementation.contains("BTreeMap"));
}
