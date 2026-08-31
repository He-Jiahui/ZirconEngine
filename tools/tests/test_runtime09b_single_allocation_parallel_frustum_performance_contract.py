import pathlib
import unittest


REPO_ROOT = pathlib.Path(__file__).resolve().parents[2]
SOURCE_PATH = REPO_ROOT / (
    "zircon_runtime/src/graphics/visibility/culling/parallel_frustum.rs"
)


def production_source() -> str:
    return SOURCE_PATH.read_text(encoding="utf-8").split("#[cfg(test)]", 1)[0]


class SingleAllocationParallelFrustumPerformanceContract(unittest.TestCase):
    def test_parallel_path_maps_source_indices_directly(self) -> None:
        source = production_source()
        self.assertIn("parallel_map_indices", source)
        self.assertIn("TaskPool", source)
        self.assertIn("parallel_map_indices(task_pool, candidates.len(), |index|", source)
        self.assertIn("let candidate = candidates[index];", source)

    def test_parallel_path_builds_final_visibility_entries(self) -> None:
        source = production_source()
        body = source.split("pub(crate) fn mesh_frustum_visibility(", 1)[1]
        body = body.split("pub(crate) fn serial_mesh_frustum_visibility(", 1)[0]
        self.assertIn("MeshFrustumVisibility {", body)
        self.assertIn("stable_instance_key: candidate.stable_instance_key", body)
        self.assertIn("visible: visibility_test.is_visible(candidate.bounds)", body)

    def test_parallel_path_has_no_intermediate_work_item_allocation(self) -> None:
        source = production_source()
        self.assertNotIn("MeshFrustumWorkItem", source)
        self.assertNotIn("let mut work_items", source)
        self.assertNotIn("parallel_for(", source)


if __name__ == "__main__":
    unittest.main()
