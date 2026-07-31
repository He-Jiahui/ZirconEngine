import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
TOOLS_DIR = ROOT / "zircon_editor" / "src" / "core" / "tools"


class Editor08ToolSchedulerContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.mod_source = (TOOLS_DIR / "mod.rs").read_text(encoding="utf-8")
        cls.scheduler_source = (TOOLS_DIR / "scheduler.rs").read_text(encoding="utf-8")
        cls.tool_id_source = (TOOLS_DIR / "tool_id.rs").read_text(encoding="utf-8")
        cls.tests_source = (TOOLS_DIR / "tests.rs").read_text(encoding="utf-8")

    def test_folder_owner_exports_typed_scheduler_contract(self) -> None:
        for declaration in (
            "mod scheduler;",
            "mod tool_id;",
            "pub use scheduler::{",
            "pub use tool_id::{",
            "ToolId",
            "ToolIdError",
            "MAX_TOOL_ID_BYTES",
        ):
            self.assertIn(declaration, self.mod_source)

    def test_tool_ids_are_validated_instead_of_using_raw_strings(self) -> None:
        for contract in (
            "pub struct ToolId",
            "pub enum ToolIdError",
            "pub fn parse",
            "MAX_TOOL_ID_BYTES",
            "Empty",
            "InvalidCharacter",
            "TooLong",
        ):
            self.assertIn(contract, self.tool_id_source)

    def test_scheduler_uses_bounded_fifo_per_exclusive_resource(self) -> None:
        for contract in (
            "pub enum ExclusiveResource",
            "ViewportInput",
            "ModalSurface",
            "SceneModeSlot",
            "VecDeque<ToolId>",
            "max_queue_per_resource",
            "QueueFull",
            ".push_back(",
            ".pop_front()",
        ):
            self.assertIn(contract, self.scheduler_source)

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
            "pub fn release_all(",
            "pub fn events(&self)",
        ):
            self.assertIn(contract, self.scheduler_source)

    def test_behavior_contract_covers_fifo_idempotence_bounds_and_shutdown(self) -> None:
        for test_name in (
            "acquire_is_idempotent_for_the_current_holder",
            "contended_tools_activate_in_fifo_order",
            "duplicate_queued_acquire_does_not_grow_the_queue",
            "full_queue_returns_typed_denial_without_mutation",
            "withdraw_removes_only_the_callers_pending_request",
            "release_all_clears_owned_and_queued_resources",
            "lifecycle_events_preserve_release_then_activation_order",
            "non_owner_release_and_missing_withdraw_are_side_effect_free",
        ):
            self.assertIn(f"fn {test_name}", self.tests_source)

    def test_production_scheduler_avoids_panic_paths(self) -> None:
        production = self.scheduler_source + self.tool_id_source
        for forbidden in (".unwrap()", ".expect(", "panic!(", "unreachable!("):
            self.assertNotIn(forbidden, production)


if __name__ == "__main__":
    unittest.main()
