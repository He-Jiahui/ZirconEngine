import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]

STATUS_ID = "editor_ui_11_m5_active_zui_wording_runtime_diagnostic_guard_passed"

STATUS_DOC_REQUIREMENTS = {
    "docs/plans/zircon_editor/editor_ui/11-zui-suffix-convergence-and-ui-toml-retirement.md": [
        STATUS_ID,
        "active `.zui` wording / runtime diagnostics",
        "test_zui_current_layout_wording_convergence.py",
        "native_host_contract.rs",
        "runtime_ui_golden.rs",
        ".zui asset document diagnostics",
        "Cargo gate not claimed",
    ],
    "docs/editor-and-tooling/zui-asset-governance.md": [
        STATUS_ID,
        "active runtime/editor diagnostics use `.zui` document wording",
        "UiV2* Rust type names remain schema/API names",
        "test_zui_current_layout_wording_convergence.py",
    ],
    "docs/ui-and-layout/ui-asset-documents-and-editor-protocol.md": [
        STATUS_ID,
        "editor/runtime-facing `.zui` diagnostics",
        ".zui asset document",
        "UiV2Document",
    ],
    "docs/plans/engine-code-structure-convention.md": [
        STATUS_ID,
        "current `.zui` diagnostics must not call active assets v2",
        "UiV2* Rust type names may remain internal schema/API names",
        "test_zui_current_layout_wording_convergence.py",
    ],
    "docs/plans/engine-code-review-findings-2026-06.md": [
        STATUS_ID,
        "active `.zui` tests and diagnostics no longer describe current assets as v2",
        "runtime_ui_golden.rs",
        "no loader/importer compatibility path",
    ],
    ".codex/sessions/20260628-0317-zui-migration-validation.md": [
        STATUS_ID,
        "active `.zui` wording / runtime diagnostics",
        "focused current wording guard",
        ".zui asset document diagnostics",
        "Cargo gate not claimed",
    ],
}


class ZuiDocsCurrentStatusActiveZuiWordingGuardTests(unittest.TestCase):
    def test_active_zui_wording_runtime_diagnostic_status_is_recorded(self):
        failures: list[str] = []
        for relative_path, required_phrases in STATUS_DOC_REQUIREMENTS.items():
            text = (REPO_ROOT / relative_path).read_text(encoding="utf-8")
            for phrase in required_phrases:
                if phrase not in text:
                    failures.append(f"{relative_path}: {phrase}")

        if failures:
            self.fail(
                "Active .zui wording/runtime diagnostic status is missing:\n"
                + "\n".join(failures)
            )


if __name__ == "__main__":
    unittest.main()
