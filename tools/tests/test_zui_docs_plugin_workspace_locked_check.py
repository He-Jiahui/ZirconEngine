from pathlib import Path
import unittest


REPO_ROOT = Path(__file__).resolve().parents[2]
STATUS_ID = "editor_ui_11_m5_plugin_workspace_zui_only_locked_all_targets_check_passed"
REQUIRED_STATUS_PHRASES = [
    STATUS_ID,
    "cargo check --manifest-path zircon_plugins\\Cargo.toml --workspace --locked --all-targets",
    "Finished `dev` profile",
    "production legacy UI suffix file count = 0",
    "target_count=39",
    "manifest_schema_violations=0",
    "retired_ui_asset_files=0",
    "zui-only-clean",
    "tools/tests/test_zui_docs_plugin_workspace_locked_check.py",
    "不声明 Hub/editor E2E、完整 export matrix 或 startup-to-first-frame",
]

STATUS_DOCS = [
    "docs/plans/zircon_editor/editor_ui/11-zui-suffix-convergence-and-ui-toml-retirement.md",
    "docs/plans/engine-code-structure-convention.md",
    "docs/plans/engine-code-review-findings-2026-06.md",
    ".codex/sessions/20260628-0317-zui-migration-validation.md",
]


class ZuiDocsPluginWorkspaceLockedCheckTests(unittest.TestCase):
    def test_plugin_workspace_zui_only_locked_check_status_is_recorded(self) -> None:
        failures: list[str] = []
        for relative_path in STATUS_DOCS:
            text = (REPO_ROOT / relative_path).read_text(encoding="utf-8")
            for phrase in REQUIRED_STATUS_PHRASES:
                if phrase not in text:
                    failures.append(f"{relative_path}: {phrase}")

        self.assertFalse(
            failures,
            "Plugin workspace .zui-only locked all-targets status is missing:\n"
            + "\n".join(failures),
        )


if __name__ == "__main__":
    unittest.main()
