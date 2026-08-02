import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]

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

if __name__ == "__main__":
    unittest.main()
