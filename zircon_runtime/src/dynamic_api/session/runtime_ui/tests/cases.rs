use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use std::collections::{BTreeSet, HashMap};
use std::time::{Duration, Instant};

use super::super::profile::RuntimeDynamicSessionProfile;
use super::super::registry::RuntimeFrameDemand;
use super::super::state::RuntimeDynamicSession;
use super::input_publication::RuntimeUiInputQueryAdmission;
use super::input_routing::{
    capture_surface_for_event, split_global_node_id, ui_input_timestamp_at, update_capture_surface,
};
use super::{global_node_id, RuntimeUiSurface, RuntimeUiSurfaceSet};
use crate::core::CoreError;
use crate::plugin::RuntimeExtensionRegistry;
use crate::scene::SystemStage;
use crate::ui::dispatch::UiInputManager;
use crate::ui::surface::UiSurface;
use zircon_runtime_interface::ui::binding::UiEventKind;
use zircon_runtime_interface::ui::component::UiComponentEventKind;
use zircon_runtime_interface::ui::dispatch::{
    UiInputEvent, UiInputEventMetadata, UiInputTimestamp, UiKeyboardInputEvent,
    UiKeyboardInputState, UiMouseMotionInputEvent, UiNavigationInputEvent, UiPointerEvent,
    UiPointerInputEvent,
};
use zircon_runtime_interface::ui::event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId};
use zircon_runtime_interface::ui::layout::{UiFrame, UiPoint};
use zircon_runtime_interface::ui::surface::{
    UiNavigationEventKind, UiPointerButton, UiPointerEventKind,
};
use zircon_runtime_interface::ui::template::UiBindingRef;
use zircon_runtime_interface::ui::tree::{
    UiDirtyFlags, UiInputPolicy, UiTemplateNodeMetadata, UiTreeNode,
};

#[test]
fn global_node_ids_keep_same_local_ids_distinct_across_runtime_ui_surfaces() {
    let local = UiNodeId::new(41);
    let first = global_node_id(0, local);
    let second = global_node_id(1, local);

    assert_ne!(first, second);
    assert_eq!(split_global_node_id(first), Some((0, local)));
    assert_eq!(split_global_node_id(second), Some((1, local)));
}

#[test]
fn stable_project_ui_submission_reuses_the_same_allocation() {
    let mut surfaces = RuntimeUiSurfaceSet {
        surfaces: vec![RuntimeUiSurface {
            surface: UiSurface::new(zircon_runtime_interface::ui::event_ui::UiTreeId::new(
                "test-runtime-ui",
            )),
            input: UiInputManager::default(),
        }],
        ..RuntimeUiSurfaceSet::default()
    };
    let viewport = crate::core::math::UVec2::new(640, 360);

    let first = surfaces.render_submission(viewport).unwrap().unwrap();
    let second = surfaces.render_submission(viewport).unwrap().unwrap();

    assert!(Arc::ptr_eq(&first, &second));
}

#[test]
fn local_surface_change_reuses_unchanged_segment_allocation() {
    let changed_node = UiNodeId::new(2);
    let mut surfaces = RuntimeUiSurfaceSet {
        surfaces: vec![
            test_runtime_surface("test-runtime-ui:first", UiNodeId::new(1)),
            test_runtime_surface("test-runtime-ui:second", changed_node),
        ],
        ..RuntimeUiSurfaceSet::default()
    };
    let viewport = crate::core::math::UVec2::new(640, 360);

    let first = surfaces.render_submission(viewport).unwrap().unwrap();
    surfaces.surfaces[1]
        .surface
        .mark_node_dirty(
            changed_node,
            UiDirtyFlags {
                render: true,
                ..UiDirtyFlags::default()
            },
        )
        .unwrap();
    let second = surfaces.render_submission(viewport).unwrap().unwrap();

    assert!(!Arc::ptr_eq(&first, &second));
    assert!(Arc::ptr_eq(
        first.segments()[0].extract(),
        second.segments()[0].extract()
    ));
    assert!(!Arc::ptr_eq(
        first.segments()[1].extract(),
        second.segments()[1].extract()
    ));
}

