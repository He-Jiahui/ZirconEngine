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
        target = self.source("edit_policy/target.rs")
        decision = self.source("edit_policy/decision.rs")

        for owner in ("decision", "policy", "target"):
            self.assertIn(f"mod {owner};", facade)
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

        self.assertIn("EditorOperationInvocation", intent)
        self.assertIn("Arc<EditorOperationInvocation>", intent)
        self.assertIn("PendingEditId", intent)
        self.assertIn("PendingEditRetention", intent)
        self.assertIn("EmptyCohortKey", intent)
        self.assertIn("fn validate", intent)
        self.assertIn(".validate()", queue)
        self.assertIn("checked_add", queue)
        self.assertIn("PendingEditQueueLimits", queue)
        self.assertIn("max_payload_bytes", queue)
        self.assertIn("max_oldest_age", queue)
        self.assertIn("PendingEditPage", queue)
        self.assertIn("apply_with_budget", queue)
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
        self.assertIn("retention: PendingEditRetention", controller)
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
            "latest_retention_replaces_the_payload_without_duplicating_retry_authority",
            "invalid_retention_is_rejected_at_the_queue_boundary",
            "bounded_retention_reports_only_declared_cohort_evictions",
            "lossless_admission_rejects_global_limit_without_dropping_another_intent",
            "oldest_age_limit_rejects_new_intents_without_discarding_retained_work",
            "failed_apply_requeues_the_same_intent_for_a_later_budgeted_retry",
            "budgeted_apply_leaves_unattempted_intents_for_the_next_resolution_turn",
            "queued_route_surfaces_declared_bounded_evictions_to_its_caller",
            "pending_decision_blocks_the_next_play_start",
            "playing_cannot_resolve_pending_edits",
            "resolution_in_progress_blocks_play_start",
        ):
            self.assertIn(case, tests)


if __name__ == "__main__":
    unittest.main()
