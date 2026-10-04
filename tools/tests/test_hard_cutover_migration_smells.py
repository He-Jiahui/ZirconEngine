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
from runtime_structure_audits.module_convention_gate import (  # noqa: E402
    _split_render_scoped_migration_debt,
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
        self.assertEqual("classified-and-clear", report["hard_cutover_gate_status"])

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

    def test_cfg_test_module_string_braces_do_not_end_the_skipped_item_early(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "zircon_runtime/src/owner.rs"
            source.parent.mkdir(parents=True)
            source.write_text(
                "pub fn current_owner() {}\n\n"
                "#[cfg(test)]\n"
                "mod tests {\n"
                "    const CLOSING_BRACE: &str = \"}\";\n"
                "    const FIXTURE: &str = \"legacy fixture wording\";\n"
                "}\n",
                encoding="utf-8",
            )

            report = hard_cutover_migration_smells_audit(root)

        self.assertEqual(1, report["source_file_count"])
        self.assertEqual(0, report["legacy_reference_count"])
        self.assertEqual([], report["smell_decisions"])
        self.assertEqual([], report["risks"])

    def test_current_owner_debts_receive_specific_classifications(self) -> None:
        expected = {
            "zircon_app/src/entry/engine_entry.rs": (
                "legacy-runtime-module-composition-debt"
            ),
            "zircon_editor/src/ui/host/editor_manager.rs": (
                "legacy-editor-host-service-debt"
            ),
            "zircon_runtime/src/builtin/runtime_modules/assembly.rs": (
                "legacy-runtime-module-composition-debt"
            ),
            "zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/project/selection.rs": (
                "legacy-runtime-plugin-selection-admission-debt"
            ),
            "zircon_runtime/src/render_graph/access/resource_access_intent.rs": (
                "legacy-runtime-render-graph-access-debt"
            ),
            "zircon_runtime/src/text/document/storage.rs": (
                "legacy-runtime-text-compatibility-debt"
            ),
            "zircon_runtime/src/ui/component/state_reducer/state_model.rs": (
                "legacy-runtime-ui-event-dto-debt"
            ),
            "zircon_runtime/src/ui/surface/surface/default_interactions/table/width_mutation.rs": (
                "legacy-runtime-ui-table-update-route-debt"
            ),
            "zircon_runtime_interface/src/reflect/schema_catalog/mod.rs": (
                "legacy-runtime-reflect-schema-import-debt"
            ),
        }
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for relative_path in expected:
                source = root / relative_path
                source.parent.mkdir(parents=True, exist_ok=True)
                source.write_text(
                    'pub const CURRENT_OWNER: &str = "legacy";\n',
                    encoding="utf-8",
                )

            report = hard_cutover_migration_smells_audit(root)

        actual = {
            decision["path"]: decision["classification"]
            for decision in report["smell_decisions"]
        }
        self.assertEqual(expected, actual)
        self.assertEqual([], report["unclassified_locations"])

    def test_render_graph_access_debt_stays_in_the_render_handoff(self) -> None:
        render_debt, non_render_debt = _split_render_scoped_migration_debt(
            [
                "hard-cutover: legacy-runtime-render-graph-access-debt: remove the "
                "compatibility access intent (14 location(s))",
                "hard-cutover: legacy-runtime-text-compatibility-debt: replace the "
                "retained source contract (6 location(s))",
            ]
        )

        self.assertEqual(
            [
                "hard-cutover: legacy-runtime-render-graph-access-debt: remove the "
                "compatibility access intent (14 location(s))"
            ],
            render_debt,
        )
        self.assertEqual(
            [
                "hard-cutover: legacy-runtime-text-compatibility-debt: replace the "
                "retained source contract (6 location(s))"
            ],
            non_render_debt,
        )

    def test_repository_hard_cutover_smells_are_classified_without_hard_blockers(
        self,
    ) -> None:
        report = hard_cutover_migration_smells_audit(REPO_ROOT)

        self.assertEqual([], report["unclassified_locations"])
        self.assertEqual(0, report["compat_reference_count"])
        self.assertEqual(0, report["shim_reference_count"])
        self.assertEqual(0, report["migration_bridge_smell_count"])
        self.assertNotIn(
            "unclassified-hard-cutover-smell",
            report["classification_counts"],
        )


if __name__ == "__main__":
    unittest.main()
