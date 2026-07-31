from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
DECISION = ROOT / "zircon_editor" / "src" / "core" / "notifications" / "decision"


class DecisionNotificationCenterContractTests(unittest.TestCase):
    def source(self, relative: str) -> str:
        return (DECISION / relative).read_text(encoding="utf-8")

    def test_decision_owner_is_folder_backed_with_thin_facades(self) -> None:
        notifications = (DECISION.parent / "mod.rs").read_text(encoding="utf-8")
        facade = self.source("mod.rs")

        self.assertIn("mod decision;", notifications)
        for owner in ("center", "error", "id", "model", "receipt"):
            self.assertIn(f"mod {owner};", facade)
        self.assertNotIn("impl DecisionNotificationCenter", facade)

    def test_model_uses_typed_ids_options_and_explicit_cancel_policy(self) -> None:
        ids = self.source("id.rs")
        model = self.source("model.rs")

        for kind in (
            "NotificationId",
            "DecisionOptionId",
            "DecisionTicket",
            "DecisionReceiptSequence",
        ):
            self.assertIn(f"struct {kind}", ids)
        self.assertIn("DecisionOption", model)
        self.assertIn("default_option", model)
        self.assertIn("cancel_option", model)
        self.assertIn("title_key", model)
        self.assertIn("message_key", model)

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
        model = self.source("model.rs")

        self.assertIn("MAX_NOTIFICATION_ID_BYTES", ids)
        self.assertIn("MAX_DECISION_OPTION_ID_BYTES", ids)
        self.assertIn("MAX_DECISION_OPTIONS", model)
        self.assertIn("MAX_LOCALIZATION_KEY_BYTES", model)
        self.assertIn("Arc<DecisionNotificationData>", model)


if __name__ == "__main__":
    unittest.main()
