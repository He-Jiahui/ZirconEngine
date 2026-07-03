import unittest
from pathlib import Path


class TilemapEditorAuthoringTypedErrorDiagnosticsTests(unittest.TestCase):
    def test_tilemap_editor_converts_typed_authoring_errors_at_editor_boundary(
        self,
    ) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        authoring_source = (
            repo_root / "zircon_plugins/tilemap_2d/editor/src/authoring.rs"
        ).read_text(encoding="utf-8")

        self.assertIn(
            "pub fn validate_tilemap_for_editor(tilemap: &TileMapAsset) -> Vec<String>",
            authoring_source,
        )
        self.assertIn("diagnostics.push(error.to_string());", authoring_source)
        self.assertNotIn("diagnostics.push(error);", authoring_source)


if __name__ == "__main__":
    unittest.main()
