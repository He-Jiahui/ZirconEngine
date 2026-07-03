import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]

STATUS_ID = "editor_ui_11_m5_current_layout_wording_zui_guard_passed"
REQUIRED_STATUS_PHRASES = [
    STATUS_ID,
    "test_zui_current_layout_wording_convergence.py",
    "critical_editor_shells_are_hard_cut_to_zui_assets",
    "runtime_ui_golden_is_hard_cut_to_zui_fixtures",
    "runtime_fixture_host_tests_are_hard_cut_to_zui_paths",
    "component_showcase_is_hard_cut_to_zui_catalog_components",
    "retired `.ui.toml` / `.v2.ui.toml` suffixes",
]

STATUS_DOCS = [
    "docs/plans/zircon_editor/editor_ui/11-zui-suffix-convergence-and-ui-toml-retirement.md",
    "docs/plans/engine-code-structure-convention.md",
    "docs/plans/engine-code-review-findings-2026-06.md",
    ".codex/sessions/20260628-0317-zui-migration-validation.md",
]


class ZuiDocsCurrentStatusCurrentLayoutWordingGuardTests(unittest.TestCase):
    def test_current_layout_wording_guard_status_is_recorded(self):
        failures: list[str] = []
        for relative_path in STATUS_DOCS:
            text = (REPO_ROOT / relative_path).read_text(encoding="utf-8")
            for phrase in REQUIRED_STATUS_PHRASES:
                if phrase not in text:
                    failures.append(f"{relative_path}: {phrase}")

        if failures:
            self.fail(
                "Current .zui layout wording guard status is missing:\n"
                + "\n".join(failures)
            )


if __name__ == "__main__":
    unittest.main()
