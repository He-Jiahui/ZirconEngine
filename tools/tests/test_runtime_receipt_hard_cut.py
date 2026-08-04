import unittest
from pathlib import Path


class RuntimeReceiptHardCutTests(unittest.TestCase):
    def setUp(self) -> None:
        self.repo_root = Path(__file__).resolve().parents[2]
        self.runtime_absorption = (
            self.repo_root / "zircon_runtime/src/tests/runtime_absorption"
        )

    def test_retired_receipt_tree_and_mounts_do_not_return(self) -> None:
        retired_paths = (
            self.runtime_absorption / "plan_status.rs",
            self.runtime_absorption / "plan_status",
            self.runtime_absorption
            / "structure_convention/test_file_budget/status_slices",
            self.runtime_absorption
            / "structure_convention/test_file_budget/row_data",
            self.runtime_absorption
            / "structure_convention/test_file_budget/status_output_expected_slices.rs",
            self.runtime_absorption
            / "structure_convention/test_file_budget/status_output_row_data.rs",
            self.runtime_absorption / "naming_boundary/support/status_evidence.rs",
        )

        self.assertEqual([], [path.as_posix() for path in retired_paths if path.exists()])

        residuals = []
        retired_fragments = (
            "plan_status",
            "support/status_evidence.rs",
            "test_file_budget/status_slices",
            "test_file_budget/row_data",
            "status_output_expected_slices.rs",
            "status_output_row_data.rs",
        )
        for source_path in self.runtime_absorption.rglob("*.rs"):
            source = source_path.read_text(encoding="utf-8")
            if any(fragment in source for fragment in retired_fragments):
                residuals.append(source_path.relative_to(self.repo_root).as_posix())
        self.assertEqual([], residuals)

    def test_status_specific_python_auditor_is_removed(self) -> None:
        scripts = (
            self.repo_root
            / ".codex/skills/zircon-project-skills/"
            "zr-runtime-interface-convergence/scripts"
        )
        retired_modules = tuple(
            scripts
            / "runtime_structure_audits"
            / f"runtime_plan_status_{suffix}.py"
            for suffix in (
                "anchor_inventory",
                "boundary",
                "markdown",
                "output_anchors",
                "sources",
                "support_inventory",
            )
        )

        self.assertEqual(
            [],
            [
                path.relative_to(self.repo_root).as_posix()
                for path in retired_modules
                if path.exists()
            ],
        )
        audit_source = (scripts / "audit_runtime_structure.py").read_text(
            encoding="utf-8"
        )
        self.assertNotIn("runtime_plan_status", audit_source)

    def test_product_structure_guard_owners_remain_mounted(self) -> None:
        owner = self.runtime_absorption / "structure_convention"
        required = (
            owner / "module_convention_gate.rs",
            owner / "production_file_budget.rs",
            owner / "provider_boilerplate.rs",
            owner / "facade_surface.rs",
            owner / "lock_poison_policy.rs",
            owner / "runtime_dead_code/mod.rs",
            owner / "graphics_dead_code/mod.rs",
        )

        self.assertEqual([], [path.as_posix() for path in required if not path.is_file()])

        runtime_absorption_source = (self.runtime_absorption / "mod.rs").read_text(
            encoding="utf-8"
        )
        for mount in ("mod naming_boundary;", "mod structure_convention;"):
            self.assertIn(mount, runtime_absorption_source)

        structure_source = (self.runtime_absorption / "structure_convention.rs").read_text(
            encoding="utf-8"
        )
        required_mounts = (
            "mod module_convention_gate;",
            "mod production_file_budget;",
            "mod provider_boilerplate;",
            "mod facade_surface;",
            "mod lock_poison_policy;",
            "mod runtime_dead_code;",
            "mod graphics_dead_code;",
        )
        for mount in required_mounts:
            self.assertIn(mount, structure_source)


if __name__ == "__main__":
    unittest.main()
