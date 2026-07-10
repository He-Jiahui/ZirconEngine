from tools.tests.plugin_status_document import StatusDocumentPath as Path
import unittest


REPO_ROOT = Path(__file__).resolve().parents[2]
STATUS_ID = "plugins_13_m5_t1_ui_document_importer_zui_document_kind_convergence"

STATUS_DOCS = [
    "docs/plans/zircon_plugins/09-export-publishing.md",
    "docs/plans/zircon_plugins/13-standalone-plugin-build.md",
    "docs/zircon_plugins/plugin-standalone-build.md",
    "docs/plans/zircon_editor/editor_ui/11-zui-suffix-convergence-and-ui-toml-retirement.md",
    "docs/plans/engine-code-structure-convention.md",
    "docs/plans/engine-code-review-findings-2026-06.md",
    ".codex/sessions/20260628-0317-zui-migration-validation.md",
]


class UiDocumentImporterZuiDocumentKindConvergenceStatusTests(unittest.TestCase):
    def test_ui_document_importer_runtime_and_manifest_are_kind_aware(self):
        runtime_plugin = (
            REPO_ROOT / "zircon_plugins/ui_document_importer/runtime/src/plugin.rs"
        ).read_text(encoding="utf-8")
        runtime_lib = (
            REPO_ROOT / "zircon_plugins/ui_document_importer/runtime/src/lib.rs"
        ).read_text(encoding="utf-8")
        manifest = (REPO_ROOT / "zircon_plugins/ui_document_importer/plugin.toml").read_text(
            encoding="utf-8"
        )

        required_markers = [
            (runtime_plugin, "ui_document_importer.zui_document"),
            (
                runtime_plugin,
                "with_additional_output_kinds([AssetKind::UiLayout, AssetKind::UiStyle])",
            ),
            (runtime_plugin, "import_ui_zui_document"),
            (runtime_lib, "UiV2AssetKind::View"),
            (runtime_lib, "UiV2AssetKind::Style | UiV2AssetKind::ThemeTokens"),
            (runtime_lib, "UiV2AssetKind::Component"),
            (runtime_lib, "zui_importer_decodes_view_and_style_assets"),
            (manifest, 'id = "ui_document_importer.zui_document"'),
            (manifest, 'additional_output_kinds = ["UiLayout", "UiStyle"]'),
        ]
        missing = [marker for source, marker in required_markers if marker not in source]
        self.assertFalse(
            missing,
            "UI document importer .zui document convergence markers are missing:\n"
            + "\n".join(missing),
        )

    def test_current_status_docs_record_ui_document_importer_convergence(self):
        required_phrases = [
            STATUS_ID,
            "ui_document_importer.zui_document",
            "additional_output_kinds = [\"UiLayout\", \"UiStyle\"]",
            "zui_importer_decodes_view_and_style_assets",
            "7/7",
            "target_count=39",
            "dist_capable=39",
            "runtime_registration=0",
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
            "UI document importer .zui document convergence status is incomplete:\n"
            + "\n".join(missing),
        )


if __name__ == "__main__":
    unittest.main()
