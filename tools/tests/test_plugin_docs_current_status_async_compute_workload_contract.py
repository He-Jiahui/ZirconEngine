import unittest
from tools.tests.plugin_status_document import StatusDocumentPath as Path


REPO_ROOT = Path(__file__).resolve().parents[2]
STATUS_ID = "plugins_13_m5_t1_async_compute_workload_and_workspace_followups_focused_passed"
REQUIRED_STATUS_PHRASES = [
    "view.<view_id>.open",
    "support.authoring.*",
    "asset_importer.model.runtime",
    "QueueLane::AsyncCompute",
    "RenderGraphComputeWorkload",
    "Hybrid GI",
    "Virtual Geometry",
    "Particles",
    "SSAO",
    "VFX Graph",
    "resolved_layout_advances_for_sdf_glyphs",
    "graphics::tests::plugin_feature_compile::gi_and_virtual_geometry_opt_in_add_feature_runtime_passes_to_graph",
    "tools/tests/test_plugin_docs_current_status_async_compute_workload_contract.py",
    "不声明完整",
]

STATUS_DOCS = [
    "docs/plans/zircon_plugins/09-export-publishing.md",
    "docs/plans/zircon_plugins/13-standalone-plugin-build.md",
    "docs/zircon_plugins/plugin-standalone-build.md",
    "docs/plans/engine-code-structure-convention.md",
    "docs/plans/engine-code-review-findings-2026-06.md",
    "docs/plans/zircon_editor/editor_ui/11-zui-suffix-convergence-and-ui-toml-retirement.md",
    "docs/assets-and-rendering/render-framework-architecture.md",
]


class PluginDocsCurrentStatusAsyncComputeWorkloadContractTests(unittest.TestCase):
    def test_current_docs_record_async_compute_workload_followup_status(
        self,
    ) -> None:
        failures: list[str] = []
        combined_text_parts: list[str] = []
        for relative_path in STATUS_DOCS:
            text = (REPO_ROOT / relative_path).read_text(encoding="utf-8")
            combined_text_parts.append(text)
            if STATUS_ID not in text:
                failures.append(f"{relative_path}: missing {STATUS_ID}")

        combined_text = "\n".join(combined_text_parts)
        for phrase in REQUIRED_STATUS_PHRASES:
            if phrase not in combined_text:
                failures.append(f"combined docs: missing {phrase}")

        self.assertFalse(
            failures,
            "Current plugin docs do not record async compute workload followup status:\n"
            + "\n".join(failures),
        )


if __name__ == "__main__":
    unittest.main()
