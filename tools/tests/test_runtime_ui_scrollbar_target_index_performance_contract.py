from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SCROLLBAR = (
    ROOT
    / "zircon_runtime/src/ui/surface/surface/default_interactions/scrollbar.rs"
)
CONTROL_INDEX = ROOT / "zircon_runtime/src/ui/surface/control_index.rs"


def function_body(source: str, name: str) -> str:
    start = source.index(f"fn {name}(")
    boundaries = (
        source.find(marker, start + 1)
        for marker in (
            "\nfn ",
            "\npub(super) fn ",
            "\npub(crate) fn ",
            "\npub fn ",
            "\n    fn ",
            "\n    pub(super) fn ",
            "\n    pub(crate) fn ",
            "\n    pub fn ",
            "\n#[cfg(",
        )
    )
    next_functions = [boundary for boundary in boundaries if boundary >= 0]
    return source[start:] if not next_functions else source[start : min(next_functions)]


class RuntimeUiScrollbarTargetIndexPerformanceContractTests(unittest.TestCase):
    def test_scrollbar_reference_lookup_delegates_to_surface_index(self) -> None:
        source = SCROLLBAR.read_text(encoding="utf-8")
        body = function_body(source, "resolve_node_reference")

        self.assertIn("first_node_id_for_reference", body)
        self.assertNotIn("tree.nodes.iter()", body)
        self.assertNotIn("self.tree.nodes.iter()", body)

    def test_reference_index_uses_hash_buckets_with_exact_validation(self) -> None:
        source = CONTROL_INDEX.read_text(encoding="utf-8")
        body = function_body(source, "first_node_id_for_reference")
        compact = "".join(body.split())

        self.assertIn("reference_hash", body)
        self.assertIn("reference_node_ids_by_hash", body)
        self.assertIn("node_matches_reference", body)
        self.assertIn("node_ids.iter().copied().find", compact)
        self.assertNotIn("tree.nodes.iter()", body)

    def test_reference_index_is_updated_through_existing_pending_node_sync(self) -> None:
        source = CONTROL_INDEX.read_text(encoding="utf-8")
        insert = function_body(source, "insert")
        remove = function_body(source, "remove")

        self.assertIn("insert_reference", insert)
        self.assertIn("remove_references", remove)
        self.assertIn("reference_hashes_by_node", source)

    def test_reference_hash_is_deterministic_and_allocation_free(self) -> None:
        source = CONTROL_INDEX.read_text(encoding="utf-8")
        body = function_body(source, "reference_hash")
        compact = "".join(body.split())

        self.assertIn("value.as_bytes()", compact)
        self.assertNotIn("DefaultHasher", body)
        self.assertNotIn(".to_string()", body)
        self.assertNotIn(".to_owned()", body)
        self.assertNotIn("format!", body)


if __name__ == "__main__":
    unittest.main()
