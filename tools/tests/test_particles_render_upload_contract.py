import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
EXECUTORS = ROOT / "zircon_plugins/particles/runtime/src/render/executors.rs"


class ParticlesRenderUploadContractTests(unittest.TestCase):
    def test_transparent_render_passes_upload_sink_without_an_extra_mutable_reference(self):
        source = EXECUTORS.read_text(encoding="utf-8")

        self.assertIn("            draw.buffer_uploads,", source)
        self.assertNotIn("            &mut draw.buffer_uploads,", source)


if __name__ == "__main__":
    unittest.main()
