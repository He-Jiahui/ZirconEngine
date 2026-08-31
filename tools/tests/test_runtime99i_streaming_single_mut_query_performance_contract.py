import pathlib
import unittest


REPO_ROOT = pathlib.Path(__file__).resolve().parents[2]
SOURCE_PATH = REPO_ROOT / "zircon_runtime/src/scene/ecs/query/query_state/mutable.rs"


def single_mut_body() -> str:
    source = SOURCE_PATH.read_text(encoding="utf-8")
    body = source.split("pub(crate) fn single_mut_with_ticks", 1)[1]
    return body.split("pub(crate) fn get_many_mut_with_ticks", 1)[0]


class StreamingSingleMutQueryPerformanceContract(unittest.TestCase):
    def test_single_mut_streams_stable_locations_without_candidate_projection(self) -> None:
        body = single_mut_body()
        self.assertIn("for stable_location in world.stable_query_location_iter(", body)
        self.assertNotIn("let candidates", body)
        self.assertNotIn("collect::<Vec<_>>()", body)

    def test_single_mut_reuses_one_component_location_scratch(self) -> None:
        body = single_mut_body()
        scratch = "let mut component_locations = Vec::with_capacity(self.access.reads().len())"
        self.assertEqual(body.count(scratch), 1)
        self.assertIn(
            "validate_entity_with_locations(world, entity, ticks, &mut component_locations)",
            body,
        )
        self.assertIn("component_locations.clear()", body)

    def test_mutable_fetch_remains_after_unique_match_selection(self) -> None:
        body = single_mut_body()
        loop_index = body.index("for stable_location in")
        selection_index = body.index("let Some(entity) = matched else")
        fetch_index = body.index("D::fetch_mut_with_component_locations")
        self.assertLess(loop_index, selection_index)
        self.assertLess(selection_index, fetch_index)
        self.assertIn("return Err(QuerySingleError::MultipleEntities)", body)
        self.assertIn("return Err(QuerySingleError::NoEntities)", body)


if __name__ == "__main__":
    unittest.main()
