from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
PLAY = ROOT / "zircon_editor" / "src" / "core" / "play"


class PlayEditProtectionContractTests(unittest.TestCase):
    def source(self, relative: str) -> str:
        return (PLAY / relative).read_text(encoding="utf-8")

    def test_policy_is_folder_backed_and_covers_the_three_playing_tiers(self) -> None:
        facade = self.source("edit_policy/mod.rs")
        policy = self.source("edit_policy/policy.rs")
        decision = self.source("edit_policy/decision.rs")
        target = (
            ROOT
            / "zircon_editor"
            / "src"
            / "core"
            / "editing"
            / "operation"
            / "edit_target.rs"
        ).read_text(encoding="utf-8")

        for owner in ("decision", "policy"):
            self.assertIn(f"mod {owner};", facade)
        self.assertNotIn("mod target;", facade)
        self.assertIn("EditOperationTarget", target)
        self.assertIn("PlayDomain", target)
        self.assertIn("EditDocument", target)
        self.assertIn("EditWorkspace", target)
        self.assertIn("ApplyNow", decision)
        self.assertIn("RunningDocumentLocked", decision)
        self.assertIn("QueueUntilPlayStops", decision)
        self.assertIn("running_document", policy)

    def test_pending_queue_owns_shared_payloads_bounded_retention_and_retry_reports(self) -> None:
        intent = self.source("pending_edits/intent.rs")
        queue = self.source("pending_edits/queue.rs")
        resolution = self.source("pending_edits/resolution.rs")
        retention = (
            ROOT
            / "zircon_editor"
            / "src"
            / "core"
            / "editing"
            / "operation"
            / "pending_edit_retention.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("EditorOperationInvocation", intent)
        self.assertIn("Arc<EditorOperationInvocation>", intent)
        self.assertIn("PendingEditId", intent)
        self.assertIn("PendingEditRetention", intent)
        self.assertIn("belongs_to_cohort", intent)
        self.assertIn("cohort_kind", intent)
        self.assertIn("NonZeroUsize", retention)
        self.assertIn("pub fn bounded", retention)
        self.assertIn("PendingEditCohortKind", retention)
        self.assertIn("bounded_limits", queue)
        self.assertIn("checked_add", queue)
        self.assertIn("PendingEditQueueLimits", queue)
        self.assertIn("max_payload_bytes", queue)
        self.assertIn("max_oldest_age", queue)
        self.assertIn("PendingEditPage", queue)
        self.assertIn("apply_with_budget", queue)
        self.assertIn("in_flight_count", queue)
        self.assertIn("catch_unwind", queue)
        self.assertIn("requeue_for_retry", queue)
        self.assertIn("pub fn discard", queue)
        self.assertNotIn("pub fn snapshot", queue)
        self.assertNotIn("apply_all", queue)
        self.assertNotIn("discard_all", queue)
        self.assertIn("PendingEditApplyFailure", resolution)
        self.assertIn("intent", resolution)

    def test_controller_gates_before_backend_and_prompts_after_exit(self) -> None:
        controller = self.source("controller.rs")
        protection = self.source("edit_protection.rs")
        request = self.source("request.rs")
        report = self.source("transition_report.rs")

        gate = controller.index("edit_protection.begin_play")
        backend_start = controller.index("backend.start(")
        self.assertLess(gate, backend_start)
        self.assertIn("edit_protection.end_play", controller)
        self.assertIn("route_edit", controller)
        self.assertIn("DeferredOperationInvocation", controller)
        self.assertIn(".route(target, deferred)", controller)
        self.assertIn("apply_pending_edits", controller)
        self.assertIn("discard_pending_edits", controller)
        self.assertIn("pending_edits_summary", controller)
        self.assertNotIn("pending_edits_snapshot", controller)
        self.assertNotIn("pub fn edit_protection", controller)
        self.assertIn("resolving", protection)
        self.assertIn("evicted_ids", protection)
        self.assertNotIn("pub fn pending_edits(&self)", protection)
        self.assertIn("with_running_document", request)
        self.assertIn("pending_edit_prompt", report)

    def test_behavior_tests_cover_lock_queue_apply_discard_and_start_guard(self) -> None:
        tests = "\n".join(
            (
                self.source("edit_policy/tests.rs"),
                self.source("pending_edits/tests.rs"),
            )
        )
        for case in (
            "playing_policy_applies_locks_and_queues_by_target",
            "latest_retention_coalesces_only_the_same_target_and_operation",
            "bounded_policy_rejects_zero_limits_before_play_routing",
            "bounded_retention_evicts_only_its_typed_cohort",
            "lossless_admission_respects_global_limits_without_dropping_another_intent",
            "bounded_retention_rejects_new_work_after_its_own_age_limit",
            "lossless_retention_preserves_fifo_order_and_retry_authority",
            "budgeted_apply_leaves_unattempted_intents_for_the_next_resolution_turn",
            "queued_route_surfaces_declared_bounded_evictions_to_its_caller",
            "pending_decision_blocks_the_next_play_start_until_queue_resolution",
            "playing_cannot_resolve_pending_edits",
            "resolution_in_progress_blocks_play_start",
        ):
            self.assertIn(case, tests)


if __name__ == "__main__":
    unittest.main()
