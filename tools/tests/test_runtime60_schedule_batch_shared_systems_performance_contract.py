import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
CONFLICT_GRAPH = (
    REPO_ROOT / "zircon_runtime" / "src" / "scene" / "ecs" / "schedule_conflict_graph.rs"
)
EXECUTOR = (
    REPO_ROOT / "zircon_runtime" / "src" / "scene" / "ecs" / "schedule_parallel_executor.rs"
)


def function_body(source: str, function_name: str) -> str:
    match = re.search(
        rf"\bfn\s+{re.escape(function_name)}(?:\s*<[^>]+>)?\s*\(",
        source,
    )
    if match is None:
        raise AssertionError(f"missing function {function_name}")
    opening = source.find("{", match.end())
    if opening < 0:
        raise AssertionError(f"missing body for {function_name}")
    depth = 0
    for index in range(opening, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[opening + 1 : index]
    raise AssertionError(f"unterminated body for {function_name}")


class SharedScheduleBatchSystemsPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.graph = CONFLICT_GRAPH.read_text(encoding="utf-8")
        cls.executor = EXECUTOR.read_text(encoding="utf-8")

    def test_compiled_batch_freezes_shared_system_storage(self) -> None:
        self.assertIn("systems: Arc<ScheduleParallelBatchSystems>", self.graph)
        push = function_body(self.graph, "push_system")
        self.assertIn("Arc::make_mut(&mut self.systems)", push)

    def test_executor_borrows_ids_from_shared_storage(self) -> None:
        run = function_body(self.executor, "run_batches_with_report")
        self.assertIn("let batch_systems = batch.shared_systems();", run)
        self.assertIn("batch_systems.system_ids()", run)
        self.assertNotIn("system_ids.to_vec()", run)

    def test_release_evidence_tracks_heap_allocations(self) -> None:
        self.assertIn(
            "PERF_RESULT runtime60_shared_schedule_batch_systems",
            self.graph,
        )
        self.assertIn("legacy_heap_allocations_per_batch=65", self.graph)
        self.assertIn("optimized_heap_allocations_per_batch=0", self.graph)


if __name__ == "__main__":
    unittest.main()
