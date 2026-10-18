import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
RENDER_MODULE = ROOT / "zircon_runtime/src/core/framework/render/mod.rs"


class VirtualGeometryRenderPayloadExportContractTests(unittest.TestCase):
    def test_cluster_range_payload_is_reexported_with_the_page_payload(self):
        source = RENDER_MODULE.read_text(encoding="utf-8")
        export_start = source.index("pub use virtual_geometry_debug_snapshot::{")
        export_end = source.index("};", export_start)
        export = source[export_start:export_end]

        self.assertIn("RenderVirtualGeometryPagePayload,", export)
        self.assertIn("RenderVirtualGeometryPagePayloadClusterRange,", export)
        self.assertIn("RenderVirtualGeometryPagePayloadVertex,", export)


if __name__ == "__main__":
    unittest.main()