fn test_runtime_surface(tree_id: &str, node_id: UiNodeId) -> RuntimeUiSurface {
    let mut surface = UiSurface::new(UiTreeId::new(tree_id));
    surface.tree.insert_root(
        UiTreeNode::new(node_id, UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 100.0, 40.0)),
    );
    RuntimeUiSurface {
        surface,
        input: UiInputManager::default(),
    }
}

#[test]
fn pointer_capture_routes_follow_up_events_back_to_the_owning_surface() {
    let mut captures = HashMap::new();
    update_capture_surface(&mut captures, Some(7), 0, UiPointerEventKind::Down, true);

    assert_eq!(
        capture_surface_for_event(&captures, Some(7), UiPointerEventKind::Move),
        Some(0)
    );
    assert_eq!(
        capture_surface_for_event(&captures, Some(7), UiPointerEventKind::Up),
        Some(0)
    );

    update_capture_surface(&mut captures, Some(7), 0, UiPointerEventKind::Up, false);
    assert_eq!(
        capture_surface_for_event(&captures, Some(7), UiPointerEventKind::Move),
        None
    );
}

#[test]
fn raw_mouse_motion_does_not_rebuild_or_dispatch_surfaces() {
    let mut surfaces = RuntimeUiSurfaceSet {
        surfaces: vec![
            test_runtime_surface("test-runtime-ui:bottom", UiNodeId::new(1)),
            test_runtime_surface("test-runtime-ui:top", UiNodeId::new(2)),
        ],
        ..RuntimeUiSurfaceSet::default()
    };
    let dirty_before = surfaces
        .surfaces
        .iter()
        .map(|surface| surface.surface.dirty_flags())
        .collect::<Vec<_>>();
    assert!(dirty_before.iter().any(|dirty| {
        dirty.layout || dirty.hit_test || dirty.render || dirty.style || dirty.text
    }));

    let handled = surfaces
        .dispatch_input(
            crate::core::math::UVec2::new(640, 360),
            UiInputEvent::MouseMotion(UiMouseMotionInputEvent {
                metadata: UiInputEventMetadata::default(),
                delta_x: 2.0,
                delta_y: -1.0,
            }),
        )
        .unwrap();

    assert!(!handled);
    assert_eq!(
        surfaces
            .surfaces
            .iter()
            .map(|surface| surface.surface.dirty_flags())
            .collect::<Vec<_>>(),
        dirty_before
    );
}

#[test]
fn resized_pointer_directory_skips_non_candidate_dirty_surfaces() {
    let published_viewport = crate::core::math::UVec2::new(640, 360);
    let resized_viewport = crate::core::math::UVec2::new(1280, 720);
    let bottom_node = UiNodeId::new(5);
    let top_node = UiNodeId::new(6);
    let mut surfaces = RuntimeUiSurfaceSet {
        surfaces: vec![
            test_pointer_runtime_surface(
                "test-runtime-ui:pointer-bottom",
                bottom_node,
                UiFrame::new(0.0, 0.0, 100.0, 40.0),
            ),
            test_pointer_runtime_surface(
                "test-runtime-ui:pointer-top",
                top_node,
                UiFrame::new(300.0, 100.0, 100.0, 40.0),
            ),
        ],
        ..RuntimeUiSurfaceSet::default()
    };
    surfaces.render_submission(published_viewport).unwrap();
    let query = surfaces.input_publication.query(
        resized_viewport,
        zircon_runtime_interface::ui::layout::UiPoint::new(24.0, 24.0),
        zircon_runtime_interface::ui::layout::UiPoint::new(20.0, 20.0),
    );
    let RuntimeUiInputQueryAdmission::Published(query) = query else {
        panic!("published pointer query expected");
    };
    assert_eq!(query.candidate_count(), 1);
    assert_eq!(
        surfaces.input_publication.candidate_surface(query, 0),
        Some(0)
    );

    surfaces.surfaces[1]
        .surface
        .mark_node_dirty(
            top_node,
            UiDirtyFlags {
                hit_test: true,
                render: true,
                ..UiDirtyFlags::default()
            },
        )
        .unwrap();
    let top_dirty_before = surfaces.surfaces[1].surface.dirty_flags();

    surfaces
        .dispatch_pointer(
            resized_viewport,
            UiPointerEventKind::Move,
            zircon_runtime_interface::ui::layout::UiPoint::new(24.0, 24.0),
            None,
            None,
            zircon_runtime_interface::ui::dispatch::UiPointerSource::Mouse,
            0.0,
        )
        .unwrap();

    assert_eq!(surfaces.surfaces[1].surface.dirty_flags(), top_dirty_before);
}

