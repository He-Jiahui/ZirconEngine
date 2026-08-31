import unittest
from pathlib import Path


class RuntimePreferenceStorageContractOwnerStructureTests(unittest.TestCase):
    def setUp(self) -> None:
        self.repo_root = Path(__file__).resolve().parents[2]
        self.owner = (
            self.repo_root
            / "zircon_runtime/src/core/framework/platform/preferences/storage.rs"
        )
        self.owner_dir = self.owner.with_suffix("")

    def test_preference_storage_contract_uses_focused_folder_backed_owners(self) -> None:
        owner_source = self.owner.read_text(encoding="utf-8")
        production_lines = [
            line
            for line in owner_source.splitlines()
            if line.strip() and not line.lstrip().startswith("//")
        ]

        self.assertLessEqual(len(production_lines), 36)
        for declaration in (
            "mod snapshot;",
            "mod storage_contract;",
            "mod terminal;",
            "mod tickets;",
            "mod work_deadline;",
        ):
            self.assertIn(declaration, owner_source)

        expected_children = {
            "snapshot.rs": (
                "pub enum PreferenceDurabilityState",
                "pub struct PreferenceEviction",
                "pub struct PreferenceReadSnapshot",
            ),
            "storage_contract.rs": ("pub trait PreferenceStorage",),
            "terminal.rs": (
                "pub struct PreferencePersistenceFailureProjection",
                "pub enum PreferenceMutationTerminal",
                "pub enum PreferenceTicketWaitResult",
                "pub enum PreferenceMutationCancelError",
            ),
            "tickets.rs": (
                "pub trait PreferenceMutationTicket",
                "pub trait PreferenceMutationCancellation",
                "pub struct PreferenceMutationSubmission",
                "pub trait PreferenceFlushTicket",
            ),
            "work_deadline.rs": ("pub struct PreferenceWorkDeadline",),
        }
        for child_name, anchors in expected_children.items():
            child = self.owner_dir / child_name
            self.assertTrue(child.is_file(), child)
            child_source = child.read_text(encoding="utf-8")
            for anchor in anchors:
                self.assertIn(anchor, child_source)

        for public_symbol in (
            "PreferenceDurabilityState",
            "PreferenceEviction",
            "PreferenceFlushTicket",
            "PreferenceMutationCancelError",
            "PreferenceMutationCancellation",
            "PreferenceMutationSubmission",
            "PreferenceMutationTerminal",
            "PreferenceMutationTicket",
            "PreferencePersistenceFailureProjection",
            "PreferenceReadSnapshot",
            "PreferenceStorage",
            "PreferenceTicketWaitResult",
            "PreferenceWorkDeadline",
        ):
            self.assertIn(public_symbol, owner_source)

        for forbidden in (
            "pub struct PreferenceWorkDeadline",
            "pub enum PreferenceDurabilityState",
            "pub struct PreferencePersistenceFailureProjection",
            "pub trait PreferenceMutationTicket",
            "pub struct PreferenceReadSnapshot",
            "pub trait PreferenceStorage",
        ):
            self.assertNotIn(forbidden, owner_source)


if __name__ == "__main__":
    unittest.main()
