import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
TASK_POOL = (
    REPO_ROOT
    / "zircon_plugins"
    / "navigation"
    / "runtime"
    / "src"
    / "manager"
    / "bake"
    / "task_pool.rs"
)


def function_body(source: str, function_name: str) -> str:
    match = re.search(rf"\bfn\s+{re.escape(function_name)}\s*\(", source)
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


class TiledBakePlanArcPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = TASK_POOL.read_text(encoding="utf-8")

    def test_pending_state_shares_one_immutable_plan(self) -> None:
        self.assertRegex(
            self.source,
            r"plan:\s*Arc<RecastTiledBakePlan>",
        )

    def test_tile_workers_clone_only_the_outer_arc(self) -> None:
        body = function_body(self.source, "prepare_tiled_task")
        self.assertIn("let plan = Arc::new(plan);", body)
        self.assertIn("let plan = Arc::clone(&plan);", body)
        self.assertNotIn("plan.clone()", body)

    def test_harvest_waits_until_dispatch_releases_its_plan(self) -> None:
        self.assertIn("dispatch_complete: bool", self.source)
        is_ready = function_body(self.source, "is_ready")
        self.assertIn("*dispatch_complete && *completed == results.len()", is_ready)
        prepare = function_body(self.source, "prepare_tiled_task")
        self.assertIn("drop(plan);", prepare)
        self.assertIn("*dispatch_complete = true;", prepare)

    def test_harvest_recovers_the_plan_without_a_deep_clone(self) -> None:
        body = function_body(self.source, "finish_tiled_task")
        self.assertIn("Arc::try_unwrap(plan)", body)
        self.assertNotIn("plan.clone()", body)

    def test_release_evidence_covers_plan_clone_cost(self) -> None:
        self.assertIn(
            "PERF_RESULT plugins14_tiled_bake_shared_plan",
            self.source,
        )
        self.assertIn("legacy_arc_refcount_pairs_per_tile=4", self.source)
        self.assertIn("optimized_arc_refcount_pairs_per_tile=1", self.source)


if __name__ == "__main__":
    unittest.main()
