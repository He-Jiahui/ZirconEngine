from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
COLLECTION = (
    ROOT
    / "zircon_editor/src/ui/retained_host/ui/pane_data_conversion/"
    "pane_component_projection"
)


class EditorCollectionSourceWindowPerformanceContractTests(unittest.TestCase):
    def test_visible_window_collects_from_a_lazy_iterator(self) -> None:
        source = (COLLECTION / "collection_window.rs").read_text(encoding="utf-8")

        self.assertIn("fn collect_visible_collection_items<I>", source)
        self.assertIn("I: IntoIterator", source)

    def test_string_items_are_windowed_before_owned_string_collection(self) -> None:
        source = (COLLECTION / "collection_projection/items.rs").read_text(
            encoding="utf-8"
        )
        function = source.split("fn projected_collection_items", 1)[1].split(
            "fn projected_collection_rows", 1
        )[0]
        compact_function = "".join(function.split())

        self.assertNotIn("value_as_options", function)
        self.assertIn("filter_map(value_as_string)", function)
        self.assertIn("collect_visible_collection_items(items", compact_function)

    def test_typed_rows_are_windowed_before_row_dto_collection(self) -> None:
        source = (COLLECTION / "collection_projection/items.rs").read_text(
            encoding="utf-8"
        )
        function = source.split("fn projected_collection_rows", 1)[1].split(
            "fn scalar_identity", 1
        )[0]
        before_window_branch = function.split("if virtualization.enabled", 1)[0]
        compact_function = "".join(function.split())

        self.assertIn("collect_visible_collection_items(rows", compact_function)
        self.assertNotIn(".collect()", before_window_branch)


if __name__ == "__main__":
    unittest.main()
