from pathlib import Path
import re
import unittest


ROOT = Path(__file__).resolve().parents[2]
ROUTES_RS = ROOT / (
    "zircon_plugins/sound/runtime/src/kira_bridge/graph_compile/routes.rs"
)


def source() -> str:
    return ROUTES_RS.read_text(encoding="utf-8")


def compact(text: str) -> str:
    return re.sub(r"\s+", "", text)


def expansion_body() -> str:
    text = source().split("fn expand_track_sends", 1)[1]
    return text.split("#[cfg(test)]", 1)[0]


class Plugins11CachedExpandedRoutesContract(unittest.TestCase):
    def test_recursive_expansion_reports_cache_population_only(self) -> None:
        body = compact(expansion_body())

        self.assertIn(")->Result<(),SoundError>", body)
        self.assertIn("ifcache.contains_key(&track){returnOk(());}", body)

    def test_cache_hits_do_not_clone_expanded_route_vectors(self) -> None:
        body = compact(expansion_body())

        self.assertNotIn("routes.clone()", body)
        self.assertNotIn("returnOk(routes.clone())", body)

    def test_parent_expansion_borrows_cached_downstream_routes(self) -> None:
        body = compact(expansion_body())

        self.assertIn("expand_track_sends(tracks,send.target,cache,visiting)?;", body)
        self.assertIn("letdownstream_routes=cache.get(&send.target)", body)
        self.assertIn("fordownstreamindownstream_routes", body)

    def test_gain_map_reserves_for_direct_and_downstream_routes(self) -> None:
        body = compact(expansion_body())

        self.assertIn("gains.reserve(1+downstream_routes.len())", body)

    def test_completed_routes_move_directly_into_the_cache(self) -> None:
        body = compact(expansion_body())

        self.assertIn("cache.insert(track,routes);Ok(())", body)

    def test_shared_downstream_behavior_has_a_direct_rust_contract(self) -> None:
        self.assertIn("shared_downstream_routes_are_reused_from_cache", source())


if __name__ == "__main__":
    unittest.main()
