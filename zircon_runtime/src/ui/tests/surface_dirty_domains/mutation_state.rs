use super::*;

#[test]
fn surface_dirty_route_state_mutations_keep_legacy_dirty_for_bridges() {
    let mut surface = test_surface();

    let visibility = surface
        .mutate_property(UiPropertyMutationRequest::new(
            button_id(),
            "visibility",
            UiValue::Enum("hidden".to_string()),
        ))
        .unwrap();

    assert_eq!(visibility.status, UiPropertyMutationStatus::Accepted);
    assert_eq!(
        surface
            .tree
            .node(button_id())
            .expect("button node should exist")
            .visibility,
        UiVisibility::Hidden
    );
    assert!(
        surface
            .tree
            .node(button_id())
            .expect("button node should exist")
            .state_flags
            .dirty
    );
    assert_eq!(
        surface.dirty_flags(),
        UiDirtyFlags {
            hit_test: true,
            render: true,
            input: true,
            ..Default::default()
        }
    );

    surface.clear_dirty_flags();
    let input_policy = surface
        .mutate_property(UiPropertyMutationRequest::new(
            button_id(),
            "input_policy",
            UiValue::Enum("ignore".to_string()),
        ))
        .unwrap();

    assert_eq!(input_policy.status, UiPropertyMutationStatus::Accepted);
    assert_eq!(
        surface
            .tree
            .node(button_id())
            .expect("button node should exist")
            .input_policy,
        UiInputPolicy::Ignore
    );
    assert!(
        surface
            .tree
            .node(button_id())
            .expect("button node should exist")
            .state_flags
            .dirty
    );
    assert_eq!(
        surface.dirty_flags(),
        UiDirtyFlags {
            hit_test: true,
            render: true,
            input: true,
            ..Default::default()
        }
    );
}

#[test]
fn surface_dirty_layout_marking_keeps_structured_domains_precise() {
    let mut surface = test_surface();

    surface.tree.mark_layout_dirty(button_id()).unwrap();

    assert_eq!(
        surface.dirty_flags(),
        UiDirtyFlags {
            layout: true,
            hit_test: true,
            render: true,
            ..Default::default()
        }
    );
    assert!(
        !surface
            .tree
            .node(button_id())
            .expect("button node should exist")
            .state_flags
            .dirty
    );
    assert!(
        !surface
            .tree
            .node(root_id())
            .expect("root node should exist")
            .state_flags
            .dirty
    );
}
