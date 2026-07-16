import ast
import sys
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
AUDIT_SCRIPTS = (
    REPO_ROOT
    / ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts"
)
sys.path.insert(0, str(AUDIT_SCRIPTS))

from runtime_structure_audits.dynamic_runtime_api_boundary import (  # noqa: E402
    dynamic_runtime_api_boundary_audit,
)
from runtime_structure_audits.dynamic_runtime_api_archive_inventory import (  # noqa: E402
    RUNTIME_15_RUNTIME_INDEX_OUTPUT_ARCHIVE,
)


class RuntimeDynamicApiBoundaryArchiveOwnershipTests(unittest.TestCase):
    def test_dynamic_api_inventories_hard_cut_runtime_15_evidence_to_archive(self) -> None:
        inventory_root = AUDIT_SCRIPTS / "runtime_structure_audits"
        canonical = (
            "docs/plans/_archive/zircon_runtime/runtime/15/"
            "2026-07-09-runtime-index-output-records.md"
        )
        retired = (
            "docs/plans/zircon_runtime/runtime/15/"
            "2026-07-09-runtime-index-output-records.md"
        )
        owner_path = inventory_root / "dynamic_runtime_api_archive_inventory.py"
        owner_tree = ast.parse(
            owner_path.read_text(encoding="utf-8"), filename=str(owner_path)
        )
        owner_literals = [
            node.value
            for node in ast.walk(owner_tree)
            if isinstance(node, ast.Constant) and isinstance(node.value, str)
        ]

        self.assertEqual(canonical, RUNTIME_15_RUNTIME_INDEX_OUTPUT_ARCHIVE)
        self.assertEqual(1, owner_literals.count(canonical))
        self.assertNotIn(retired, owner_literals)

        for name in (
            "dynamic_runtime_api_ui_contract_inventory.py",
            "dynamic_runtime_api_validation_inventory.py",
        ):
            consumer_path = inventory_root / name
            consumer_tree = ast.parse(
                consumer_path.read_text(encoding="utf-8"), filename=str(consumer_path)
            )
            consumer_literals = [
                node.value
                for node in ast.walk(consumer_tree)
                if isinstance(node, ast.Constant) and isinstance(node.value, str)
            ]
            imports = [
                node
                for node in ast.walk(consumer_tree)
                if isinstance(node, ast.ImportFrom)
                and node.module
                == "runtime_structure_audits.dynamic_runtime_api_archive_inventory"
            ]

            self.assertEqual(1, len(imports), name)
            self.assertEqual(
                ["RUNTIME_15_RUNTIME_INDEX_OUTPUT_ARCHIVE"],
                [alias.name for alias in imports[0].names],
                name,
            )
            self.assertNotIn(canonical, consumer_literals, name)
            self.assertNotIn(retired, consumer_literals, name)

    def test_numbered_runtime_09_10_archives_supply_concrete_contract_evidence(self) -> None:
        report = dynamic_runtime_api_boundary_audit(REPO_ROOT)

        self.assertEqual([], report["missing_ui_pending_gate_anchors"])
        self.assertEqual([], report["missing_ui_contract_single_source_anchors"])
        self.assertEqual([], report["missing_ui_v2_contract_sync_anchors"])
        self.assertEqual([], report["missing_doc_anchors"])
        self.assertEqual([], report["risks"])


if __name__ == "__main__":
    unittest.main()
