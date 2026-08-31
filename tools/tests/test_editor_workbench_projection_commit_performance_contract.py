from pathlib import Path
import unittest

from tools.editor_workbench_projection_pressure import run


ROOT = Path(__file__).resolve().parents[2]
HOST = ROOT / "zircon_editor/src/ui/retained_host"
EVENT_BRIDGE = HOST / "event_bridge.rs"
DISPATCH_EFFECTS = HOST / "app/host_lifecycle/dispatch_effects.rs"
DISPATCH_SIDE_EFFECTS = HOST / "app/host_lifecycle/dispatch_effects/side_effects.rs"
INVALIDATION_DECISION = (
    HOST / "app/host_lifecycle/recompute/invalidation/decision.rs"
)
CHROME_PROJECTION = (
    HOST / "app/viewport/toolbar_pointer/chrome_projection.rs"
)
NATIVE_WINDOW_STORE = HOST / "app/native_windows/store.rs"
WORKBENCH_BRIDGE = (
    HOST
    / "callback_dispatch/template_bridge/workbench/componentized_window.rs"
)
TEMPLATE_SURFACE = (
    ROOT / "zircon_editor/src/ui/workbench/reference/template_surface.rs"
)
COMPONENTIZED_WORKBENCH_TESTS = (
    HOST / "app/tests/componentized_workbench.rs"
)
WORKBENCH_CONTEXT_MENU = HOST / "app/workbench_context_menu.rs"
WORKBENCH_NOTIFICATIONS = HOST / "app/workbench_notifications.rs"
WORKBENCH_NOTIFICATION_TESTS = (
    ROOT / "zircon_editor/src/tests/host/retained_event_bridge/workbench_notifications.rs"
)
WORKBENCH_CONTROL_DISPATCH = (
    HOST / "callback_dispatch/workbench/control.rs"
)


