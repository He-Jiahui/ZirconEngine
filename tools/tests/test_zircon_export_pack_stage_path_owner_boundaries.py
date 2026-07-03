from __future__ import annotations

import ast
from pathlib import Path
import unittest


REPO_ROOT = Path(__file__).resolve().parents[2]
PACK_STAGE = REPO_ROOT / "tools/zircon_export/pack_stage.py"
PACK_STAGE_PATHS = REPO_ROOT / "tools/zircon_export/pack_stage_paths.py"


PATH_OWNER_FUNCTIONS = {
    "pack_asset_manifest_argument_diagnostic",
    "pack_file_argument_diagnostic",
    "pack_optional_path_argument_diagnostic",
    "pack_asset_manifest_path",
    "pack_output_path",
    "resolve_pack_optional_path",
    "resolve_pack_stage_path",
    "pack_delta_argument_diagnostics",
    "pack_asset_manifest_diagnostic",
}


def _function_names(path: Path) -> set[str]:
    tree = ast.parse(path.read_text(encoding="utf-8"))
    return {
        node.name
        for node in ast.walk(tree)
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef))
    }


def _imports_from(path: Path, module_name: str) -> set[str]:
    tree = ast.parse(path.read_text(encoding="utf-8"))
    imported: set[str] = set()
    for node in ast.walk(tree):
        if isinstance(node, ast.ImportFrom) and node.module == module_name:
            for alias in node.names:
                imported.add(alias.name)
    return imported


class PackStagePathOwnerBoundaryTests(unittest.TestCase):
    def test_pack_path_and_argument_helpers_live_in_path_owner(self) -> None:
        self.assertTrue(
            PACK_STAGE_PATHS.exists(),
            "Pack path and argument helpers belong in pack_stage_paths.py",
        )
        stage_functions = _function_names(PACK_STAGE)
        path_functions = _function_names(PACK_STAGE_PATHS)

        for function_name in sorted(PATH_OWNER_FUNCTIONS):
            self.assertIn(
                function_name,
                path_functions,
                f"{function_name} belongs in pack_stage_paths.py",
            )
            self.assertNotIn(
                function_name,
                stage_functions,
                f"{function_name} should not live in pack_stage.py",
            )

        self.assertTrue(
            PATH_OWNER_FUNCTIONS.issubset(
                _imports_from(PACK_STAGE, "pack_stage_paths")
            ),
            "pack_stage.py should consume path helpers through pack_stage_paths.py",
        )

    def test_pack_stage_and_path_owner_file_budgets(self) -> None:
        self.assertLessEqual(
            len(PACK_STAGE.read_text(encoding="utf-8").splitlines()),
            340,
            "pack_stage.py should stay focused on Pack orchestration",
        )
        self.assertLessEqual(
            len(PACK_STAGE_PATHS.read_text(encoding="utf-8").splitlines()),
            140,
            "pack_stage_paths.py should stay focused on path and argument helpers",
        )


if __name__ == "__main__":
    unittest.main()
