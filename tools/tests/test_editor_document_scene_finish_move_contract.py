from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]


class EditorDocumentSceneFinishMoveContractTests(unittest.TestCase):
    def test_prepared_scene_finish_moves_the_document_without_cloning_the_scene(self) -> None:
        source = (
            ROOT / "zircon_editor/src/core/project/scene_document.rs"
        ).read_text(encoding="utf-8")
        finish = source.split("pub(crate) fn finish", 1)[1].split(
            "fn remove_staging", 1
        )[0]

        self.assertIn("document: Option<ProjectSceneDocument>", source)
        self.assertIn(".take()", finish)
        self.assertNotIn(".clone()", finish)


if __name__ == "__main__":
    unittest.main()
