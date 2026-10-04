use super::support::*;
use crate::core::editing::engine::{HistoryContextId, HistoryStatus};
use crate::core::editing::intent::EditorIntent;
use crate::core::editor_event::EditorHierarchyEvent;
use crate::ui::retained_host::app::hierarchy_pointer::HierarchyTerminalReason;
use crate::ui::retained_host::hierarchy_pointer::{current_hierarchy_row_metrics, hierarchy_row_y};
use zircon_runtime_interface::ui::dispatch::UiPointerId;
use zircon_runtime_interface::ui::layout::UiPoint;

struct NativeHierarchyFixture {
    harness: ChildWindowHostHarness,
    child: UiHostWindow,
    source: u64,
    target: u64,
    press: UiPoint,
    release: UiPoint,
    tab_press: UiPoint,
    baseline_parents: Vec<(u64, Option<u64>)>,
    baseline_history: HistoryStatus,
}

impl NativeHierarchyFixture {
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
        let child = harness.detach_view_to_child_window("editor.hierarchy#1", "window:hierarchy");
        let presentation = child.get_host_presentation();
        let floating = presentation
            .host_scene_data
            .floating_layer
            .floating_windows
            .iter()
            .find(|window| window.window_id.as_str() == "window:hierarchy")
            .expect("detached hierarchy must be presented by the child UiHostWindow");
        assert_eq!(floating.active_pane.kind.as_str(), "Hierarchy");
        let content_x = floating.frame.x + 1.0;
        let content_y = floating.frame.y + floating.header_frame.height.max(0.0) + 1.0;
        let content_width = (floating.frame.width - 2.0).max(0.0);
        let metrics = current_hierarchy_row_metrics();
        let press = UiPoint::new(
            content_x + content_width * 0.5,
            content_y + hierarchy_row_y(metrics, 0, 0.0) + metrics.row_height - 0.5,
        );
        let release = UiPoint::new(press.x, content_y + hierarchy_row_y(metrics, 1, 0.0) + 0.5);
        let tab = floating
            .tab_frames
            .get(0)
            .expect("detached hierarchy must expose a native tab route");
        assert!(tab.frame.width > 8.0);
        let tab_press = UiPoint::new(
            floating.frame.x + floating.header_frame.x + tab.frame.x + 4.0,
            floating.frame.y + floating.header_frame.y + tab.frame.y + tab.frame.height * 0.5,
        );
        let baseline_parents = parents(&harness);
        let baseline_history = history(&harness);
        Self {
            harness,
            child,
            source,
            target,
            press,
            release,
            tab_press,
            baseline_parents,
            baseline_history,
        }
    }

    fn activate(&self) {
        self.child
            .dispatch_native_primary_press_for_test(self.press.x, self.press.y);
        self.child
            .dispatch_native_pointer_move_for_test(self.press.x + 4.0, self.press.y);
        let host = self.harness.host.borrow();
        assert!(host.active_hierarchy_drag_node_ids.contains(&self.source));
        assert!(host.active_scene_drag_payload.is_some());
    }

    fn terminal_count(&self) -> u64 {
        self.harness
            .host
            .borrow()
            .hierarchy_input_owner
            .terminal_count()
    }

    fn last_terminal(&self) -> Option<(u64, HierarchyTerminalReason)> {
        self.harness
            .host
            .borrow()
            .hierarchy_input_owner
            .last_terminal()
            .map(|receipt| (receipt.generation, receipt.reason))
    }

    fn assert_canceled_without_reparent(&self, journal: usize, terminal_before: u64) {
        assert_eq!(parents(&self.harness), self.baseline_parents);
        assert_eq!(history(&self.harness), self.baseline_history);
        assert_eq!(reparent_events_since(&self.harness, journal), 0);
        let host = self.harness.host.borrow();
        assert!(host.active_scene_drag_payload.is_none());
        assert!(host.active_hierarchy_drag_node_ids.is_empty());
        assert!(host.active_hierarchy_drag_identity.is_none());
        assert_eq!(
            host.hierarchy_input_owner.terminal_count(),
            terminal_before + 1
        );
    }

    fn late_owner_up(&self) {
        self.child
            .dispatch_native_primary_release_for_test(self.release.x, self.release.y);
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
fn child_hierarchy_owner_release_in_root_cancels_once_before_late_child_up() {
    let _guard = lock_env();
    let fixture = NativeHierarchyFixture::new("zircon_hierarchy_native_cross_window_release");
    fixture.activate();
    let journal = fixture.harness.journal_len();
    let terminal_before = fixture.terminal_count();

    fixture
        .harness
        .root_ui
        .dispatch_native_primary_release_for_test(4.0, 4.0);
    fixture.assert_canceled_without_reparent(journal, terminal_before);
    let receipt = fixture
        .last_terminal()
        .expect("cross-window terminal receipt");
    assert!(receipt.0 > 0);
    assert_eq!(receipt.1, HierarchyTerminalReason::ForeignWindow);
    fixture.late_owner_up();
    fixture.assert_canceled_without_reparent(journal, terminal_before);
    assert_eq!(fixture.last_terminal(), Some(receipt));
}

#[test]
fn child_hierarchy_outside_release_cancels_before_late_inside_up() {
    let _guard = lock_env();
    let fixture = NativeHierarchyFixture::new("zircon_hierarchy_native_outside_release");
    fixture.activate();
    let journal = fixture.harness.journal_len();
    let terminal_before = fixture.terminal_count();

    fixture
        .child
        .dispatch_native_primary_release_for_test(-16.0, -16.0);
    fixture.assert_canceled_without_reparent(journal, terminal_before);
    fixture.late_owner_up();
    fixture.assert_canceled_without_reparent(journal, terminal_before);
}

#[test]
fn child_hierarchy_outside_move_cancels_before_late_inside_up() {
    let _guard = lock_env();
    let fixture = NativeHierarchyFixture::new("zircon_hierarchy_native_outside_move");
    fixture.activate();
    let journal = fixture.harness.journal_len();
    let terminal_before = fixture.terminal_count();

    fixture
        .child
        .dispatch_native_pointer_move_for_test(-16.0, -16.0);
    fixture.assert_canceled_without_reparent(journal, terminal_before);
    fixture.late_owner_up();
    fixture.assert_canceled_without_reparent(journal, terminal_before);
}

#[test]
fn native_pointer_cancel_terminalizes_child_hierarchy_once() {
    let _guard = lock_env();
    let fixture = NativeHierarchyFixture::new("zircon_hierarchy_native_pointer_cancel");
    fixture.activate();
    let journal = fixture.harness.journal_len();
    let terminal_before = fixture.terminal_count();

    fixture
        .child
        .dispatch_native_pointer_cancel_for_test(fixture.press.x, fixture.press.y);
    fixture.assert_canceled_without_reparent(journal, terminal_before);
    fixture.late_owner_up();
    fixture.assert_canceled_without_reparent(journal, terminal_before);
}

#[test]
fn ordinary_hierarchy_refresh_keeps_child_window_press_authority() {
    let _guard = lock_env();
    let fixture = NativeHierarchyFixture::new("zircon_hierarchy_native_child_refresh");
    fixture.activate();
    let terminal_before = fixture.terminal_count();

    fixture
        .harness
        .host
        .borrow_mut()
        .sync_active_hierarchy_world();
    assert_eq!(fixture.terminal_count(), terminal_before);
    assert!(fixture
        .harness
        .host
        .borrow()
        .active_scene_drag_payload
        .is_some());
}

#[test]
fn different_pointer_in_root_does_not_steal_child_hierarchy_owner() {
    let _guard = lock_env();
    let fixture = NativeHierarchyFixture::new("zircon_hierarchy_native_foreign_pointer");
    let owner = UiPointerId::new(7);
    let foreign = UiPointerId::new(9);
    fixture
        .child
        .dispatch_native_primary_press_with_pointer_for_test(
            owner,
            fixture.press.x,
            fixture.press.y,
        );
    fixture
        .child
        .dispatch_native_pointer_move_with_pointer_for_test(
            owner,
            fixture.press.x + 4.0,
            fixture.press.y,
        );
    let journal = fixture.harness.journal_len();
    let terminal_before = fixture.terminal_count();

    fixture
        .harness
        .root_ui
        .dispatch_native_primary_release_with_pointer_for_test(foreign, 4.0, 4.0);
    assert_eq!(fixture.terminal_count(), terminal_before);
    assert!(fixture
        .harness
        .host
        .borrow()
        .active_scene_drag_payload
        .is_some());

    fixture
        .child
        .dispatch_native_primary_release_with_pointer_for_test(
            owner,
            fixture.release.x,
            fixture.release.y,
        );
    assert_eq!(reparent_events_since(&fixture.harness, journal), 1);
    assert_eq!(fixture.terminal_count(), terminal_before + 1);
    assert_eq!(
        parents(&fixture.harness)
            .iter()
            .find(|(id, _)| *id == fixture.source)
            .unwrap()
            .1,
        Some(fixture.target)
    );
    fixture
        .child
        .dispatch_native_primary_release_with_pointer_for_test(
            owner,
            fixture.release.x,
            fixture.release.y,
        );
    assert_eq!(reparent_events_since(&fixture.harness, journal), 1);
    assert_eq!(fixture.terminal_count(), terminal_before + 1);
}

#[test]
fn different_pointer_down_and_up_inside_child_does_not_steal_hierarchy_owner() {
    let _guard = lock_env();
    let fixture = NativeHierarchyFixture::new("zircon_hierarchy_native_foreign_inside");
    let owner = UiPointerId::new(7);
    let foreign = UiPointerId::new(9);
    fixture
        .child
        .dispatch_native_primary_press_with_pointer_for_test(
            owner,
            fixture.press.x,
            fixture.press.y,
        );
    fixture
        .child
        .dispatch_native_pointer_move_with_pointer_for_test(
            owner,
            fixture.press.x + 4.0,
            fixture.press.y,
        );
    let journal = fixture.harness.journal_len();
    let terminal_before = fixture.terminal_count();

    fixture
        .child
        .dispatch_native_primary_press_with_pointer_for_test(
            foreign,
            fixture.press.x,
            fixture.press.y,
        );
    fixture
        .child
        .dispatch_native_primary_release_with_pointer_for_test(
            foreign,
            fixture.release.x,
            fixture.release.y,
        );
    assert_eq!(fixture.terminal_count(), terminal_before);
    assert_eq!(reparent_events_since(&fixture.harness, journal), 0);
    assert!(fixture
        .harness
        .host
        .borrow()
        .active_scene_drag_payload
        .is_some());

    fixture
        .child
        .dispatch_native_primary_release_with_pointer_for_test(
            owner,
            fixture.release.x,
            fixture.release.y,
        );
    assert_eq!(reparent_events_since(&fixture.harness, journal), 1);
    assert_eq!(fixture.terminal_count(), terminal_before + 1);
    assert_eq!(
        parents(&fixture.harness)
            .iter()
            .find(|(id, _)| *id == fixture.source)
            .unwrap()
            .1,
        Some(fixture.target)
    );
}

#[test]
fn different_pointer_move_inside_child_cannot_activate_owner_reparent() {
    let _guard = lock_env();
    let fixture = NativeHierarchyFixture::new("zircon_hierarchy_native_foreign_move");
    let owner = UiPointerId::new(7);
    let foreign = UiPointerId::new(9);
    fixture
        .child
        .dispatch_native_primary_press_with_pointer_for_test(
            owner,
            fixture.press.x,
            fixture.press.y,
        );
    let journal = fixture.harness.journal_len();
    let terminal_before = fixture.terminal_count();

    fixture
        .child
        .dispatch_native_pointer_move_with_pointer_for_test(
            foreign,
            fixture.release.x,
            fixture.release.y,
        );
    assert_eq!(fixture.terminal_count(), terminal_before);
    {
        let host = fixture.harness.host.borrow();
        assert!(host.active_scene_drag_payload.is_some());
        assert_eq!(host.hierarchy_pointer_state.hovered_item_index, Some(1));
    }

    fixture
        .child
        .dispatch_native_primary_release_with_pointer_for_test(
            owner,
            fixture.release.x,
            fixture.release.y,
        );
    fixture.assert_canceled_without_reparent(journal, terminal_before);
}

#[test]
fn touch_like_native_move_cannot_activate_mouse_hierarchy_owner() {
    let _guard = lock_env();
    let fixture = NativeHierarchyFixture::new("zircon_hierarchy_native_touch_move");
    fixture
        .child
        .dispatch_native_primary_press_for_test(fixture.press.x, fixture.press.y);
    let journal = fixture.harness.journal_len();
    let terminal_before = fixture.terminal_count();

    fixture
        .child
        .dispatch_native_touch_like_pointer_move_for_test(
            UiPointerId::default(),
            fixture.release.x,
            fixture.release.y,
        );
    assert_eq!(fixture.terminal_count(), terminal_before);
    fixture.late_owner_up();
    fixture.assert_canceled_without_reparent(journal, terminal_before);
}

#[test]
fn untranslated_native_move_cannot_activate_mouse_hierarchy_owner() {
    let _guard = lock_env();
    let fixture = NativeHierarchyFixture::new("zircon_hierarchy_native_untranslated_move");
    fixture
        .child
        .dispatch_native_primary_press_for_test(fixture.press.x, fixture.press.y);
    let journal = fixture.harness.journal_len();
    let terminal_before = fixture.terminal_count();

    fixture
        .child
        .dispatch_native_untranslated_pointer_move_for_test(fixture.release.x, fixture.release.y);
    assert_eq!(fixture.terminal_count(), terminal_before);
    fixture.late_owner_up();
    fixture.assert_canceled_without_reparent(journal, terminal_before);
}

#[test]
fn foreign_tab_capture_preserves_hierarchy_owner_release() {
    let _guard = lock_env();
    let fixture = NativeHierarchyFixture::new("zircon_hierarchy_native_foreign_tab_capture");
    let owner = UiPointerId::new(7);
    let foreign = UiPointerId::new(9);
    fixture
        .child
        .dispatch_native_primary_press_with_pointer_for_test(
            owner,
            fixture.press.x,
            fixture.press.y,
        );
    fixture
        .child
        .dispatch_native_pointer_move_with_pointer_for_test(
            owner,
            fixture.press.x + 4.0,
            fixture.press.y,
        );
    let journal = fixture.harness.journal_len();
    let terminal_before = fixture.terminal_count();

    fixture
        .child
        .dispatch_native_primary_press_with_pointer_for_test(
            foreign,
            fixture.tab_press.x,
            fixture.tab_press.y,
        );
    assert!(!host_context(&fixture.child)
        .get_drag_state()
        .drag_tab_id
        .is_empty());
    assert!(fixture
        .harness
        .host
        .borrow()
        .active_scene_drag_payload
        .is_some());
    fixture
        .child
        .dispatch_native_primary_release_with_pointer_for_test(
            owner,
            fixture.release.x,
            fixture.release.y,
        );
    assert_eq!(reparent_events_since(&fixture.harness, journal), 1);
    assert_eq!(fixture.terminal_count(), terminal_before + 1);
    assert_eq!(
        parents(&fixture.harness)
            .iter()
            .find(|(id, _)| *id == fixture.source)
            .unwrap()
            .1,
        Some(fixture.target)
    );
    assert_ne!(history(&fixture.harness), fixture.baseline_history);
    assert!(fixture
        .harness
        .host
        .borrow()
        .active_scene_drag_payload
        .is_none());
    assert!(!host_context(&fixture.child)
        .get_drag_state()
        .drag_tab_id
        .is_empty());
    fixture
        .child
        .dispatch_native_primary_release_with_pointer_for_test(
            owner,
            fixture.release.x,
            fixture.release.y,
        );
    assert_eq!(reparent_events_since(&fixture.harness, journal), 1);
    assert_eq!(fixture.terminal_count(), terminal_before + 1);
    fixture
        .child
        .dispatch_native_primary_release_with_pointer_for_test(
            foreign,
            fixture.tab_press.x,
            fixture.tab_press.y,
        );
    assert!(host_context(&fixture.child)
        .get_drag_state()
        .drag_tab_id
        .is_empty());
    assert_eq!(reparent_events_since(&fixture.harness, journal), 1);
}

#[test]
fn unrelated_root_focus_loss_does_not_cancel_child_hierarchy_owner() {
    let _guard = lock_env();
    let fixture = NativeHierarchyFixture::new("zircon_hierarchy_native_other_focus");
    fixture.activate();
    let journal = fixture.harness.journal_len();
    let terminal_before = fixture.terminal_count();

    host_context(&fixture.harness.root_ui).invoke_native_window_focus_lost();
    assert_eq!(fixture.terminal_count(), terminal_before);
    fixture.late_owner_up();
    assert_eq!(reparent_events_since(&fixture.harness, journal), 1);
    assert_eq!(fixture.terminal_count(), terminal_before + 1);
}

#[test]
fn escape_secondary_press_and_focus_loss_each_cancel_one_native_hierarchy_gesture() {
    for reason in ["escape", "secondary", "focus"] {
        let _guard = lock_env();
        let fixture = NativeHierarchyFixture::new(&format!("zircon_hierarchy_native_{reason}"));
        fixture.activate();
        let journal = fixture.harness.journal_len();
        let terminal_before = fixture.terminal_count();
        match reason {
            "escape" => {
                fixture.child.dispatch_native_key_for_test(
                    key_event(
                        Key::Named(NamedKey::Escape),
                        PhysicalKey::Code(KeyCode::Escape),
                        None,
                        ElementState::Pressed,
                    ),
                    ModifiersState::empty(),
                );
            }
            "secondary" => {
                fixture
                    .child
                    .dispatch_native_secondary_press_for_test(fixture.press.x, fixture.press.y);
            }
            "focus" => host_context(&fixture.child).invoke_native_window_focus_lost(),
            _ => unreachable!(),
        }
        fixture.assert_canceled_without_reparent(journal, terminal_before);
        fixture.late_owner_up();
        fixture.assert_canceled_without_reparent(journal, terminal_before);
    }
}

#[test]
fn child_window_close_request_cancels_native_hierarchy_gesture_once() {
    let _guard = lock_env();
    let fixture = NativeHierarchyFixture::new("zircon_hierarchy_native_window_close");
    fixture.activate();
    let journal = fixture.harness.journal_len();
    let terminal_before = fixture.terminal_count();

    fixture.child.dispatch_native_close_request_for_test();
    fixture.assert_canceled_without_reparent(journal, terminal_before);
    fixture.late_owner_up();
    fixture.assert_canceled_without_reparent(journal, terminal_before);
}

#[test]
fn main_window_close_request_cancels_child_hierarchy_gesture_once() {
    let _guard = lock_env();
    let fixture = NativeHierarchyFixture::new("zircon_hierarchy_native_main_close");
    fixture.activate();
    let journal = fixture.harness.journal_len();
    let terminal_before = fixture.terminal_count();

    fixture
        .harness
        .host
        .borrow_mut()
        .native_main_window_close_requested();
    fixture.assert_canceled_without_reparent(journal, terminal_before);
    fixture.late_owner_up();
    fixture.assert_canceled_without_reparent(journal, terminal_before);
}
