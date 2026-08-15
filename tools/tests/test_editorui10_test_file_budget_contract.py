from __future__ import annotations

from pathlib import Path
import sys
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[2]
AUDIT_ROOT = (
    ROOT
    / ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts"
)
sys.path.insert(0, str(AUDIT_ROOT))

from editor_structure_audits.module_convention_boundary import (  # noqa: E402
    editor_module_convention_audit,
)


def write_rust(path: Path, line_count: int) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text("\n".join("// test" for _ in range(line_count)), encoding="utf-8")


def write_rust_with_test_attribute(path: Path, line_count: int) -> None:
    write_rust(path, line_count - 2)
    with path.open("a", encoding="utf-8") as output:
        output.write("\n#[test]\nfn contract_fixture() {}\n")


def write_rust_with_cfg_test_attribute(path: Path, line_count: int) -> None:
    write_rust(path, line_count - 2)
    with path.open("a", encoding="utf-8") as output:
        output.write("\n#[cfg(test)]\nmod contract_fixture {}\n")


class EditorUi10TestFileBudgetContractTests(unittest.TestCase):
    def test_audit_reports_budget_owners_by_functional_test_domain(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_rust(
                root / "zircon_editor/src/tests/host/retained_window/over_budget.rs",
                801,
            )
            write_rust(
                root / "zircon_editor/src/tests/ui/boundary/over_budget.rs",
                801,
            )
            write_rust(root / "zircon_editor/src/tests/at_budget.rs", 800)
            write_rust(root / "zircon_editor/src/tests/fixture.rs", 801)

            audit = editor_module_convention_audit(
                root,
                test_file_budget_exemptions={
                    "zircon_editor/src/tests/fixture.rs": "fixture payload"
                },
            ).to_json()

        self.assertEqual(audit["oversized_test_file_count"], 2)
        self.assertEqual(
            audit["oversized_test_files"],
            [
                {
                    "path": "zircon_editor/src/tests/host/retained_window/over_budget.rs",
                    "lines": 801,
                    "owner_class": "editor-retained-host-tests",
                },
                {
                    "path": "zircon_editor/src/tests/ui/boundary/over_budget.rs",
                    "lines": 801,
                    "owner_class": "editor-ui-tests",
                }
            ],
        )
        self.assertEqual(
            audit["oversized_test_file_exemptions"],
            [
                {
                    "path": "zircon_editor/src/tests/fixture.rs",
                    "lines": 801,
                    "reason": "fixture payload",
                }
            ],
        )

    def test_named_production_module_is_not_misclassified_as_a_test_owner(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_rust(root / "zircon_editor/src/ui/example/tests.rs", 1001)

            audit = editor_module_convention_audit(root).to_json()

        self.assertEqual(audit["oversized_test_file_count"], 0)
        self.assertEqual(
            audit["oversized_production_files"],
            [
                {
                    "path": "zircon_editor/src/ui/example/tests.rs",
                    "lines": 1001,
                    "owner_class": "editor-ui",
                }
            ],
        )

    def test_inline_test_attributes_do_not_reclassify_a_production_owner(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            production = root / "zircon_editor/src/ui/example/view_projection.rs"
            write_rust(production, 1000)
            with production.open("a", encoding="utf-8") as output:
                output.write("\n#[cfg(test)]\nmod tests {}\n")
            write_rust_with_test_attribute(
                root / "zircon_editor/src/ui/example/tests.rs",
                801,
            )

            audit = editor_module_convention_audit(root).to_json()

        self.assertEqual(
            audit["oversized_test_files"],
            [
                {
                    "path": "zircon_editor/src/ui/example/tests.rs",
                    "lines": 801,
                    "owner_class": "editor-ui-tests",
                }
            ],
        )
        self.assertEqual(
            audit["oversized_production_files"],
            [
                {
                    "path": "zircon_editor/src/ui/example/view_projection.rs",
                    "lines": 1002,
                    "owner_class": "editor-ui",
                }
            ],
        )

    def test_test_module_filenames_require_real_line_attributes(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_rust(root / "zircon_editor/src/ui/example/tests.rs", 801)
            write_rust_with_test_attribute(
                root / "zircon_editor/src/ui/example/behavior_tests.rs",
                801,
            )
            write_rust_with_cfg_test_attribute(
                root / "zircon_editor/src/ui/example/cfg_tests.rs",
                801,
            )
            documented = root / "zircon_editor/src/ui/example/documented_tests.rs"
            write_rust(documented, 1000)
            with documented.open("a", encoding="utf-8") as output:
                output.write("\n// #[test]\nconst EXAMPLE: &str = \"#[cfg(test)]\";\n")
            fixture = root / "zircon_editor/src/ui/example/fixture_tests.rs"
            write_rust(fixture, 1000)
            with fixture.open("a", encoding="utf-8") as output:
                output.write("\n#[test::fixture]\nfn fake_fixture() {}\n")

            audit = editor_module_convention_audit(root).to_json()

        self.assertEqual(
            audit["oversized_test_files"],
            [
                {
                    "path": "zircon_editor/src/ui/example/behavior_tests.rs",
                    "lines": 801,
                    "owner_class": "editor-ui-tests",
                }
                ,
                {
                    "path": "zircon_editor/src/ui/example/cfg_tests.rs",
                    "lines": 801,
                    "owner_class": "editor-ui-tests",
                },
            ],
        )
        self.assertEqual(
            audit["oversized_production_files"],
            [
                {
                    "path": "zircon_editor/src/ui/example/documented_tests.rs",
                    "lines": 1002,
                    "owner_class": "editor-ui",
                },
                {
                    "path": "zircon_editor/src/ui/example/fixture_tests.rs",
                    "lines": 1002,
                    "owner_class": "editor-ui",
                },
            ],
        )

    def test_blank_test_budget_exemption_reason_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_rust(root / "zircon_editor/src/tests/fixture.rs", 801)

            with self.assertRaisesRegex(ValueError, "non-empty"):
                editor_module_convention_audit(
                    root,
                    test_file_budget_exemptions={
                        "zircon_editor/src/tests/fixture.rs": "   "
                    },
                )


if __name__ == "__main__":
    unittest.main()
