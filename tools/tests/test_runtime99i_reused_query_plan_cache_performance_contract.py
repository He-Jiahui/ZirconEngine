import pathlib
import unittest


REPO_ROOT = pathlib.Path(__file__).resolve().parents[2]
SOURCE_PATH = REPO_ROOT / "zircon_runtime/src/scene/ecs/query/query_state/cache.rs"


def source() -> str:
    return SOURCE_PATH.read_text(encoding="utf-8")


class ReusedQueryPlanCachePerformanceContract(unittest.TestCase):
    def test_incremental_rebuild_compiles_directly_into_the_retained_cache(self) -> None:
        body = source().split("if new_matches.is_empty()", 1)[1]
        body = body.split("let index_stats_before", 1)[0]
        self.assertIn("cached_archetype_plans.reserve(new_matches.len())", body)
        self.assertIn("for archetype in new_matches", body)
        self.assertIn("cached_archetype_plans.push(compiled_plan)", body)
        self.assertNotIn("compiled_new_plans", body)
        self.assertNotIn(".extend(", body)

    def test_full_rebuild_reuses_cache_capacity(self) -> None:
        body = source().split("let matched_archetypes =", 1)[1]
        body = body.split("pub fn cached_archetype_count", 1)[0]
        self.assertIn("self.cached_archetype_plans.clear()", body)
        self.assertRegex(
            body,
            r"cached_archetype_plans\s*\.reserve\(matched_archetypes\.len\(\)\)",
        )
        self.assertIn("for archetype in matched_archetypes.iter().copied()", body)
        self.assertIn("cached_archetype_plans.push(compiled_plan)", body)
        self.assertNotIn("let compiled_plans", body)
        self.assertNotIn("self.cached_archetype_plans =", body)

    def test_query_plan_compilation_count_remains_one_per_matching_archetype(self) -> None:
        body = source().split("fn compile_archetype_plan(", 1)[1]
        body = body.split("fn record_archetype_index_work(", 1)[0]
        self.assertEqual(body.count("archetype_plan_compilations.saturating_add(1)"), 1)


if __name__ == "__main__":
    unittest.main()
