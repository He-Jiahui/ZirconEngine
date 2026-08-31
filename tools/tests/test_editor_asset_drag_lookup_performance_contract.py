import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
CONTENT_DRAG_PAYLOAD_PATH = (
    REPO_ROOT
    / "zircon_editor"
    / "src"
    / "ui"
    / "retained_host"
    / "app"
    / "asset_drag_payload"
    / "content.rs"
)
CONTENT_CONTEXT_MENU_PATH = (
    REPO_ROOT
    / "zircon_editor"
    / "src"
    / "ui"
    / "retained_host"
    / "app"
    / "asset_content_pointer"
    / "context_menu.rs"
)
ITEM_GENERATION_PATH = (
    REPO_ROOT
    / "zircon_editor"
    / "src"
    / "ui"
    / "workbench"
    / "snapshot"
    / "asset"
    / "asset_workspace_item_generation.rs"
)


def rust_function(source: str, name: str) -> str:
    match = re.search(rf"fn\s+{re.escape(name)}\s*\(", source)
    if match is None:
        raise AssertionError(f"missing Rust function: {name}")
    opening = source.find("{", match.end())
    if opening < 0:
        raise AssertionError(f"missing Rust function body: {name}")
    depth = 1
    index = opening + 1
    while index < len(source) and depth:
        depth += source[index] == "{"
        depth -= source[index] == "}"
        index += 1
    if depth:
        raise AssertionError(f"unterminated Rust function: {name}")
    return source[opening + 1 : index - 1]


class EditorAssetDragLookupPerformanceContractTests(unittest.TestCase):
    def test_content_drag_uses_the_published_uuid_index(self):
        source = CONTENT_DRAG_PAYLOAD_PATH.read_text(encoding="utf-8")
        lookup = rust_function(source, "asset_drag_payload_from_snapshot")

        self.assertIn("visible_assets.selected_index(asset_uuid)", lookup)
        self.assertRegex(lookup, r"visible_assets\s*\.get\(index\)")
        self.assertNotIn(".iter()", lookup)
        self.assertNotIn(".find(", lookup)

    def test_uuid_lookup_authority_is_a_shared_hash_index(self):
        source = ITEM_GENERATION_PATH.read_text(encoding="utf-8")

        self.assertIn("indices_by_uuid: Arc<HashMap<String, usize>>", source)
        selected_index = rust_function(source, "selected_index")
        self.assertIn("self.indices_by_uuid.get(uuid).copied()", selected_index)

    def test_context_menu_reuses_the_published_uuid_index(self):
        source = CONTENT_CONTEXT_MENU_PATH.read_text(encoding="utf-8")

        self.assertIn("visible_assets.selected_index(&asset_uuid)", source)
        self.assertIn("visible_assets.get(index)", source)
        self.assertNotIn("visible_assets.iter()", source)
        self.assertNotIn(".find(|asset| asset.uuid == asset_uuid)", source)


if __name__ == "__main__":
    unittest.main()
