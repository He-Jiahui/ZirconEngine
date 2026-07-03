from pathlib import Path
import unittest


REPO_ROOT = Path(__file__).resolve().parents[2]
ROOT_GUARD = REPO_ROOT / "tools/tests/test_zui_docs_suffix_convergence.py"
PLAN_SCOPE_GUARD = (
    REPO_ROOT / "tools/tests/test_zui_docs_suffix_plan_scope_guards.py"
)
STATUS_ID = "editor_ui_11_m5_zui_docs_plan_scope_guard_owner_split"

PLAN_SCOPE_METHODS = (
    "test_workbench_shell_plan_design_declares_zui_only_layout_authority",
    "test_ui_asset_management_plan_uses_zui_for_current_asset_scope",
    "test_style_theme_plan_token_scan_targets_zui_documents_only",
)

STATUS_DOCS = (
    "docs/plans/zircon_editor/editor_ui/11-zui-suffix-convergence-and-ui-toml-retirement.md",
    "docs/plans/engine-code-structure-convention.md",
    "docs/plans/engine-code-review-findings-2026-06.md",
    ".codex/sessions/20260628-0317-zui-migration-validation.md",
)


class ZuiDocsSuffixOwnerBoundaryTests(unittest.TestCase):
    def test_plan_scope_guards_live_in_focused_owner(self) -> None:
        root_source = ROOT_GUARD.read_text(encoding="utf-8")
        plan_scope_source = PLAN_SCOPE_GUARD.read_text(encoding="utf-8")

        self.assertLessEqual(
            len(root_source.splitlines()),
            950,
            "root zui docs suffix convergence guard should stay below 950 lines",
        )
        self.assertLessEqual(
            len(plan_scope_source.splitlines()),
            220,
            "plan-scope zui docs suffix guard should stay focused",
        )
        for method in PLAN_SCOPE_METHODS:
            self.assertNotIn(method, root_source)
            self.assertIn(method, plan_scope_source)

    def test_plan_scope_guard_owner_split_status_is_recorded(self) -> None:
        required_phrases = [
            STATUS_ID,
            "tools/tests/test_zui_docs_suffix_convergence.py=920",
            "tools/tests/test_zui_docs_suffix_plan_scope_guards.py=121",
            "test_workbench_shell_plan_design_declares_zui_only_layout_authority",
            "test_ui_asset_management_plan_uses_zui_for_current_asset_scope",
            "test_style_theme_plan_token_scan_targets_zui_documents_only",
            "不声明 runtime/editor Cargo 验收",
        ]

        missing: list[str] = []
        for relative_path in STATUS_DOCS:
            source = (REPO_ROOT / relative_path).read_text(encoding="utf-8")
            for phrase in required_phrases:
                if phrase not in source:
                    missing.append(f"{relative_path}: {phrase}")

        self.assertFalse(
            missing,
            "ZUI docs suffix plan-scope owner split status is incomplete:\n"
            + "\n".join(missing),
        )


if __name__ == "__main__":
    unittest.main()
