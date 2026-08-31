from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SELECTION_TEXT = ROOT / "zircon_editor/src/ui/layouts/views/asset_browser/selection_text.rs"


def function_body(source: str, signature: str) -> str:
    start = source.index(signature)
    end = source.index("\npub(super) fn ", start + len(signature))
    return source[start:end]


class EditorAssetBreadcrumbIndexPerformanceContractTests(unittest.TestCase):
    def test_breadcrumb_builds_one_borrowed_parent_index(self) -> None:
        source = SELECTION_TEXT.read_text(encoding="utf-8")
        body = function_body(source, "pub(super) fn selected_folder_breadcrumb(")

        self.assertIn("HashMap::<&str, &AssetFolderSnapshot>::with_capacity", body)
        self.assertIn(".entry(folder.folder_id.as_str())", body)
        self.assertIn(".or_insert(folder)", body)
        self.assertIn("folders.get(id)", body)
        self.assertNotIn(".find(|folder| folder.folder_id == id)", body)
        self.assertNotIn(".collect::<Vec<_>>()", body)

    def test_breadcrumb_borrows_segments_and_stops_parent_cycles(self) -> None:
        source = SELECTION_TEXT.read_text(encoding="utf-8")
        body = function_body(source, "pub(super) fn selected_folder_breadcrumb(")

        self.assertIn("Vec<&str>", body)
        self.assertIn("HashSet::<&str>", body)
        self.assertIn("visited.insert", body)
        self.assertNotIn("display_name.clone()", body)

    def test_selected_asset_uses_generation_indices_instead_of_scanning_visible_rows(self) -> None:
        source = SELECTION_TEXT.read_text(encoding="utf-8")
        start = source.index("pub(super) fn selected_asset(")
        end = source.index("\npub(super) fn has_asset_selection", start)
        body = source[start:end]

        self.assertIn("visible_assets.selected_index", body)
        self.assertIn("visible_assets.selected_indices()", body)
        self.assertIn("visible_assets.get(selected_index)", body)
        self.assertNotIn("visible_assets.iter()", body)
        self.assertNotIn(".find(", body)


if __name__ == "__main__":
    unittest.main()
