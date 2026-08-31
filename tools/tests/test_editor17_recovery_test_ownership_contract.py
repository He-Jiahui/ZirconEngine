from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
RECOVERY = ROOT / "zircon_editor" / "src" / "core" / "recovery"


class RecoveryTestOwnershipContractTests(unittest.TestCase):
    def autosave_adapter_source(self) -> str:
        owner = RECOVERY / "tests" / "autosave_adapter"
        self.assertTrue(owner.is_dir())
        self.assertFalse((RECOVERY / "tests" / "autosave_adapter.rs").exists())
        return "\n".join(
            path.read_text(encoding="utf-8") for path in sorted(owner.glob("*.rs"))
        )

    def test_autosave_adapter_tests_have_a_named_child_owner(self) -> None:
        root = (RECOVERY / "tests.rs").read_text(encoding="utf-8")
        adapter = self.autosave_adapter_source()

        self.assertIn("mod autosave_adapter;", root)
        for test_name in (
            "autosave_adapter_defers_snapshot_capture_until_the_admitted_mutex_turn",
            "autosave_adapter_releases_single_flight_when_atomic_admission_is_rejected",
            "autosave_adapter_advances_after_a_write_failure_and_shutdown_rejects_new_work",
        ):
            self.assertIn(f"fn {test_name}", adapter)
            self.assertNotIn(f"fn {test_name}", root)

    def test_adapter_support_types_live_with_their_tests(self) -> None:
        root = (RECOVERY / "tests.rs").read_text(encoding="utf-8")
        adapter = self.autosave_adapter_source()

        for symbol in (
            "fn wait_for_autosave_completion",
            "struct GateJob",
            "struct CountingSnapshotSource",
        ):
            self.assertIn(symbol, adapter)
            self.assertNotIn(symbol, root)

    def test_session_guard_tests_have_a_named_child_owner(self) -> None:
        root = (RECOVERY / "tests.rs").read_text(encoding="utf-8")
        owner = (RECOVERY / "tests" / "session_guard.rs").read_text(encoding="utf-8")

        self.assertIn("mod session_guard;", root)
        for test_name in (
            "session_guard_persists_heartbeat_and_requires_explicit_residual_takeover",
            "concurrent_residual_takeover_keeps_exactly_one_live_guard",
            "live_guard_rejects_takeover_before_heartbeat_and_release",
        ):
            self.assertIn(f"fn {test_name}", owner)
            self.assertNotIn(f"fn {test_name}", root)

    def test_test_owner_files_stay_under_the_structure_review_threshold(self) -> None:
        owners = [RECOVERY / "tests.rs", RECOVERY / "tests" / "session_guard.rs"]
        owners.extend(sorted((RECOVERY / "tests" / "autosave_adapter").glob("*.rs")))
        for owner in owners:
            with self.subTest(owner=owner.relative_to(RECOVERY)):
                self.assertLessEqual(
                    len(owner.read_text(encoding="utf-8").splitlines()),
                    800,
                )


if __name__ == "__main__":
    unittest.main()
