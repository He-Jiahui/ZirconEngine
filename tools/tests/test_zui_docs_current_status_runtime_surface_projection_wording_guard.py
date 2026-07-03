import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]

STATUS_ID = "editor_ui_11_m5_runtime_surface_projection_zui_wording_guard_passed"

STATUS_DOC_REQUIREMENTS = {
    "docs/plans/zircon_editor/editor_ui/11-zui-suffix-convergence-and-ui-toml-retirement.md": [
        STATUS_ID,
        "runtime surface projection `.zui` wording",
        "asset_fixture_projection.rs",
        "zui_surface_projection_does_not_call_template_tree_builder",
        "Cargo gate not claimed",
    ],
    "docs/ui-and-layout/ui-asset-documents-and-editor-protocol.md": [
        STATUS_ID,
        ".zui surface projection",
        "asset_fixture_projection.rs",
        "UiV2SurfaceBuilder remains internal API name",
    ],
    "docs/zircon_runtime/structure/module-convention.md": [
        STATUS_ID,
        "runtime fixture surface tests use `.zui` wording",
        "asset_fixture_projection.rs",
        "UiV2* names remain internal implementation names",
    ],
    "docs/plans/engine-code-structure-convention.md": [
        STATUS_ID,
        "runtime fixture surface tests should use `.zui` wording",
        "UiV2* Rust type names may remain internal schema/API names",
    ],
    "docs/plans/engine-code-review-findings-2026-06.md": [
        STATUS_ID,
        "runtime fixture surface projection no longer describes current fixtures as ui v2 surface",
        "no loader/importer compatibility path",
    ],
    ".codex/sessions/20260628-0317-zui-migration-validation.md": [
        STATUS_ID,
        "runtime surface projection `.zui` wording",
        "focused current wording guard",
        "Cargo gate not claimed",
    ],
}


class ZuiDocsCurrentStatusRuntimeSurfaceProjectionWordingGuardTests(unittest.TestCase):
    def test_runtime_surface_projection_zui_wording_status_is_recorded(self):
        failures: list[str] = []
        for relative_path, required_phrases in STATUS_DOC_REQUIREMENTS.items():
            text = (REPO_ROOT / relative_path).read_text(encoding="utf-8")
            for phrase in required_phrases:
                if phrase not in text:
                    failures.append(f"{relative_path}: {phrase}")

        if failures:
            self.fail(
                "Runtime surface projection .zui wording status is missing:\n"
                + "\n".join(failures)
            )


if __name__ == "__main__":
    unittest.main()
