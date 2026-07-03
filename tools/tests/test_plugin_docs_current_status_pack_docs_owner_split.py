import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]

STATUS_SLUG = "plugins_13_m5_t1_pack_docs_status_owner_split"
REQUIRED_PHRASES = [
    STATUS_SLUG,
    "test_plugin_docs_current_status_pack_owner_splits.py",
    "test_plugin_docs_current_status_pack_owner_boundaries.py",
    "test_plugin_docs_current_status_convergence.py",
    "Pack",
    "current-status",
    "4270",
    "3664",
    "307",
]

REQUIRED_FILES = [
    REPO_ROOT / "docs/plans/zircon_plugins/13-standalone-plugin-build.md",
    REPO_ROOT / "docs/plans/zircon_plugins/09-export-publishing.md",
    REPO_ROOT / "docs/zircon_plugins/plugin-standalone-build.md",
    REPO_ROOT / "docs/cli-and-tooling/zircon-export-tool.md",
    REPO_ROOT / "docs/plans/engine-code-structure-convention.md",
    REPO_ROOT / "docs/plans/engine-code-review-findings-2026-06.md",
    REPO_ROOT / ".codex/sessions/20260628-0317-zui-migration-validation.md",
]


class PackDocsStatusOwnerSplitDocsTests(unittest.TestCase):
    def test_pack_docs_status_owner_split_is_recorded(self):
        missing: list[str] = []
        for path in REQUIRED_FILES:
            text = path.read_text(encoding="utf-8")
            for phrase in REQUIRED_PHRASES:
                if phrase not in text:
                    missing.append(f"{path.relative_to(REPO_ROOT)}: {phrase}")

        self.assertEqual([], missing)


if __name__ == "__main__":
    unittest.main()
