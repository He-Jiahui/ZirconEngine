import unittest
from pathlib import Path


class RuntimeNativeRegistrationReplayErrorOwnerStructureTests(unittest.TestCase):
    def test_registration_replay_error_is_child_owned(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        owner_path = (
            repo_root
            / "zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/registration_replay.rs"
        )
        error_path = (
            repo_root
            / "zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/registration_replay/error.rs"
        )

        owner = owner_path.read_text(encoding="utf-8")
        self.assertLessEqual(len(owner.splitlines()), 800)

        error = error_path.read_text(encoding="utf-8")
        self.assertLessEqual(len(error.splitlines()), 800)
        self.assertIn("mod error;", owner)
        self.assertIn(
            "pub(super) use error::NativePluginRegistrationReplayError;", owner
        )
        self.assertNotIn("enum NativePluginRegistrationReplayError", owner)
        for anchor in (
            "pub(in super::super) enum NativePluginRegistrationReplayError",
            "impl std::fmt::Display for NativePluginRegistrationReplayError",
            "impl std::error::Error for NativePluginRegistrationReplayError",
            "InvalidRegistrationManifest",
            "InvalidSystemAccessAuthority",
            "RegisterNativeSystem",
        ):
            self.assertIn(anchor, error)

        self.assertIn("type NativePluginRegistrationReplayResult<T>", owner)
        self.assertEqual(owner.count(".sort_unstable();"), 3)
        self.assertIn(
            '#[path = "registration_replay/optimization_tests.rs"]', owner
        )

    def test_typed_error_review_guard_reads_error_child(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        guard = (
            repo_root
            / "zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/live_host/replay_and_runtime/registration_replay.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("registration_replay/error.rs", guard)
        self.assertIn("registration_replay_error", guard)

    def test_plan_and_boundary_docs_record_error_owner(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        child_path = "registration_replay/error.rs"
        status = (
            "runtime_06_15_native_registration_replay_error_owner_split_"
            "static_passed_cargo_deferred"
        )
        docs = (
            "docs/plans/engine-code-structure-convention.md",
            "docs/plans/engine-code-review-findings-2026-06.md",
            "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
            "docs/plans/zircon_runtime/runtime/06/2026-07-09-plugin-surface-and-lifecycle-output-records.md",
            "docs/engine-architecture/native-plugin-boundary.md",
        )

        for relative_path in docs:
            source = (repo_root / relative_path).read_text(encoding="utf-8")
            self.assertIn(child_path, source, relative_path)
            self.assertIn(status, source, relative_path)


if __name__ == "__main__":
    unittest.main()
