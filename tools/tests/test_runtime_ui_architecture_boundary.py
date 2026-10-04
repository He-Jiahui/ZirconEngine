import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch


REPO_ROOT = Path(__file__).resolve().parents[2]
AUDIT_SCRIPTS = (
    REPO_ROOT
    / ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts"
)
sys.path.insert(0, str(AUDIT_SCRIPTS))

from runtime_structure_audits.ui_architecture_boundary import (  # noqa: E402
    LEGACY_MIGRATION_TERMS,
    _files_with_matching_term,
    _matching_term_line_count,
    ui_architecture_boundary_audit,
)
from runtime_structure_audits import ui_architecture_boundary as audit_module  # noqa: E402


class RuntimeUiArchitectureBoundaryTests(unittest.TestCase):
    def test_source_inventory_rejects_removed_required_entry(self) -> None:
        with patch.object(audit_module, "SOURCE_FILES", audit_module.SOURCE_FILES[:-1]):
            report = ui_architecture_boundary_audit(REPO_ROOT)

        self.assertIn(
            "Runtime 09 UI architecture source inventory count changed without audit sync.",
            report["risks"],
        )

    def test_document_mentions_cannot_replace_rust_test_declarations(self) -> None:
        original_read_text = audit_module._read_text
        for guard_name in (
            "runtime_09_ui_architecture_baselines_match_current_source_scan",
            "runtime_09_navigation_legacy_reply_rename_reduces_ui_input_debt",
            "runtime_09_ui_architecture_mirror_docs_match_structure_audit_counts",
        ):
            with self.subTest(guard=guard_name):
                mutated_paths = []

                def read_without_guard(path: Path) -> str:
                    source = original_read_text(path)
                    declaration = f"fn {guard_name}("
                    if path.suffix == ".rs" and declaration in source:
                        mutated_paths.append(path)
                        return (
                            source.replace(declaration, f"fn removed_{guard_name}(")
                            + f"\n/* #[test]\nfn {guard_name}() {{}} */\n"
                            + f'const OLD_TEST: &str = r"#[test]\nfn {guard_name}() {{}}";\n'
                        )
                    return source

                with patch.object(audit_module, "_read_text", read_without_guard):
                    report = ui_architecture_boundary_audit(REPO_ROOT)

                self.assertTrue(mutated_paths, "fixture must remove an existing Rust test")
                self.assertIn(guard_name, report["missing_guard_anchors"])
                self.assertTrue(report["risks"])
                if guard_name == audit_module.MIRROR_DOCS_GUARD:
                    self.assertFalse(report["mirror_docs_guard_present"])

    def test_current_text_surface_leaves_and_index_route_are_mirrored(self) -> None:
        report = ui_architecture_boundary_audit(REPO_ROOT)

        self.assertEqual(49, report["expected_source_file_count"])
        self.assertEqual(49, len(report["source_files"]))
        self.assertEqual([], report["missing_source_files"])
        self.assertEqual(23, report["expected_ui_entry_count"])
        self.assertEqual([], report["ui_missing_entries"])
        self.assertEqual([], report["ui_unexpected_entries"])
        self.assertEqual(45, report["expected_surface_entry_count"])
        self.assertEqual([], report["surface_missing_entries"])
        self.assertEqual([], report["surface_unexpected_entries"])
        self.assertEqual(15, report["legacy_full_hits"])
        self.assertEqual(0, report["legacy_production_hits"])
        self.assertEqual([], report["legacy_production_files"])
        self.assertEqual(254, report["taffy_production_hits"])
        self.assertEqual(16, len(report["taffy_production_files"]))
        self.assertEqual([], report["missing_required_doc_mentions"])
        self.assertEqual([], report["risks"])

    def test_legacy_metric_tracks_retired_migration_terms_not_benchmark_labels(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            root = Path(temporary_directory)
            benchmark = root / "benchmark.rs"
            benchmark.write_text(
                "let legacy_samples = samples();\n"
                'assert!(legacy_p95_ns > optimized_p95_ns, "legacy comparator");\n'
                "let legacy_properties = comparator_inputs();\n"
                'println!("legacy_property_resolutions_per_rename=3");\n',
                encoding="utf-8",
            )
            retired = root / "retired.rs"
            retired.write_text(
                "let legacy_visible = matches!(backend, LegacyZircon);\n"
                "let backend_name = legacy_zircon;\n",
                encoding="utf-8",
            )
            files = [benchmark, retired]

            self.assertEqual(
                2,
                _matching_term_line_count(files, LEGACY_MIGRATION_TERMS),
            )
            self.assertEqual(
                ["retired.rs"],
                _files_with_matching_term(root, files, LEGACY_MIGRATION_TERMS),
            )


if __name__ == "__main__":
    unittest.main()
