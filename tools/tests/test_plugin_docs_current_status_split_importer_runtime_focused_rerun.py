from tools.tests.plugin_status_document import StatusDocumentPath as Path
import unittest


REPO_ROOT = Path(__file__).resolve().parents[2]
STATUS_ID = "plugins_13_m5_t1_split_importer_runtime_focused_rerun"

STATUS_DOCS = [
    "docs/plans/zircon_plugins/index.md",
    "docs/plans/zircon_plugins/13-standalone-plugin-build.md",
    "docs/zircon_plugins/plugin-standalone-build.md",
    "docs/plans/engine-code-structure-convention.md",
    "docs/plans/engine-code-review-findings-2026-06.md",
]


class SplitImporterRuntimeFocusedRerunStatusTests(unittest.TestCase):
    def test_current_status_docs_record_split_importer_runtime_focused_rerun(self):
        required_phrases = [
            STATUS_ID,
            "package_manifest_declares_opus_importer_dist_contract",
            "package_manifest_declares_wgsl_importer_dist_contract",
            "package_manifest_declares_ui_document_importer_dist_contract",
            "zui_importer_decodes_view_and_style_assets",
            "opus_importer 1/1",
            "shader_wgsl_importer 1/1",
            "ui_document_importer 7/7",
            "不声明 Hub/editor E2E、完整 export matrix 或 startup-to-first-frame",
        ]

        missing: list[str] = []
        for relative_path in STATUS_DOCS:
            source = (REPO_ROOT / relative_path).read_text(encoding="utf-8")
            for phrase in required_phrases:
                if phrase not in source:
                    missing.append(f"{relative_path}: {phrase}")

        self.assertFalse(
            missing,
            "split importer runtime focused rerun status is incomplete:\n"
            + "\n".join(missing),
        )


if __name__ == "__main__":
    unittest.main()
