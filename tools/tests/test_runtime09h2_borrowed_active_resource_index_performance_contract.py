from pathlib import Path
import re
import unittest


ROOT = Path(__file__).resolve().parents[2]
FILTERING_RS = ROOT / (
    "zircon_runtime/src/graphics/pipeline/render_pipeline_asset/"
    "descriptor_filtering.rs"
)


def source() -> str:
    return FILTERING_RS.read_text(encoding="utf-8")


def active_resource_builder() -> str:
    text = source()
    return text.split("fn active_post_process_graph_resources", 1)[1].split(
        "fn remove_resources_without_enabled_provider", 1
    )[0]


class Runtime09H2BorrowedActiveResourceIndexContract(unittest.TestCase):
    def test_active_resource_index_uses_borrowed_hash_keys(self) -> None:
        text = source()

        self.assertIn("collections::HashSet", text)
        self.assertNotIn("BTreeSet", text)
        self.assertRegex(
            text,
            r"fn active_post_process_graph_resources\([^)]*\) -> HashSet<&(?:'\w+ )?str>",
        )

    def test_builder_does_not_clone_resource_names(self) -> None:
        body = active_resource_builder()

        self.assertNotIn(".cloned()", body)
        self.assertGreaterEqual(body.count(".map(String::as_str)"), 3)

    def test_builder_reserves_the_resource_reference_bound(self) -> None:
        body = active_resource_builder()

        self.assertIn("resource_reference_count", body)
        self.assertIn("HashSet::with_capacity(resource_reference_count)", body)

    def test_all_consumers_accept_borrowed_hash_sets(self) -> None:
        text = source()

        self.assertRegex(text, r"OnceCell<HashSet<&(?:'\w+ )?str>>")
        borrowed_parameters = re.findall(
            r"active_resources: &(?:mut )?HashSet<&(?:'\w+ )?str>", text
        )
        self.assertGreaterEqual(len(borrowed_parameters), 2)


if __name__ == "__main__":
    unittest.main()
