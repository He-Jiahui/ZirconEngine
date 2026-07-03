from pathlib import Path
import unittest


REPO_ROOT = Path(__file__).resolve().parents[2]
STATUS_ID = "editor_ui_11_m5_zui_docs_suffix_convergence_test_owner_budget_recovered"
REQUIRED_STATUS_PHRASES = [
    STATUS_ID,
    "tools/tests/test_zui_docs_suffix_convergence.py<=380",
    "tools/tests/test_zui_docs_suffix_status_guards.py",
    "test_runtime_asset_ui_reference_zui_guard_status_is_recorded",
    "test owner budget recovered",
    "python -m unittest tools.tests.test_zui_docs_suffix_convergence_test_owner_boundaries",
    "不声明 Hub/editor E2E、完整 export matrix 或 startup-to-first-frame",
]

STATUS_DOCS = [
    "docs/plans/zircon_editor/editor_ui/11-zui-suffix-convergence-and-ui-toml-retirement.md",
    "docs/plans/engine-code-structure-convention.md",
    "docs/plans/engine-code-review-findings-2026-06.md",
    ".codex/sessions/20260628-0317-zui-migration-validation.md",
]


class ZuiDocsCurrentStatusSuffixTestOwnerBudgetTests(unittest.TestCase):
    def test_suffix_convergence_test_owner_budget_status_is_recorded(self) -> None:
        failures: list[str] = []
        for relative_path in STATUS_DOCS:
            text = (REPO_ROOT / relative_path).read_text(encoding="utf-8")
            for phrase in REQUIRED_STATUS_PHRASES:
                if phrase not in text:
                    failures.append(f"{relative_path}: {phrase}")

        self.assertFalse(
            failures,
            "ZUI docs suffix convergence test-owner budget status is missing:\n"
            + "\n".join(failures),
        )


if __name__ == "__main__":
    unittest.main()
