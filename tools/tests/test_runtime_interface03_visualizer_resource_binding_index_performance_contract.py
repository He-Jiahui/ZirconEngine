import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
VISUALIZER_SOURCE = (
    ROOT / "zircon_runtime_interface/src/ui/surface/render/visualizer.rs"
)
INDEX_SOURCE = (
    ROOT
    / "zircon_runtime_interface/src/ui/surface/render/visualizer/resource_binding_index.rs"
)


def function_body(source: str, signature: str) -> str:
    start = source.index(signature)
    brace = source.index("{", start)
    depth = 0
    for index in range(brace, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[brace : index + 1]
    raise AssertionError(f"unterminated function body for {signature}")


class RuntimeInterface03VisualizerResourceBindingIndexPerformanceContract(
    unittest.TestCase
):
    def test_visualizer_routes_resource_admission_through_index_module(self) -> None:
        source = VISUALIZER_SOURCE.read_text(encoding="utf-8")
        body = function_body(source, "pub fn from_paint_elements_batches_cache(")

        self.assertIn("mod resource_binding_index;", source)
        self.assertIn("use resource_binding_index::ResourceBindingIndex;", source)
        self.assertIn("let mut resource_bindings = ResourceBindingIndex::default();", body)
        self.assertIn("for_each_paint_resource(element, |resource|", body)
        self.assertIn("resource_bindings.add(resource,", body)
        self.assertNotIn("let mut resource_bindings = Vec::new();", body)
        self.assertNotIn("for resource in paint_resource_keys(element)", body)

    def test_resource_index_uses_id_buckets_and_exact_candidate_match(self) -> None:
        source = INDEX_SOURCE.read_text(encoding="utf-8")

        self.assertIn("HashMap<String, Vec<usize>>", source)
        self.assertIn(
            "self.bindings[candidate_index].resource == *resource",
            source,
        )
        self.assertIn("resource.id.as_str()", source)
        self.assertNotIn("bindings.iter_mut().find", source)
        self.assertIn(
            "RUNTIME_INTERFACE03_VISUALIZER_RESOURCE_BINDING_INDEX_BENCH_V1",
            source,
        )

    def test_borrowed_admission_avoids_temporary_resource_vec_and_fast_paths_order(self) -> None:
        visualizer = VISUALIZER_SOURCE.read_text(encoding="utf-8")
        index = INDEX_SOURCE.read_text(encoding="utf-8")

        self.assertIn("fn for_each_paint_resource(", visualizer)
        self.assertNotIn("fn paint_resource_keys(", visualizer)
        self.assertNotIn("fn push_unique_resource(", visualizer)
        self.assertIn("values.last().copied()", index)
        self.assertIn("values.contains(&value)", index)
        self.assertIn(
            "RUNTIME_INTERFACE03_VISUALIZER_BORROWED_RESOURCE_ADMISSION_BENCH_V1",
            index,
        )


if __name__ == "__main__":
    unittest.main()