#[test]
fn invalid_pointer_input_does_not_probe_dirty_surfaces() {
    let viewport = crate::core::math::UVec2::new(640, 360);
    let node = UiNodeId::new(7);
    let mut surfaces = RuntimeUiSurfaceSet {
        surfaces: vec![test_pointer_runtime_surface(
            "test-runtime-ui:invalid-pointer",
            node,
            UiFrame::new(0.0, 0.0, 100.0, 40.0),
        )],
        ..RuntimeUiSurfaceSet::default()
    };
    surfaces.render_submission(viewport).unwrap();
    surfaces.surfaces[0]
        .surface
        .mark_node_dirty(
            node,
            UiDirtyFlags {
                hit_test: true,
                render: true,
                ..UiDirtyFlags::default()
            },
        )
        .unwrap();
    let dirty_before = surfaces.surfaces[0].surface.dirty_flags();

    assert!(!surfaces
        .dispatch_pointer(
            viewport,
            UiPointerEventKind::Move,
            UiPoint::new(f32::NAN, 12.0),
            None,
            None,
            zircon_runtime_interface::ui::dispatch::UiPointerSource::Mouse,
            0.0,
        )
        .unwrap());
    assert_eq!(surfaces.surfaces[0].surface.dirty_flags(), dirty_before);
}

#[test]
fn focused_keyboard_input_dispatches_only_to_the_published_owner() {
    let mut bottom =
        test_focusable_runtime_surface("test-runtime-ui:focus-bottom", UiNodeId::new(11));
    bottom.surface.focus_node(UiNodeId::new(11)).unwrap();
    let top = test_runtime_surface("test-runtime-ui:focus-top", UiNodeId::new(22));
    let mut surfaces = RuntimeUiSurfaceSet {
        surfaces: vec![bottom, top],
        ..RuntimeUiSurfaceSet::default()
    };
    surfaces.refresh_input_owners_from_publication();
    let top_dirty_before = surfaces.surfaces[1].surface.dirty_flags();

    let handled = surfaces
        .dispatch_input(crate::core::math::UVec2::new(640, 360), keyboard_input())
        .unwrap();

    assert!(!handled);
    assert_eq!(surfaces.focused_surface, Some(0));
    assert_eq!(surfaces.surfaces[1].surface.dirty_flags(), top_dirty_before);
}

#[test]
fn focused_input_without_a_published_owner_does_not_probe_surfaces() {
    let mut surfaces = RuntimeUiSurfaceSet {
        surfaces: vec![
            test_runtime_surface("test-runtime-ui:no-focus-bottom", UiNodeId::new(31)),
            test_runtime_surface("test-runtime-ui:no-focus-top", UiNodeId::new(32)),
        ],
        ..RuntimeUiSurfaceSet::default()
    };
    let dirty_before = surfaces
        .surfaces
        .iter()
        .map(|surface| surface.surface.dirty_flags())
        .collect::<Vec<_>>();

    assert!(!surfaces
        .dispatch_input(crate::core::math::UVec2::new(640, 360), keyboard_input(),)
        .unwrap());
    assert_eq!(
        surfaces
            .surfaces
            .iter()
            .map(|surface| surface.surface.dirty_flags())
            .collect::<Vec<_>>(),
        dirty_before
    );
}

#[test]
fn a_focus_transition_selects_the_actual_surface_over_stack_order() {
    let mut bottom = test_focusable_runtime_surface(
        "test-runtime-ui:focus-transition-bottom",
        UiNodeId::new(41),
    );
    let mut top =
        test_focusable_runtime_surface("test-runtime-ui:focus-transition-top", UiNodeId::new(42));
    top.surface.focus_node(UiNodeId::new(42)).unwrap();
    let mut surfaces = RuntimeUiSurfaceSet {
        surfaces: vec![bottom, top],
        ..RuntimeUiSurfaceSet::default()
    };
    surfaces.refresh_input_owners_from_publication();
    assert_eq!(surfaces.focused_surface, Some(1));

    surfaces.surfaces[0]
        .surface
        .focus_node(UiNodeId::new(41))
        .unwrap();
    surfaces.update_focused_surface_after_dispatch(0, None, Some(UiNodeId::new(41)));

    assert_eq!(surfaces.focused_surface, Some(0));
    assert_eq!(
        surfaces.focused_surfaces,
        BTreeSet::from([0_usize, 1_usize])
    );
    surfaces.refresh_input_owners_from_publication();
    assert_eq!(surfaces.focused_surface, Some(0));
}

