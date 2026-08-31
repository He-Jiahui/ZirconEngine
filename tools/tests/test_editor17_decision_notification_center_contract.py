from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
DECISION = ROOT / "zircon_editor" / "src" / "core" / "notifications" / "decision"
IDENTITY = DECISION.parent / "identity"
PLAY_PENDING_DECISION = (
    ROOT / "zircon_editor" / "src" / "ui" / "host" / "play_pending_decision"
)
JOBS_PROGRESS_OBSERVER = (
    ROOT
    / "zircon_editor"
    / "src"
    / "core"
    / "jobs"
    / "system"
    / "progress_observer.rs"
)
NOTIFICATION_SERVICE = (
    ROOT / "zircon_editor" / "src" / "core" / "notifications" / "service.rs"
)


class DecisionNotificationCenterContractTests(unittest.TestCase):
    def source(self, relative: str) -> str:
        return (DECISION / relative).read_text(encoding="utf-8")

    def identity_source(self, relative: str) -> str:
        return (IDENTITY / relative).read_text(encoding="utf-8")

    def test_decision_owner_is_folder_backed_with_thin_facades(self) -> None:
        notifications = (DECISION.parent / "mod.rs").read_text(encoding="utf-8")
        facade = self.source("mod.rs")

        self.assertIn("mod decision;", notifications)
        for owner in ("center", "error", "id", "model", "receipt"):
            self.assertIn(f"mod {owner};", facade)
        self.assertNotIn("impl DecisionNotificationCenter", facade)

    def test_identity_types_are_not_reexported_through_the_decision_owner(self) -> None:
        facade = self.source("mod.rs")

        self.assertNotIn("pub(crate) use super::identity", facade)
        self.assertIn(
            "use crate::core::notifications::{NotificationId, NotificationSource};",
            self.source("model.rs"),
        )
        self.assertIn(
            "use crate::core::notifications::NotificationId;", self.source("center.rs")
        )

    def test_model_uses_typed_ids_options_and_explicit_cancel_policy(self) -> None:
        ids = self.source("id.rs")
        notification_ids = self.identity_source("id.rs")
        model = self.source("model.rs")

        for kind in (
            "DecisionOptionId",
            "DecisionTicket",
            "DecisionReceiptSequence",
        ):
            self.assertIn(f"struct {kind}", ids)
        self.assertIn("struct NotificationId", notification_ids)
        self.assertIn("use crate::core::notifications::NotificationId", ids)
        self.assertNotIn("struct NotificationId", ids)
        self.assertIn("DecisionOption", model)
        self.assertIn("default_option", model)
        self.assertIn("cancel_option", model)
        self.assertIn("title_key", model)
        self.assertIn("message_key", model)
        self.assertIn("message_arguments", model)
        self.assertIn("MAX_DECISION_MESSAGE_ARGUMENTS", model)
        self.assertIn("with_message_argument", model)

    def test_center_is_bounded_cursor_based_and_callback_free(self) -> None:
        center = self.source("center.rs")
        receipt = self.source("receipt.rs")

        self.assertIn("BTreeMap", center)
        self.assertIn("VecDeque", center)
        self.assertIn("AtomicU64", center)
        self.assertIn("checked_add", center)
        self.assertIn("pending_capacity", center)
        self.assertIn("receipt_capacity", center)
        self.assertIn("receipts_since", center)
        self.assertIn("CursorExpired", center)
        self.assertIn("resume_cursor", center)
        self.assertIn("ForeignTicket", center)
        self.assertIn("ForeignCursor", center)
        self.assertIn("newly_resolved", receipt)
        self.assertNotIn("Box<dyn Fn", center)

    def test_behavior_matrix_covers_receipts_cancel_capacity_and_cursor_gap(self) -> None:
        tests = self.source("tests.rs")

        for case in (
            "publish_and_resolve_receipt_once",
            "repeated_same_receipt_is_idempotent",
            "conflicting_second_receipt_is_rejected",
            "cancel_requires_declared_option",
            "repeated_cancel_receipt_is_idempotent",
            "stale_ticket_cannot_resolve_reused_notification_id",
            "expired_cursor_recovers_oldest_retained_receipt",
            "foreign_ticket_and_cursor_are_rejected",
            "oversized_payload_is_rejected",
            "pending_capacity_rejects_without_mutation",
            "bounded_receipt_history_reports_cursor_gap",
            "concurrent_same_option_resolves_once",
            "concurrent_conflicting_options_have_one_winner",
            "concurrent_publish_honors_pending_capacity",
            "cancel_and_resolve_are_linearized",
        ):
            self.assertIn(case, tests)

    def test_payload_is_bounded_and_snapshot_clone_is_shared(self) -> None:
        ids = self.source("id.rs")
        notification_ids = self.identity_source("id.rs")
        model = self.source("model.rs")

        self.assertIn("MAX_NOTIFICATION_ID_BYTES", notification_ids)
        self.assertIn("MAX_DECISION_OPTION_ID_BYTES", ids)
        self.assertIn("MAX_DECISION_OPTIONS", model)
        self.assertIn("MAX_LOCALIZATION_KEY_BYTES", model)
        self.assertIn("Arc<DecisionNotificationData>", model)

    def test_play_adapter_preserves_one_live_decision_until_receipt_consumption(self) -> None:
        adapter = (PLAY_PENDING_DECISION / "adapter.rs").read_text(encoding="utf-8")
        tests = "\n".join(
            path.read_text(encoding="utf-8")
            for path in sorted((PLAY_PENDING_DECISION / "tests").rglob("*.rs"))
        )

        self.assertIn("completed_receipt_tickets", adapter)
        self.assertIn("snapshot.resolved().is_none()", adapter)
        self.assertIn("concurrent_prompt_publication_keeps_one_pending_decision", tests)
        self.assertIn(
            "resolved_receipt_is_not_republished_until_the_adapter_consumes_it", tests
        )
        self.assertIn("publish_replacement_after_expiry", adapter)
        self.assertIn("republish(stale_cutoff)", adapter)
        self.assertIn("expired_recovery_cutoff", adapter)
        self.assertIn("expired_receipt_range_requires_a_new_explicit_play_choice", tests)
        self.assertIn(
            "expired_owned_receipt_without_a_pending_prompt_keeps_the_cursor_expired",
            tests,
        )
        self.assertIn(
            "foreign_only_receipt_expiry_advances_without_a_play_prompt", tests
        )
        self.assertIn(
            "live_play_receipt_after_frozen_cutoff_remains_consumable", tests
        )

    def test_job_progress_callbacks_use_one_ordered_lifecycle_dispatch(self) -> None:
        system = JOBS_PROGRESS_OBSERVER.read_text(encoding="utf-8")
        service = NOTIFICATION_SERVICE.read_text(encoding="utf-8")

        self.assertIn("ProgressObserverDispatch", system)
        self.assertIn("ProgressObserverEvent::Admitted", system)
        self.assertIn("ProgressObserverEvent::Finished", system)
        self.assertIn("ProgressObserverEvent::Resynchronize", system)
        self.assertIn("MAX_PROGRESS_OBSERVER_EVENTS", system)
        self.assertIn("catch_unwind", system)
        self.assertIn("deliver_progress_observer_events", system)
        self.assertIn(
            "observer_event_backlog_collapses_to_one_authoritative_resynchronization",
            system,
        )
        self.assertIn(
            "concurrent_promotion_cannot_deliver_finish_before_admission", service
        )
        self.assertIn(
            "observer_panic_resynchronizes_without_unwinding_job_lifecycle", service
        )


if __name__ == "__main__":
    unittest.main()
