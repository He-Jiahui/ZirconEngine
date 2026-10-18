import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
TASKS = ROOT / "zircon_runtime/src/core/runtime/tasks/bounded_keyed_io"
ADMISSION = TASKS / "admission.rs"
TICKET = TASKS / "ticket.rs"
LANE = TASKS / "lane.rs"
AUTH_TESTS = TASKS / "ticket/tests/astra_authority_tests.rs"


class RuntimeCancellationAuthorityContractTests(unittest.TestCase):
    def test_authority_owns_ticket_instance_not_reusable_numeric_identity(self) -> None:
        admission = ADMISSION.read_text(encoding="utf-8")
        ticket = TICKET.read_text(encoding="utf-8")

        for anchor in (
            "pub struct BoundedKeyedIoCancelAuthority",
            "ticket: BoundedKeyedIoTicket,",
            "self.ticket.clone()",
            "pub(super) fn authorizes(&self, ticket: &BoundedKeyedIoTicket)",
            "self.ticket.same_instance(ticket)",
            "pub(super) fn same_instance(&self, other: &Self) -> bool",
            "Arc::ptr_eq(&self.state, &other.state)",
        ):
            self.assertIn(anchor, admission + ticket)

        self.assertNotIn("self.ticket.id == ticket.id", admission)
        self.assertNotIn("self.ticket.generation == ticket.generation", admission)

    def test_cancel_checks_authority_before_mutating_ticket_and_preserves_terminal_guards(self) -> None:
        source = TICKET.read_text(encoding="utf-8")
        cancel = "pub fn cancel_before_start" + source.split("pub fn cancel_before_start", 1)[1].split("pub(crate) fn mark_started", 1)[0]

        for anchor in (
            "pub fn cancel_before_start(",
            "if !authority.authorizes(self)",
            "BoundedKeyedIoCancelError::WrongAuthority",
            "CancelledBeforeStart",
            "BoundedKeyedIoCancelError::AlreadyStarted",
            "if state.fence_pins != 0",
            "BoundedKeyedIoCancelError::FencePinned",
            "if state.started",
            "self.state.changed.notify_all();",
        ):
            self.assertIn(anchor, cancel)

        authority_check = cancel.index("if !authority.authorizes(self)")
        state_lock = cancel.index("let mut state = self.lock()")
        self.assertLess(authority_check, state_lock)

    def test_lane_admission_binds_the_authority_to_the_new_ticket(self) -> None:
        source = LANE.read_text(encoding="utf-8")
        admission = ADMISSION.read_text(encoding="utf-8")

        for anchor in (
            "let ticket = BoundedKeyedIoTicket::pending(id, generation, false);",
            "cancel_authority: BoundedKeyedIoCancelAuthority::new(&ticket),",
            "Ok(BoundedKeyedIoAdmission {",
        ):
            self.assertIn(anchor, source)
        for anchor in (
            "pub fn cancel_authority(&self)",
            "self.cancel_authority.clone()",
        ):
            self.assertIn(anchor, admission)

    def test_authority_regressions_cover_collisions_reuse_and_fence_start_edges(self) -> None:
        source = AUTH_TESTS.read_text(encoding="utf-8")

        for test_name in (
            "astra_m25_foreign_lane_authority_cannot_cancel_a_colliding_ticket",
            "astra_m25_reused_numeric_identity_rejects_old_authority",
            "astra_m25_owner_checks_preserve_fence_and_start_guards",
        ):
            self.assertIn(f"fn {test_name}(", source)
        for anchor in (
            "Err(BoundedKeyedIoCancelError::WrongAuthority)",
            "Err(BoundedKeyedIoCancelError::FencePinned)",
            "Err(BoundedKeyedIoCancelError::AlreadyStarted)",
            "assert_eq!(source_ticket.id(), target_ticket.id());",
            "assert_eq!(source_ticket.generation(), target_ticket.generation());",
        ):
            self.assertIn(anchor, source)


if __name__ == "__main__":
    unittest.main()
