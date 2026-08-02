import unittest
from pathlib import Path


DOC_PATHS = (
    "docs/plans/zircon_plugins/01-plugin-architecture-core.md",
    "docs/zircon_runtime/plugin/extension_registry.md",
    "docs/plans/engine-code-structure-convention.md",
    "docs/plans/engine-code-review-findings-2026-06.md",
)


class PluginDocsCurrentStatusTypedExtensionFreezeTests(unittest.TestCase):
    def setUp(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        self.documents = {
            path: (repo_root / path).read_text(encoding="utf-8")
            for path in DOC_PATHS
        }

    def test_status_records_runtime_freeze_and_remaining_work(self) -> None:
        combined = "\n".join(self.documents.values())
        required_phrases = (
            "TypedExtensionPoint",
            "FrozenExtensionTable",
            "RuntimeExtensionRegistry::finalize",
            "runtime_extension_catalog_finalizes_dense_tables_before_apply",
            "runtime_extension_apply_finalizes_dense_tables",
            "owner_unload_revokes_all_slots",
            "插件架构整体状态：进行中",
        )
        missing = [phrase for phrase in required_phrases if phrase not in combined]
        self.assertEqual([], missing)


if __name__ == "__main__":
    unittest.main()