#[test]
fn navigation_dispatches_only_to_the_published_eligible_surface() {
    let viewport = crate::core::math::UVec2::new(640, 360);
    let bottom_node = UiNodeId::new(51);
    let top_node = UiNodeId::new(52);
    let mut surfaces = RuntimeUiSurfaceSet {
        surfaces: vec![
            test_focusable_runtime_surface("test-runtime-ui:navigation-bottom", bottom_node),
            test_runtime_surface("test-runtime-ui:navigation-top", top_node),
        ],
        ..RuntimeUiSurfaceSet::default()
    };
    surfaces.render_submission(viewport).unwrap();
    assert_eq!(surfaces.navigation_surface, Some(0));

    surfaces.surfaces[1]
        .surface
        .mark_node_dirty(
            top_node,
            UiDirtyFlags {
                render: true,
                ..UiDirtyFlags::default()
            },
        )
        .unwrap();
    let top_dirty_before = surfaces.surfaces[1].surface.dirty_flags();

    let handled = surfaces
        .dispatch_input(
            viewport,
            UiInputEvent::Navigation(UiNavigationInputEvent {
                metadata: UiInputEventMetadata::default(),
                kind: UiNavigationEventKind::Next,
            }),
        )
        .unwrap();

    assert!(handled);
    assert_eq!(surfaces.focused_surface, Some(0));
    assert_eq!(surfaces.navigation_surface, Some(0));
    assert_eq!(
        surfaces.surfaces[0].surface.focus.focused,
        Some(bottom_node)
    );
    assert_eq!(surfaces.surfaces[1].surface.dirty_flags(), top_dirty_before);
}

fn test_focusable_runtime_surface(tree_id: &str, node_id: UiNodeId) -> RuntimeUiSurface {
    let mut runtime_surface = test_runtime_surface(tree_id, node_id);
    runtime_surface
        .surface
        .tree
        .node_mut(node_id)
        .unwrap()
        .state_flags = UiStateFlags {
        visible: true,
        enabled: true,
        focusable: true,
        ..UiStateFlags::default()
    };
    runtime_surface
}

fn test_pointer_runtime_surface(
    tree_id: &str,
    node_id: UiNodeId,
    frame: UiFrame,
) -> RuntimeUiSurface {
    let mut surface = UiSurface::new(UiTreeId::new(tree_id));
    let mut node = UiTreeNode::new(node_id, UiNodePath::new("root"))
        .with_frame(frame)
        .with_input_policy(UiInputPolicy::Receive);
    node.state_flags.clickable = true;
    node.state_flags.hoverable = true;
    surface.tree.insert_root(node);
    RuntimeUiSurface {
        surface,
        input: UiInputManager::default(),
    }
}

fn keyboard_input() -> UiInputEvent {
    UiInputEvent::Keyboard(UiKeyboardInputEvent {
        metadata: UiInputEventMetadata::default(),
        state: UiKeyboardInputState::Pressed,
        key_code: 65,
        scan_code: Some(30),
        physical_key: "KeyA".to_string(),
        logical_key: "A".to_string(),
        text: Some("a".to_string()),
    })
}

#[test]
fn runtime_ui_clock_advances_from_monotonic_time_while_game_delta_is_paused() {
    let origin = Instant::now();
    let paused_game_delta = Duration::ZERO;

    let before_pause = ui_input_timestamp_at(origin, origin);
    let after_pause = ui_input_timestamp_at(origin, origin + Duration::from_millis(120));

    assert_eq!(paused_game_delta, Duration::ZERO);
    assert_eq!(before_pause, UiInputTimestamp::from_micros(1));
    assert_eq!(
        after_pause,
        UiInputTimestamp::from_micros(120_001),
        "UI input time must follow monotonic real time without consuming game delta"
    );
}

