use super::*;
use crate::core::editing::authoring_world::AuthoringWorldSeed;
use crate::core::editing::engine::{HistoryContextId, HistoryStatus};
use crate::core::editor_event::EditorHierarchyEvent;
use crate::core::editor_message::DocumentId;
use crate::core::play::WorldDomain;
use crate::ui::retained_host::hierarchy_pointer::{current_hierarchy_row_metrics, hierarchy_row_y};
use zircon_runtime_interface::ui::layout::UiPoint;

struct IdentityFixture {
    harness: ChildWindowHostHarness,
    source: u64,
    target: u64,
    press: UiPoint,
    release: UiPoint,
}

impl IdentityFixture {
    fn new(label: &str) -> Self {
        let harness = ChildWindowHostHarness::new(label);
        harness.activate_workbench_page();
        harness.activate_drawer_tab(ActivityDrawerSlot::LeftTop, "editor.hierarchy#1");
        let (source, target) = {
            let host = harness.host.borrow();
            let entries = &host.hierarchy_scene_entries;
            assert!(entries.len() >= 2);
            (entries[0].entity, entries[1].entity)
        };
        {
            let mut host = harness.host.borrow_mut();
            host.runtime
                .shell()
                .lock()
                .state
                .apply_intent(crate::core::editing::intent::EditorIntent::SelectNode(
                    source,
                ))
                .unwrap();
            host.sync_active_hierarchy_world();
            host.refresh_ui();
            host.recompute_if_dirty();
        }
        let presentation = harness.root_ui.get_host_presentation();
        let dock = &presentation.host_scene_data.left_dock;
        assert_eq!(dock.pane.kind.as_str(), "Hierarchy");
        let origin_x = dock.region_frame.x
            + if dock.rail_before_panel {
                dock.rail_width_px
            } else {
                0.0
            }
            + dock.content_frame.x;
        let origin_y = dock.region_frame.y + dock.content_frame.y;
        let metrics = current_hierarchy_row_metrics();
        let press = UiPoint::new(
            origin_x + dock.content_frame.width * 0.5,
            origin_y + hierarchy_row_y(metrics, 0, 0.0) + metrics.row_height - 0.5,
        );
        let release = UiPoint::new(press.x, origin_y + hierarchy_row_y(metrics, 1, 0.0) + 0.5);
        Self {
            harness,
            source,
            target,
            press,
            release,
        }
    }

    fn press_and_move(&self) {
        self.harness
            .root_ui
            .dispatch_native_primary_press_for_test(self.press.x, self.press.y);
        self.harness
            .root_ui
            .dispatch_native_pointer_move_for_test(self.press.x + 4.0, self.press.y);
        let host = self.harness.host.borrow();
        assert!(host.active_scene_drag_payload.is_some());
        assert!(host.active_hierarchy_drag_node_ids.contains(&self.source));
    }

    fn release(&self) {
        self.harness
            .root_ui
            .dispatch_native_primary_release_for_test(self.release.x, self.release.y);
    }

    fn replace_same_shape_world(&self, refresh_before_release: bool) {
        let mut host = self.harness.host.borrow_mut();
        let old_gateway = host.runtime.world_gateway_identity(WorldDomain::Edit);
        let mut replacement = Some(AuthoringWorldSeed::from(
            DefaultLevelManager::default().create_default_level(),
        ));
        host.runtime
            .shell()
            .lock()
            .state
            .reload_active_scene_world(&mut replacement, None)
            .unwrap();
        assert!(replacement.is_none());
        assert_ne!(
            host.runtime.world_gateway_identity(WorldDomain::Edit),
            old_gateway
        );
        let projected_count = host.hierarchy_scene_entries.len();
        if refresh_before_release {
            host.sync_active_hierarchy_world();
            host.refresh_ui();
            host.recompute_if_dirty();
        }
        drop(host);
        assert_eq!(parents(&self.harness).len(), projected_count);
        assert!(parents(&self.harness)
            .iter()
            .any(|(id, _)| *id == self.source));
    }
}

fn parents(harness: &ChildWindowHostHarness) -> Vec<(u64, Option<u64>)> {
    harness
        .host
        .borrow()
        .runtime
        .shell()
        .lock()
        .state
        .world
        .expect_with_world(|scene| {
            scene
                .nodes()
                .iter()
                .map(|node| (node.id, node.parent))
                .collect()
        })
}

fn history(harness: &ChildWindowHostHarness) -> HistoryStatus {
    let host = harness.host.borrow();
    let shell = host.runtime.shell().lock();
    shell
        .state
        .transactions()
        .history_status(HistoryContextId::Document(
            shell.state.active_scene_document.unwrap(),
        ))
        .unwrap()
}

fn reparent_events_since(harness: &ChildWindowHostHarness, baseline: usize) -> usize {
    harness
        .delta_events_since(baseline)
        .into_iter()
        .filter(|event| {
            matches!(
                event,
                EditorEvent::Hierarchy(EditorHierarchyEvent::ReparentNodes { .. })
            )
        })
        .count()
}

#[test]
fn same_shape_world_replacement_rejects_stale_native_up_before_and_after_reflow() {
    let _guard = lock_env();
    for refresh in [false, true] {
        let fixture = IdentityFixture::new("zircon_hierarchy_identity_replacement");
        fixture.press_and_move();
        fixture.replace_same_shape_world(refresh);
        let before = parents(&fixture.harness);
        let history_before = history(&fixture.harness);
        let journal = fixture.harness.journal_len();
        fixture.release();
        assert_eq!(parents(&fixture.harness), before);
        assert_eq!(history(&fixture.harness), history_before);
        assert_eq!(reparent_events_since(&fixture.harness, journal), 0);
        let host = fixture.harness.host.borrow();
        assert!(host.active_scene_drag_payload.is_none());
        assert!(host.active_hierarchy_drag_node_ids.is_empty());
    }
}

