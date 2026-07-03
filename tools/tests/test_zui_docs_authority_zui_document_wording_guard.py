import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]

STATUS_ID = "editor_ui_11_m5_authority_docs_zui_document_wording_guard_passed"

AUTHORITY_DOC_WORDING_TARGETS = {
    "docs/editor-and-tooling/zui-asset-governance.md": [
        "production suffix for UI v2 documents",
        "current UI v2 schema version",
    ],
}

AUTHORITY_DOC_REQUIRED_PHRASES = {
    "docs/editor-and-tooling/zui-asset-governance.md": [
        "`.zui` is the production suffix for current UI documents.",
        "current `.zui` schema version",
        "UiV2 TOML schema structs in memory",
    ],
}

STATUS_DOC_REQUIREMENTS = {
    "docs/plans/zircon_editor/editor_ui/11-zui-suffix-convergence-and-ui-toml-retirement.md": [
        STATUS_ID,
        "authority docs `.zui` document wording",
        "test_zui_docs_authority_zui_document_wording_guard.py",
        "production suffix for current UI documents",
    ],
    "docs/editor-and-tooling/zui-asset-governance.md": [
        STATUS_ID,
        "authority docs `.zui` document wording",
        "current UI documents",
        "UiV2 TOML schema structs in memory",
    ],
    "docs/ui-and-layout/ui-asset-documents-and-editor-protocol.md": [
        STATUS_ID,
        "authority docs use `.zui` document wording",
        "current UI documents",
    ],
    "docs/plans/engine-code-structure-convention.md": [
        STATUS_ID,
        "authority docs must describe current layouts as `.zui` documents",
        "UiV2* Rust type names may remain internal schema/API names",
    ],
    "docs/plans/engine-code-review-findings-2026-06.md": [
        STATUS_ID,
        "zui-asset-governance.md no longer calls the production suffix UI v2 documents",
        "no runtime/editor code changed",
    ],
    ".codex/sessions/20260628-0317-zui-migration-validation.md": [
        STATUS_ID,
        "authority docs `.zui` document wording",
        "docs-only wording guard",
    ],
}


class ZuiDocsAuthorityZuiDocumentWordingGuardTests(unittest.TestCase):
    def test_authority_docs_describe_current_documents_as_zui(self):
        failures: list[str] = []
        for relative_path, stale_phrases in AUTHORITY_DOC_WORDING_TARGETS.items():
            text = (REPO_ROOT / relative_path).read_text(encoding="utf-8")
            for phrase in stale_phrases:
                if phrase in text:
                    failures.append(f"{relative_path}: stale {phrase}")

        for relative_path, required_phrases in AUTHORITY_DOC_REQUIRED_PHRASES.items():
            text = (REPO_ROOT / relative_path).read_text(encoding="utf-8")
            for phrase in required_phrases:
                if phrase not in text:
                    failures.append(f"{relative_path}: missing {phrase}")

        if failures:
            self.fail(
                "Current authority docs do not consistently use .zui document wording:\n"
                + "\n".join(failures)
            )

    def test_authority_docs_zui_document_wording_status_is_recorded(self):
        failures: list[str] = []
        for relative_path, required_phrases in STATUS_DOC_REQUIREMENTS.items():
            text = (REPO_ROOT / relative_path).read_text(encoding="utf-8")
            for phrase in required_phrases:
                if phrase not in text:
                    failures.append(f"{relative_path}: {phrase}")

        if failures:
            self.fail(
                "Authority docs .zui document wording status is missing:\n"
                + "\n".join(failures)
            )


if __name__ == "__main__":
    unittest.main()