#[test]
fn runtime_surface_set_stamps_input_and_ticks_each_surface_timer_at_one_time() {
    let menu_node = UiNodeId::new(3);
    let double_click_node = UiNodeId::new(4);
    let mut surfaces = RuntimeUiSurfaceSet {
        surfaces: vec![
            test_runtime_surface("test-runtime-ui:tooltip", UiNodeId::new(1)),
            test_typeahead_runtime_surface("test-runtime-ui:typeahead", menu_node),
            test_pointer_runtime_surface(
                "test-runtime-ui:double-click",
                double_click_node,
                UiFrame::new(0.0, 0.0, 100.0, 40.0),
            ),
        ],
        focused_surfaces: BTreeSet::from([1]),
        focused_surface: Some(1),
        ..RuntimeUiSurfaceSet::default()
    };
    surfaces.input_clock.origin = Instant::now() - Duration::from_secs(1);
    let viewport = crate::core::math::UVec2::new(640, 360);
    let timer_start = surfaces.next_input_metadata().timestamp;

    {
        let runtime_surface = &mut surfaces.surfaces[0];
        runtime_surface.input.arm_tooltip_candidate(
            &mut runtime_surface.surface,
            timer_start,
            UiNodeId::new(1),
            "status.hint",
            100,
        );
    }

    assert!(surfaces
        .dispatch_input(viewport, keyboard_input())
        .expect("menu keyboard text should route to the focused surface"));
    let typeahead_deadline = surfaces.surfaces[1]
        .input
        .timers()
        .typeahead_expiration(menu_node)
        .expect("real input should arm the typeahead deadline");
    assert!(
        typeahead_deadline.monotonic_micros >= timer_start.monotonic_micros.saturating_add(25_000),
        "a caller's default timestamp must be replaced with real UI time before arming timers"
    );

    surfaces.surfaces[2].surface.focus.pressed = Some(double_click_node);
    let pointer_release = UiInputEvent::Pointer(UiPointerInputEvent {
        metadata: UiInputEventMetadata::default(),
        event: UiPointerEvent::new(UiPointerEventKind::Up, UiPoint::new(1.0, 1.0))
            .with_button(UiPointerButton::Primary),
        precise_scroll: None,
    });
    let _ = surfaces
        .dispatch_input(viewport, pointer_release)
        .expect("pointer release should dispatch on all candidate surfaces");
    let double_click_deadline = surfaces.surfaces[2]
        .input
        .timers()
        .double_click_expiration()
        .expect("primary release should arm double-click classification");
    assert!(
        double_click_deadline.monotonic_micros
            >= timer_start.monotonic_micros.saturating_add(500_000),
        "double-click classification must use the same real UI clock as other surface timers"
    );

    let sample = surfaces.next_input_metadata().timestamp;
    let tooltip_deadline = surfaces.surfaces[0]
        .input
        .timers()
        .tooltip_expiration(UiNodeId::new(1))
        .expect("tooltip candidate should retain its deadline");
    let earliest_deadline = typeahead_deadline.min(tooltip_deadline);
    assert_eq!(
        surfaces.next_input_timer_delay_at(sample),
        Some(Duration::from_micros(
            earliest_deadline
                .monotonic_micros
                .saturating_sub(sample.monotonic_micros)
        )),
        "the runtime should request the earliest visible UI timer across surfaces"
    );

    let expired_at = UiInputTimestamp::from_micros(u64::MAX);
    surfaces
        .tick_input_timers_at(expired_at)
        .expect("all runtime UI surface timers should tick");

    for runtime_surface in &surfaces.surfaces {
        assert_eq!(runtime_surface.input.timers().last_tick(), Some(expired_at));
    }
    assert!(surfaces.surfaces[0]
        .surface
        .input
        .tooltip
        .as_ref()
        .is_some_and(|tooltip| tooltip.visible));
    assert_eq!(
        surfaces.surfaces[1]
            .input
            .timers()
            .typeahead_expiration(menu_node),
        None
    );
    assert_eq!(
        surfaces.surfaces[2]
            .input
            .timers()
            .double_click_expiration(),
        None
    );
}

