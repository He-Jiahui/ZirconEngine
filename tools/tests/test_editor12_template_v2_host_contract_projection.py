"""Static contract coverage for the generic plugin V2 retained-host pane path."""

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]


def source(relative_path: str) -> str:
    return (ROOT / relative_path).read_text(encoding="utf-8")


class EditorPluginV2HostContractTests(unittest.TestCase):
    def test_template_v2_pane_has_one_generic_host_node_carrier(self) -> None:
        basic = source(
            "zircon_editor/src/ui/retained_host/host_contract/data/panes/basic.rs"
        )
        pane = source(
            "zircon_editor/src/ui/retained_host/host_contract/data/panes/pane.rs"
        )

        self.assertIn("struct TemplateV2PaneData", basic)
        self.assertIn("pub nodes: ModelRc<TemplatePaneNodeData>", basic)
        self.assertIn("pub template_v2: TemplateV2PaneData", pane)

    def test_template_v2_payload_reaches_every_generic_host_consumer(self) -> None:
        conversion = source(
            "zircon_editor/src/ui/retained_host/ui/apply_presentation/pane_conversion.rs"
        )
        projection = source(
            "zircon_editor/src/ui/retained_host/ui/pane_data_conversion/template_runtime_projection.rs"
        )
        consumers = [
            "zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/docks/pane/template_nodes/selection.rs",
            "zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node/pane_nodes.rs",
            "zircon_editor/src/ui/retained_host/host_contract/window/template_hover/panes.rs",
            "zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry/pane_frames/template_nodes/source.rs",
            "zircon_editor/src/ui/retained_host/host_contract/data/world_space_submission/builder/pane.rs",
        ]

        self.assertIn("has_template_v2_payload", conversion)
        self.assertIn("template_v2", conversion)
        self.assertIn("PanePayload::TemplateV2", projection)
        self.assertIn("to_host_contract_template_v2_pane_from_host_pane_with_runtime", projection)
        self.assertIn("build_host_model_with_surface", projection)
        for relative_path in consumers:
            self.assertIn("template_v2.nodes", source(relative_path), relative_path)

        implementation = "\n".join([conversion, projection] + [source(path) for path in consumers])
        self.assertNotIn("navigation.", implementation.lower())


if __name__ == "__main__":
    unittest.main()
