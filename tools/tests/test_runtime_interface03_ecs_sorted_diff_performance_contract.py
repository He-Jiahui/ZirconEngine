import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
ECS_SOURCE = ROOT / "zircon_runtime_interface/src/ui/ecs.rs"
DIFF_SOURCE = ROOT / "zircon_runtime_interface/src/ui/ecs/diff.rs"


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


class RuntimeInterface03EcsSortedDiffPerformanceContract(unittest.TestCase):
    def test_snapshot_diff_routes_ordered_nodes_to_linear_helper(self) -> None:
        ecs_source = ECS_SOURCE.read_text(encoding="utf-8")
        body = function_body(ecs_source, "pub fn diff_from(&self, previous: &Self)")

        self.assertIn("mod diff;", ecs_source)
        self.assertIn("projection_changes(&previous.nodes, &self.nodes)", body)
        self.assertNotIn("projection_node_map", body)

    def test_sorted_diff_uses_cursor_scans_with_safe_fallback(self) -> None:
        source = DIFF_SOURCE.read_text(encoding="utf-8")

        self.assertIn("fn nodes_are_strictly_ordered", source)
        self.assertIn("fn ordered_projection_changes", source)
        self.assertIn("fn mapped_projection_changes", source)
        self.assertIn("previous_index += 1", source)
        self.assertIn("current_index += 1", source)
        self.assertIn("projection_node_map", source)
        self.assertIn(
            "RUNTIME_INTERFACE03_LINEAR_SORTED_ECS_DIFF_BENCH_V1",
            source,
        )


if __name__ == "__main__":
    unittest.main()
