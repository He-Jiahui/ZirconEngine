import unittest
from pathlib import Path


class Runtime15ArchiveGuardHardCutTests(unittest.TestCase):
    def test_runtime15_archive_guard_cohort_does_not_compile_historical_output_receipts(
        self,
    ) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        structure_guard_root = (
            repo_root
            / "zircon_runtime/src/tests/runtime_absorption/structure_convention"
        )
        guard_paths = (
            "animation_manager.rs",
            "facade_surface.rs",
            "runtime_dead_code/guard_layout.rs",
            "runtime_dead_code/production_scan.rs",
            "runtime_dead_code/runtime_owned.rs",
            "runtime_dead_code/runtime_ui.rs",
            "runtime_dead_code/script_host.rs",
            "runtime_dead_code/ui_text.rs",
        )
        retired_archive_prefix = (
            "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-"
        )

        stale_guards = []
        for relative_path in guard_paths:
            guard_path = structure_guard_root / relative_path
            self.assertTrue(guard_path.is_file(), guard_path.as_posix())
            source = guard_path.read_text(encoding="utf-8")
            if retired_archive_prefix in source:
                stale_guards.append(guard_path.relative_to(repo_root).as_posix())

        self.assertEqual([], stale_guards)


if __name__ == "__main__":
    unittest.main()
