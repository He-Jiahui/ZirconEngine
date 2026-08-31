from pathlib import Path
import re
import unittest


ROOT = Path(__file__).resolve().parents[2]
RECORD_RS = ROOT / (
    "zircon_runtime/src/graphics/runtime/render_framework/viewport_record/"
    "viewport_record.rs"
)
QUERY_RS = ROOT / (
    "zircon_runtime/src/graphics/runtime/render_framework/viewport_record/"
    "visible_spatial_query.rs"
)


def source(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def compact(text: str) -> str:
    return re.sub(r"\s+", "", text)


def store_body() -> str:
    text = source(QUERY_RS)
    return text.split("fn store_visible_spatial_query(", 1)[1].split(
        "fn visible_spatial_query(", 1
    )[0]


def getter_body() -> str:
    text = source(QUERY_RS)
    return text.split("fn visible_spatial_query(", 1)[1].split("#[cfg(test)]", 1)[0]


class Runtime09H2DirectVisibleSnapshotContract(unittest.TestCase):
    def test_viewport_record_stores_the_snapshot_directly(self) -> None:
        text = compact(source(RECORD_RS))

        self.assertIn(
            "last_visible_spatial_query:Option<crate::core::framework::render::"
            "RenderVisibleSpatialQuerySnapshot>",
            text,
        )
        self.assertNotIn("Option<Arc<crate::core::framework::render::RenderVisible", text)

    def test_store_allocates_only_the_shared_query_handle(self) -> None:
        body = compact(store_body())

        self.assertIn("letquery=Arc::new(VisibleSpatialQuery::from_context", body)
        self.assertIn(
            "self.last_visible_spatial_query=Some(RenderVisibleSpatialQuerySnapshot::new(",
            body,
        )
        self.assertNotIn("Some(Arc::new(RenderVisibleSpatialQuerySnapshot::new(", body)

    def test_getter_clones_the_cheap_inner_query_handle(self) -> None:
        body = compact(getter_body())

        self.assertIn("self.last_visible_spatial_query.as_ref().cloned()", body)
        self.assertNotIn("as_deref().cloned()", body)

    def test_direct_storage_behavior_has_a_rust_contract(self) -> None:
        self.assertIn(
            "viewport_record_returns_owned_visible_snapshot_without_consuming_storage",
            source(QUERY_RS),
        )


if __name__ == "__main__":
    unittest.main()
