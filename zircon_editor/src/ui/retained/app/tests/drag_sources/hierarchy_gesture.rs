use super::*;
use crate::core::editing::engine::{HistoryContextId, HistoryStatus};
use crate::core::editing::intent::EditorIntent;
use crate::core::editor_event::EditorHierarchyEvent;
use crate::ui::retained_host::hierarchy_pointer::{current_hierarchy_row_metrics, hierarchy_row_y};
use zircon_runtime::scene::components::NodeKind;
use zircon_runtime_interface::ui::layout::UiPoint;

struct GestureFixture {
    harness: ChildWindowHostHarness,
    source: u64,
    target: u64,
    press: UiPoint,
    release: UiPoint,
    parents: Vec<(u64, Option<u64>)>,
    history: HistoryStatus,
}

impl GestureFixture {
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
                .apply_intent(EditorIntent::SelectNode(source))
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
        assert!((release.y - press.y - 2.0).abs() < 0.001);
        let parents = parents(&harness);
        assert_eq!(
            parents.iter().find(|(id, _)| *id == source).unwrap().1,
            None
        );
        assert_eq!(
            parents.iter().find(|(id, _)| *id == target).unwrap().1,
            None
        );
        let history = history(&harness);
        Self {
            harness,
            source,
            target,
            press,
            release,
            parents,
            history,
        }
    }

    fn press(&self) {
        self.harness
            .root_ui
            .dispatch_native_primary_press_for_test(self.press.x, self.press.y);
        let host = self.harness.host.borrow();
        assert!(host.active_scene_drag_payload.is_some());
        assert!(host.active_hierarchy_drag_node_ids.contains(&self.source));
        assert_eq!(
            host.runtime
                .shell()
                .lock()
                .state
                .viewport_controller
                .selection()
                .active_primary(),
            Some(self.source),
            "the real native press must keep normal row selection",
        );
    }

    fn move_by(&self, dx: f32, dy: f32) {
        self.harness
            .root_ui
            .dispatch_native_pointer_move_for_test(self.press.x + dx, self.press.y + dy);
    }

    fn release(&self) {
        self.harness
            .root_ui
            .dispatch_native_primary_release_for_test(self.release.x, self.release.y);
        let host = self.harness.host.borrow();
        assert!(host.active_scene_drag_payload.is_none());
        assert!(host.active_hierarchy_drag_node_ids.is_empty());
    }

    fn assert_unchanged(&self, journal_before: usize) {
        assert_eq!(parents(&self.harness), self.parents);
        assert_eq!(history(&self.harness), self.history);
        assert_eq!(reparent_events_since(&self.harness, journal_before), 0);
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
fn hierarchy_native_press_release_without_move_preserves_world_and_history() {
    let _guard = lock_env();
    let fixture = GestureFixture::new("zircon_hierarchy_no_move");
    fixture.press();
    let journal = fixture.harness.journal_len();
    fixture.release();
    fixture.assert_unchanged(journal);
}

#[test]
fn hierarchy_native_subthreshold_jitter_preserves_world_and_history() {
    let _guard = lock_env();
    for (dx, dy) in [
        (0.0, 2.0),
        (3.99, 0.0),
        (-3.99, 0.0),
        (2.0, 2.0),
        (-2.0, -2.0),
    ] {
        let fixture = GestureFixture::new("zircon_hierarchy_subthreshold");
        fixture.press();
        let journal = fixture.harness.journal_len();
        fixture.move_by(dx, dy);
        fixture.release();
        fixture.assert_unchanged(journal);
    }
}

#[test]
fn hierarchy_native_threshold_drag_reparents_once_in_every_direction_and_undo_restores_world() {
    let _guard = lock_env();
    for (dx, dy) in [
        (4.0, 0.0),
        (-4.0, 0.0),
        (0.0, 4.0),
        (0.0, -4.0),
        (3.0, 3.0),
        (-3.0, -3.0),
    ] {
        let fixture = GestureFixture::new("zircon_hierarchy_threshold");
        fixture.press();
        let journal = fixture.harness.journal_len();
        fixture.move_by(dx, dy);
        fixture.release();
        assert_eq!(
            parents(&fixture.harness)
                .iter()
                .find(|(id, _)| *id == fixture.source)
                .unwrap()
                .1,
            Some(fixture.target),
        );
        let committed = history(&fixture.harness);
        assert_eq!(committed.len, fixture.history.len + 1);
        assert_eq!(reparent_events_since(&fixture.harness, journal), 1);
        fixture.release();
        assert_eq!(history(&fixture.harness), committed);
        assert_eq!(reparent_events_since(&fixture.harness, journal), 1);

        assert!(fixture
            .harness
            .host
            .borrow()
            .runtime
            .shell()
            .lock()
            .state
            .apply_intent(EditorIntent::Undo)
            .unwrap());
        assert_eq!(parents(&fixture.harness), fixture.parents);
    }
}

#[test]
fn hierarchy_native_new_press_resets_an_activated_gesture() {
    let _guard = lock_env();
    let fixture = GestureFixture::new("zircon_hierarchy_new_press");
    fixture.press();
    fixture.move_by(4.0, 0.0);
    fixture.press();
    let journal = fixture.harness.journal_len();
    fixture.release();
    fixture.assert_unchanged(journal);
}

#[test]
fn hierarchy_owner_outside_or_invalid_motion_retires_armed_and_dragging_gestures() {
    let _guard = lock_env();
    for dragging in [false, true] {
        for point in [UiPoint::new(-1.0, 40.0), UiPoint::new(f32::NAN, 40.0)] {
            let fixture = GestureFixture::new("zircon_hierarchy_leave");
            fixture.press();
            if dragging {
                fixture.move_by(4.0, 0.0);
            }
            let journal = fixture.harness.journal_len();
            // Exercise the real owner's pane callback. Native cross-window capture
            // and Escape/focus-loss routing are separate qualification gates.
            pane_surface_host(&fixture.harness.root_ui)
                .invoke_hierarchy_pointer_moved(point.x, point.y, 0.0, 0.0);
            fixture.move_by(8.0, 0.0);
            fixture.release();
            fixture.assert_unchanged(journal);
        }
    }
}

#[test]
fn hierarchy_owner_geometry_reset_retires_armed_and_dragging_gestures() {
    let _guard = lock_env();
    for dragging in [false, true] {
        let fixture = GestureFixture::new("zircon_hierarchy_geometry_reset");
        fixture.press();
        if dragging {
            fixture.move_by(4.0, 0.0);
        }
        let journal = fixture.harness.journal_len();
        let size = fixture.harness.host.borrow().hierarchy_pointer_size;
        pane_surface_host(&fixture.harness.root_ui).invoke_hierarchy_pointer_moved(
            80.0,
            40.0,
            size.width + 1.0,
            size.height,
        );
        fixture.move_by(8.0, 0.0);
        fixture.release();
        fixture.assert_unchanged(journal);
    }
}

#[test]
fn hierarchy_owner_scene_structure_refresh_retires_an_activated_gesture() {
    let _guard = lock_env();
    let fixture = GestureFixture::new("zircon_hierarchy_structure_reset");
    fixture.press();
    fixture.move_by(4.0, 0.0);
    {
        let mut host = fixture.harness.host.borrow_mut();
        let row_count = host.hierarchy_scene_entries.len();
        host.runtime
            .shell()
            .lock()
            .state
            .apply_intent(EditorIntent::CreateNode(NodeKind::Empty))
            .unwrap();
        host.sync_active_hierarchy_world();
        host.refresh_ui();
        host.recompute_if_dirty();
        assert_eq!(host.hierarchy_scene_entries.len(), row_count + 1);
    }
    let before = parents(&fixture.harness);
    let history_before = history(&fixture.harness);
    let journal = fixture.harness.journal_len();
    fixture.move_by(8.0, 0.0);
    fixture.release();
    assert_eq!(parents(&fixture.harness), before);
    assert_eq!(history(&fixture.harness), history_before);
    assert_eq!(reparent_events_since(&fixture.harness, journal), 0);
}
