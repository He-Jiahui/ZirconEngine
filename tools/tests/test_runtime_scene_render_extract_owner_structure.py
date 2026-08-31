from pathlib import Path
import unittest


REPO_ROOT = Path(__file__).resolve().parents[2]
OWNER_ROOT = REPO_ROOT / "zircon_runtime" / "src" / "scene" / "render_extract"


class RuntimeSceneRenderExtractOwnerStructureTests(unittest.TestCase):
    def test_render_extract_root_delegates_to_the_producer_owner(self) -> None:
        root = (OWNER_ROOT / "mod.rs").read_text(encoding="utf-8")
        producer = (OWNER_ROOT / "producer.rs").read_text(encoding="utf-8")

        self.assertEqual("mod producer;", root.strip())
        for forbidden in ("impl World", "impl RenderExtractProducer", "self.clone()"):
            self.assertNotIn(forbidden, root)

        for required in (
            "impl World",
            "pub fn to_render_frame_extract",
            "pub(crate) fn build_prepared_render_frame_extract",
            "impl RenderExtractProducer for World",
            "self.clone()",
        ):
            self.assertIn(required, producer)
        self.assertNotIn("RenderFrameExtract::from_snapshot", producer)


if __name__ == "__main__":
    unittest.main()