fn test_typeahead_runtime_surface(tree_id: &str, node_id: UiNodeId) -> RuntimeUiSurface {
    let mut surface = UiSurface::new(UiTreeId::new(tree_id));
    let mut node = UiTreeNode::new(node_id, UiNodePath::new("menu"))
        .with_frame(UiFrame::new(0.0, 0.0, 100.0, 40.0))
        .with_input_policy(UiInputPolicy::Receive);
    node.template_metadata = Some(UiTemplateNodeMetadata {
        component: "MenuList".to_string(),
        control_id: Some("SceneMenu".to_string()),
        bindings: vec![
            test_component_binding("MenuList/KeyboardText", UiEventKind::Change),
            test_component_binding("MenuList/TypeaheadExpired", UiEventKind::Change),
        ],
        attributes: toml::from_str("component_role = 'menu-list'\ntypeahead_timeout_ms = 25")
            .expect("menu timer attributes should parse"),
        ..UiTemplateNodeMetadata::default()
    });
    surface.tree.insert_root(node);
    surface.rebuild();
    surface
        .focus_node(node_id)
        .expect("menu should accept keyboard focus");
    RuntimeUiSurface {
        surface,
        input: UiInputManager::default(),
    }
}

fn test_component_binding(id: &str, event: UiEventKind) -> UiBindingRef {
    let component_event = id
        .rsplit('/')
        .next()
        .and_then(UiComponentEventKind::from_schema_name);
    UiBindingRef {
        id: id.to_string(),
        event,
        mode: Default::default(),
        component_event,
        route: Some(id.replace('/', ".")),
        action: None,
        targets: Vec::new(),
    }
}

#[test]
fn failed_scene_tick_keeps_ui_timer_pending_for_successful_retry() {
    let mut session = RuntimeDynamicSession::new(RuntimeDynamicSessionProfile::Headless, None)
        .expect("headless dynamic session");
    let tooltip_node = UiNodeId::new(9);
    let mut surface = UiSurface::new(UiTreeId::new("test-runtime-ui:retry-tooltip"));
    surface.tree.insert_root(
        UiTreeNode::new(tooltip_node, UiNodePath::new("tooltip"))
            .with_frame(UiFrame::new(0.0, 0.0, 100.0, 40.0)),
    );
    surface.rebuild();
    let mut input = UiInputManager::default();
    let timer_start = session.runtime_ui.next_input_metadata().timestamp;
    input.arm_tooltip_candidate(&mut surface, timer_start, tooltip_node, "retry.tooltip", 0);
    session.runtime_ui.surfaces = vec![RuntimeUiSurface { surface, input }];
    assert_eq!(session.frame_demand(), RuntimeFrameDemand::Immediate);

    let fail_first_tick = Arc::new(AtomicBool::new(true));
    let fail_for_factory = Arc::clone(&fail_first_tick);
    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry
        .intern_plugin_module("test.runtime-ui-retry")
        .unwrap();
    registry
        .register_runtime_scene_system(
            owner,
            "test.runtime-ui-retry.fail-once",
            SystemStage::PostUpdate,
            move || {
                let fail_first_tick = Arc::clone(&fail_for_factory);
                move |_| {
                    if fail_first_tick.swap(false, Ordering::AcqRel) {
                        Err(CoreError::RuntimeUnavailable)
                    } else {
                        Ok(())
                    }
                }
            },
        )
        .with_order(1)
        .register()
        .expect("register the retry fixture system");
    let plan = registry
        .world_runtime_extension_plan()
        .expect("build retry fixture extension plan");
    session
        .level
        .with_world_mut(|world| plan.apply_to_world(world))
        .expect("apply retry fixture scene system");

    assert!(session.tick_frame().is_err());
    assert_eq!(
        session.runtime_ui.surfaces[0].input.timers().last_tick(),
        None,
        "the failed core scene tick must leave the UI timer due"
    );
    assert_eq!(session.frame_demand(), RuntimeFrameDemand::Immediate);

    session
        .tick_frame()
        .expect("the next successful frame should advance UI timers");
    assert!(session.runtime_ui.surfaces[0]
        .input
        .timers()
        .last_tick()
        .is_some());
    assert!(session.runtime_ui.surfaces[0]
        .surface
        .input
        .tooltip
        .as_ref()
        .is_some_and(|tooltip| tooltip.visible));
    match session.frame_demand() {
        RuntimeFrameDemand::After(delay) => {
            assert!(!delay.is_zero());
            assert!(delay <= Duration::from_millis(16));
        }
        demand => panic!("visible tooltip intro should schedule its next sample, got {demand:?}"),
    }
}
