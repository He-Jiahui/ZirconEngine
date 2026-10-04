import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
TOOLS_DIR = ROOT / "zircon_editor" / "src" / "core" / "tools"


class Editor08ToolSchedulerContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.mod_source = (TOOLS_DIR / "mod.rs").read_text(encoding="utf-8")
        cls.scheduler_source = (TOOLS_DIR / "scheduler.rs").read_text(encoding="utf-8")
        cls.identity_source = (TOOLS_DIR / "identity.rs").read_text(encoding="utf-8")
        cls.claim_source = (TOOLS_DIR / "claim.rs").read_text(encoding="utf-8")
        cls.tests_source = (TOOLS_DIR / "tests.rs").read_text(encoding="utf-8")

    def test_folder_owner_exports_typed_scheduler_contract(self) -> None:
        for declaration in (
            "mod identity;",
            "mod scheduler;",
            "pub(crate) use scheduler::ToolScheduler;",
            "pub use identity::{",
            "ToolDefinitionId",
            "ToolDefinitionIdError",
            "MAX_TOOL_DEFINITION_ID_BYTES",
        ):
            self.assertIn(declaration, self.mod_source)

    def test_tool_ids_are_validated_instead_of_using_raw_strings(self) -> None:
        for contract in (
            "pub struct ToolDefinitionId",
            "pub enum ToolDefinitionIdError",
            "pub fn parse",
            "MAX_TOOL_DEFINITION_ID_BYTES",
            "Empty",
            "InvalidCharacter",
            "TooLong",
        ):
            self.assertIn(contract, self.identity_source)

    def test_scheduler_uses_bounded_fifo_per_exclusive_resource(self) -> None:
        for contract in (
            "ToolResourceKey",
            "ToolResourceSet",
            "VecDeque<ToolId>",
            "ToolQueueLimits",
            "max_single_queue_per_resource",
            "max_set_queue",
            "QueueFull",
            ".push_back(",
            ".pop_front()",
        ):
            self.assertIn(
                contract.replace("VecDeque<ToolId>", "VecDeque<ToolRequestId>"),
                self.scheduler_source,
            )

    def test_acquire_release_and_cleanup_have_typed_outcomes_and_events(self) -> None:
        for contract in (
            "pub enum AcquireOutcome",
            "pub enum ReleaseOutcome",
            "pub enum WithdrawOutcome",
            "pub enum ToolLifecycleEvent",
            "pub struct ToolScheduleReport",
            "pub fn acquire(",
            "pub fn release(",
            "pub fn withdraw(",
            "pub(crate) fn shutdown(",
            "pub fn into_parts(self)",
        ):
            source = self.scheduler_source + self.identity_source + self.claim_source
            self.assertIn(contract, source)

    def test_behavior_contract_covers_fifo_idempotence_bounds_and_shutdown(self) -> None:
        for test_name in (
            "one_instance_cannot_replace_its_active_claim",
            "resource_set_requests_activate_atomically_and_preserve_fifo",
            "repeated_same_claim_returns_the_canonical_handle",
            "queue_full_denial_identifies_the_canonical_holder_lease",
            "withdraw_removes_only_the_exact_request_id",
            "shutdown_releases_and_withdraws_every_claim_without_promotion",
            "lifecycle_events_preserve_deactivation_then_activation_order",
            "stale_lease_cannot_release_a_newer_claim",
        ):
            self.assertIn(f"fn {test_name}", self.tests_source)

    def test_production_scheduler_avoids_panic_paths(self) -> None:
        production = self.scheduler_source + self.identity_source + self.claim_source
        for forbidden in (".unwrap()", ".expect(", "panic!(", "unreachable!("):
            self.assertNotIn(forbidden, production)


if __name__ == "__main__":
    unittest.main()