#[test]
fn stale_scene_instance_payload_cannot_fall_through_any_reference_field_action() {
    let _guard = lock_env();
    for (control, action) in [
        (
            "InstanceFieldDemo",
            "UiComponentShowcase/InstanceFieldDropped",
        ),
        ("AssetFieldDemo", "UiComponentShowcase/AssetFieldDropped"),
        ("ObjectFieldDemo", "UiComponentShowcase/ObjectFieldDropped"),
    ] {
        let fixture = IdentityFixture::new("zircon_hierarchy_identity_reference_drop");
        fixture.press_and_move();
        fixture.replace_same_shape_world(false);
        let journal = fixture.harness.journal_len();
        let before = fixture
            .harness
            .host
            .borrow()
            .component_showcase_runtime
            .showcase_demo_state()
            .value_text(control, "value");
        {
            let mut host = fixture.harness.host.borrow_mut();
            host.dispatch_component_showcase_control_activated(control, action);
            assert!(host.active_scene_drag_payload.is_none());
            assert!(host.active_hierarchy_drag_identity.is_none());
        }
        assert_eq!(
            fixture
                .harness
                .host
                .borrow()
                .component_showcase_runtime
                .showcase_demo_state()
                .value_text(control, "value"),
            before,
        );
        assert_eq!(reparent_events_since(&fixture.harness, journal), 0);
    }
}

#[test]
fn reference_fields_keep_their_demo_fallback_without_an_active_payload() {
    let _guard = lock_env();
    let harness = ChildWindowHostHarness::new("zircon_hierarchy_identity_demo_fallback");
    for (control, action, expected) in [
        (
            "InstanceFieldDemo",
            "UiComponentShowcase/InstanceFieldDropped",
            "scene://Root/RuntimeDemoLight",
        ),
        (
            "AssetFieldDemo",
            "UiComponentShowcase/AssetFieldDropped",
            "res://materials/runtime_demo.mat",
        ),
        (
            "ObjectFieldDemo",
            "UiComponentShowcase/ObjectFieldDropped",
            "object://Selection/RuntimeDemo",
        ),
    ] {
        harness
            .host
            .borrow_mut()
            .dispatch_component_showcase_control_activated(control, action);
        assert_eq!(
            harness
                .host
                .borrow()
                .component_showcase_runtime
                .showcase_demo_state()
                .value_text(control, "value")
                .as_deref(),
            Some(expected),
        );
    }
}

#[test]
fn unchanged_world_preserves_native_reparent_and_undo() {
    let _guard = lock_env();
    let fixture = IdentityFixture::new("zircon_hierarchy_identity_current");
    let initial = parents(&fixture.harness);
    let history_before = history(&fixture.harness);
    fixture.press_and_move();
    let journal = fixture.harness.journal_len();
    fixture.release();
    assert_eq!(
        parents(&fixture.harness)
            .iter()
            .find(|(id, _)| *id == fixture.source)
            .unwrap()
            .1,
        Some(fixture.target)
    );
    assert_eq!(history(&fixture.harness).len, history_before.len + 1);
    assert_eq!(reparent_events_since(&fixture.harness, journal), 1);
    assert!(fixture
        .harness
        .host
        .borrow()
        .runtime
        .shell()
        .lock()
        .state
        .apply_intent(crate::core::editing::intent::EditorIntent::Undo)
        .unwrap());
    assert_eq!(parents(&fixture.harness), initial);
}

#[test]
fn document_binding_switch_retires_native_gesture_even_when_node_ids_match() {
    let _guard = lock_env();
    let fixture = IdentityFixture::new("zircon_hierarchy_identity_document_switch");
    fixture.press_and_move();
    let before = parents(&fixture.harness);
    let journal = fixture.harness.journal_len();
    fixture
        .harness
        .host
        .borrow()
        .runtime
        .bind_scene_document(DocumentId::new(2));
    fixture.release();
    assert_eq!(parents(&fixture.harness), before);
    assert_eq!(reparent_events_since(&fixture.harness, journal), 0);
    assert!(fixture
        .harness
        .host
        .borrow()
        .active_hierarchy_drag_identity
        .is_none());
}

#[test]
fn callback_window_route_cannot_consume_another_windows_hierarchy_press() {
    let _guard = lock_env();
    let fixture = IdentityFixture::new("zircon_hierarchy_identity_window_route");
    fixture.press_and_move();
    let before = parents(&fixture.harness);
    let journal = fixture.harness.journal_len();
    fixture
        .harness
        .host
        .borrow_mut()
        .with_callback_source_window(Some(MainPageId::new("window:other-hierarchy")), |host| {
            host.hierarchy_pointer_event(2, 1, fixture.release.x, fixture.release.y, 0.0, 0.0)
        });
    assert_eq!(parents(&fixture.harness), before);
    assert_eq!(reparent_events_since(&fixture.harness, journal), 0);
    assert!(fixture
        .harness
        .host
        .borrow()
        .active_hierarchy_drag_identity
        .is_none());
}

#[test]
fn competing_reference_press_clears_hierarchy_payload_and_its_identity() {
    let _guard = lock_env();
    let fixture = IdentityFixture::new("zircon_hierarchy_identity_overwrite");
    fixture.press_and_move();
    pane_surface_host(&fixture.harness.root_ui)
        .invoke_inspector_reference_pointer_event(0, 1, 12.0, 10.0, 260.0, 180.0);
    let host = fixture.harness.host.borrow();
    assert!(host.active_scene_drag_payload.is_none());
    assert!(host.active_hierarchy_drag_identity.is_none());
    assert!(host.active_hierarchy_drag_node_ids.is_empty());
}
