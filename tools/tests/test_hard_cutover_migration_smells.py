import sys
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
AUDIT_SCRIPTS = (
    REPO_ROOT
    / ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts"
)
sys.path.insert(0, str(AUDIT_SCRIPTS))

from runtime_structure_audits.hard_cutover_migration_smells import (  # noqa: E402
    hard_cutover_migration_smells_audit,
)


class HardCutoverMigrationSmellsTests(unittest.TestCase):
    def test_test_suffix_files_do_not_count_as_production_migration_debt(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "zircon_runtime_interface/src/ui/focus_tests.rs"
            source.parent.mkdir(parents=True)
            source.write_text(
                "#[test]\n"
                "fn archived_navigation_contract() {\n"
                "    assert!(true, \"legacy fixture wording\");\n"
                "}\n",
                encoding="utf-8",
            )

            report = hard_cutover_migration_smells_audit(root)

        self.assertEqual(0, report["source_file_count"])
        self.assertEqual(0, report["legacy_reference_count"])
        self.assertEqual([], report["smell_decisions"])
        self.assertEqual([], report["risks"])

    def test_cfg_test_module_words_do_not_count_as_production_migration_debt(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "zircon_runtime/src/owner.rs"
            source.parent.mkdir(parents=True)
            source.write_text(
                "pub fn current_owner() {}\n\n"
                "#[cfg(test)]\n"
                "mod tests {\n"
                "    #[test]\n"
                "    fn archived_projection() {\n"
                "        assert!(true, \"legacy fixture wording\");\n"
                "    }\n"
                "}\n",
                encoding="utf-8",
            )

            report = hard_cutover_migration_smells_audit(root)

        self.assertEqual(0, report["legacy_reference_count"])
        self.assertEqual([], report["smell_decisions"])
        self.assertEqual([], report["risks"])

    def test_repository_hard_cutover_smells_are_classified_and_clear(self) -> None:
        report = hard_cutover_migration_smells_audit(REPO_ROOT)

        self.assertEqual([], report["unclassified_locations"])
        self.assertEqual([], report["hard_cutover_migration_debt"])
        self.assertEqual("classified-and-clear", report["hard_cutover_gate_status"])
        self.assertEqual([], report["risks"])


if __name__ == "__main__":
    unittest.main()
