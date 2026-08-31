import unittest
from pathlib import Path


def read_source(path: str) -> str:
    return Path(path).read_text(encoding="utf-8")


class EditorAssetPointerSnapshotProjectionContractTests(unittest.TestCase):
    def test_pointer_publication_uses_the_narrow_projection(self) -> None:
        source = read_source(
            "zircon_editor/src/ui/retained_host/app/pointer_layout/asset_surfaces/sync.rs"
        )
        self.assertIn("snapshot.pointer_projection()", source)
        self.assertNotIn("Arc::new(snapshot.clone())", source)

    def test_projection_keeps_only_pointer_authority_fields(self) -> None:
        source = read_source(
            "zircon_editor/src/ui/workbench/snapshot/asset/asset_workspace_snapshot.rs"
        )
        projection = source.split("pub(crate) fn pointer_projection", 1)[1].split("    }", 1)[0]
        for field in ["visible_assets", "selection"]:
            self.assertIn(field, projection)
        for field in ["folder_tree", "visible_folders", "creation_menu", "project_root"]:
            self.assertNotIn(field, projection)


if __name__ == "__main__":
    unittest.main()