class EditorWorkbenchProjectionCommitPerformanceContract(unittest.TestCase):
    def test_pending_bridge_projection_becomes_a_typed_callback_outcome(self) -> None:
        effects = EVENT_BRIDGE.read_text(encoding="utf-8")
        dispatch = DISPATCH_EFFECTS.read_text(encoding="utf-8")
        bridge = WORKBENCH_BRIDGE.read_text(encoding="utf-8")
        surface = TEMPLATE_SURFACE.read_text(encoding="utf-8")

        self.assertIn("fn request_workbench_projection", effects)
        self.assertIn(
            "merge_dirty_domains(HostInvalidationMask::WORKBENCH_PROJECTION)",
            effects,
        )
        self.assertIn("fn has_pending_host_projection_commit", bridge)
        self.assertIn("fn has_pending_host_projection_commit", surface)
        self.assertIn("host_projection_full_refresh_pending", surface)
        self.assertIn("pending_host_projection_patch_indices.is_empty()", surface)

        pending = dispatch.index("has_pending_host_projection_commit")
        promote = dispatch.index("request_workbench_projection", pending)
        invalidate = dispatch.index("let dirty_domains = effects.dirty_domains()")
        self.assertLess(pending, promote)
        self.assertLess(promote, invalidate)
        self.assertNotIn("pending_host_projection_patch_nodes", dispatch)

    def test_projection_commit_accepts_coalesced_render_and_paint_work(self) -> None:
        source = INVALIDATION_DECISION.read_text(encoding="utf-8")

        self.assertIn("fn workbench_projection_reuses_host", source)
        helper = source.split("fn workbench_projection_reuses_host", 1)[1]
        helper = helper.split("fn shell_content_reuses_committed_layout", 1)[0]
        self.assertIn("HostInvalidationMask::WORKBENCH_PROJECTION", helper)
        self.assertIn("HostInvalidationMask::PAINT_ONLY", helper)
        self.assertIn("HostInvalidationMask::RENDER", helper)
        self.assertNotIn("HostInvalidationMask::PRESENTATION_DATA", helper)
        self.assertNotIn("HostInvalidationMask::LAYOUT", helper)
        self.assertIn(
            "workbench_projection_accepts_render_and_paint_without_full_shell_recompute",
            source,
        )
        self.assertIn("workbench_projection_rejects_global_presentation", source)

    def test_viewport_chrome_sync_is_consumed_before_invalidation(self) -> None:
        effects = EVENT_BRIDGE.read_text(encoding="utf-8")
        dispatch = DISPATCH_EFFECTS.read_text(encoding="utf-8")
        chrome = CHROME_PROJECTION.read_text(encoding="utf-8")

        sync = dispatch.index("if effects.sync_viewport_chrome")
        patch = dispatch.index("self.sync_viewport_chrome_projection()", sync)
        request_paint = dispatch.index("effects.request_paint_only()", patch)
        invalidate = dispatch.index("let dirty_domains = effects.dirty_domains()")
        self.assertLess(sync, patch)
        self.assertLess(patch, request_paint)
        self.assertLess(request_paint, invalidate)
        self.assertEqual(1, dispatch.count("self.sync_viewport_chrome_projection()"))
        self.assertNotIn("record_paint_only_invalidation", chrome)
        self.assertIn("-> bool", chrome)

        chrome_event = effects.split(
            "EditorEvent::Viewport(event) if event.changes_chrome_projection()", 1
        )[1].split("match &record.event", 1)[0]
        self.assertIn("target.sync_viewport_chrome = true", chrome_event)
        self.assertNotIn("target.request_paint_only()", chrome_event)

    def test_workbench_projection_commit_preserves_pending_render_submission(self) -> None:
        recompute = (
            HOST / "app/host_lifecycle/recompute.rs"
        ).read_text(encoding="utf-8")
        production = recompute.split("#[cfg(test)]", 1)[0]

        projection = production.index("apply_workbench_projection_presentation")
        full_shell = production.index("build_recompute_shell_snapshot")
        self.assertLess(projection, full_shell)
        self.assertNotIn("self.render_dirty = false", production)

    def test_chrome_patch_queues_its_own_center_and_status_damage(self) -> None:
        chrome = CHROME_PROJECTION.read_text(encoding="utf-8")
        tests = COMPONENTIZED_WORKBENCH_TESTS.read_text(encoding="utf-8")

        patch = chrome.index("patch_scene_viewport_chrome")
        damage = chrome.index("viewport_chrome_damage_frame", patch)
        redraw = chrome.index("request_frame_update_region", damage)
        self.assertLess(patch, damage)
        self.assertLess(damage, redraw)
        self.assertIn(
            "componentized_viewport_chrome_patch_queues_center_status_damage",
            tests,
        )

    def test_chrome_patch_updates_each_native_presenter_and_queues_local_damage(self) -> None:
        chrome = CHROME_PROJECTION.read_text(encoding="utf-8")
        store = NATIVE_WINDOW_STORE.read_text(encoding="utf-8")
        tests = COMPONENTIZED_WORKBENCH_TESTS.read_text(encoding="utf-8")

        self.assertIn("presented_rows", store)
        self.assertIn("patch_scene_viewport_chrome", store)
        native_patch = store.split("fn patch_scene_viewport_chrome", 1)[1]
        native_patch = native_patch.split("fn patch_ui_asset_presentation", 1)[0]
        self.assertIn("window.patch_native_scene_viewport_chrome", native_patch)
        self.assertNotIn("window.patch_scene_viewport_chrome", native_patch)
        self.assertIn(".native_viewport_chrome_damage_frame()", native_patch)
        self.assertIn("window.request_frame_update_region", native_patch)
        self.assertIn("native_window_presenters", chrome)
        self.assertIn("patch_scene_viewport_chrome", chrome)
        self.assertIn(
            "componentized_viewport_chrome_patch_updates_native_presenter_damage",
            tests,
        )
        native_test = tests.split(
            "fn componentized_viewport_chrome_patch_updates_native_presenter_damage", 1
        )[1].split("#[test]", 1)[0]
        self.assertIn("window:hierarchy", native_test)
        self.assertIn("hierarchy.take_external_redraw_for_test()", native_test)

    def test_dispatch_error_explicitly_commits_pending_projection(self) -> None:
        dispatch = DISPATCH_EFFECTS.read_text(encoding="utf-8")
        tests = COMPONENTIZED_WORKBENCH_TESTS.read_text(encoding="utf-8")

        error_branch = dispatch.split("pub(in crate::ui::retained_host::app) fn apply_dispatch_result", 1)[1]
        error_branch = error_branch.split("fn dispatch_error_toast", 1)[0]
        status = error_branch.index("self.set_status_line(error)")
        pending = error_branch.index("commit_pending_workbench_projection", status)
        self.assertLess(status, pending)
        self.assertIn(
            "repeated_dispatch_error_commits_pending_workbench_projection",
            tests,
        )

    def test_context_menu_open_commits_changed_rows_without_global_presentation(self) -> None:
        dispatch = WORKBENCH_CONTEXT_MENU.read_text(encoding="utf-8")
        tests = COMPONENTIZED_WORKBENCH_TESTS.read_text(encoding="utf-8")

        opened = dispatch.split("Ok(true) =>", 1)[1].split("Ok(false)", 1)[0]
        self.assertNotIn("request_presentation()", opened)
        self.assertIn("apply_dispatch_effects", opened)
        self.assertIn(
            "context_menu_open_commits_projection_without_full_shell_recompute",
            tests,
        )

    def test_toast_visibility_uses_projection_with_non_workbench_fallback(self) -> None:
        effects = EVENT_BRIDGE.read_text(encoding="utf-8")
        notifications = WORKBENCH_NOTIFICATIONS.read_text(encoding="utf-8")
        tests = WORKBENCH_NOTIFICATION_TESTS.read_text(encoding="utf-8")

        toast_effect = effects.split(
            "let notifications = toast_notifications_for_record(record);", 1
        )[1].split("for effect in &record.effects", 1)[0]
        self.assertNotIn("request_presentation()", toast_effect)

        publish = notifications.split("fn publish_activity_toasts", 1)[1].split(
            "fn sync_activity_notifications", 1
        )[0]
        self.assertIn("active_activity_window_template_document_is", publish)
        self.assertIn("HostInvalidationMask::PRESENTATION_DATA", publish)
        self.assertIn("self.sync_activity_notifications()", publish)

        import_case = tests.split("EditorAssetEvent::ImportModel", 1)[1]
        self.assertIn("assert!(!effects.presentation_dirty)", import_case)

    def test_projection_pressure_scales_with_changed_rows_not_host_nodes(self) -> None:
        result = run(
            interaction_count=4_096,
            host_node_count=32_768,
            changed_row_count=24,
        )

        self.assertEqual(result["old_full_recompute_node_visits"], 134_217_728)
        self.assertEqual(result["new_projection_row_visits"], 98_304)
        self.assertEqual(result["eliminated_node_or_row_visits"], 134_119_424)
        self.assertAlmostEqual(result["work_reduction_ratio"], 1_365.3333333333333)
        self.assertEqual(result["old_toast_snapshot_collections"], 8_192)
        self.assertEqual(result["new_toast_snapshot_collections"], 4_096)

    def test_pending_decision_button_closes_through_notification_projection(self) -> None:
        dispatch = WORKBENCH_CONTROL_DISPATCH.read_text(encoding="utf-8")
        tests = COMPONENTIZED_WORKBENCH_TESTS.read_text(encoding="utf-8")

        decision = dispatch.split(
            "if bridge.is_pending_activity_decision_option", 1
        )[1].split("let selected = bridge", 1)[0]
        self.assertNotIn("request_presentation()", decision)
        test = tests.split(
            "fn resolved_pending_decision_clears_the_retained_notification_modal", 1
        )[1].split("fn deferred_pending_edit", 1)[0]
        self.assertIn("slow_path_rebuild_count", test)
        self.assertIn("host.recompute_if_dirty()", test)

    def test_dispatch_batch_projects_published_toasts_once(self) -> None:
        notifications = WORKBENCH_NOTIFICATIONS.read_text(encoding="utf-8")
        side_effects = DISPATCH_SIDE_EFFECTS.read_text(encoding="utf-8")

        self.assertIn("fn enqueue_activity_toasts", notifications)
        self.assertIn("fn refresh_activity_notification_presentation", notifications)
        publish = notifications.split("fn publish_activity_toasts", 1)[1].split(
            "fn enqueue_activity_toasts", 1
        )[0]
        self.assertIn("self.enqueue_activity_toasts", publish)
        self.assertIn("self.refresh_activity_notification_presentation()", publish)

        dispatch = side_effects.split("fn apply_dispatch_side_effects", 1)[1]
        self.assertGreaterEqual(dispatch.count("self.enqueue_activity_toasts"), 2)
        self.assertEqual(
            dispatch.count("self.refresh_activity_notification_presentation()"),
            1,
        )

    def test_viewport_toolbar_product_gate_rejects_full_shell_invalidation(self) -> None:
        capture = (ROOT / "tools/ui-profile-capture.ps1").read_text(encoding="utf-8")

        self.assertIn("$requiresWorkbenchProjectionAuthority", capture)
        authority = capture.split("$hasWorkbenchProjectionAuthority =", 1)[1].split(
            "$evidenceOk =", 1
        )[0]
        self.assertIn("host_invalidation_workbench_projection_target_count", authority)
        self.assertIn("host_invalidation_full_target_count", authority)
        self.assertIn("host_invalidation_legacy_dirty_transaction_count", authority)
        self.assertIn("slow_path_rebuild_count", authority)
        self.assertIn("-gt 0", authority)
        self.assertGreaterEqual(authority.count("-eq 0"), 3)


if __name__ == "__main__":
    unittest.main()
