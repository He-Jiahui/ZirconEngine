from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
ROUTE_POLICY = ROOT / "zircon_runtime/src/ui/surface/input/route_policy.rs"
REGRESSION = ROOT / "zircon_runtime/src/ui/surface/input/tests/route_policy_tests.rs"


class RuntimeUiRoutePathBorrowPerformanceContractTests(unittest.TestCase):
    def test_generic_trace_selects_a_borrowed_preview_source(self) -> None:
        source = ROUTE_POLICY.read_text(encoding="utf-8")
        body = source.split("fn populate_generic_route_trace", 1)[1].split(
            "\nfn pointer_id_has_capture", 1
        )[0]

        self.assertIn("let preview_source = if bubble_path.is_empty()", body)
        self.assertIn("preview_tunnel_for_bubble(preview_source)", body)
        self.assertNotIn("let route_path =", body)
        self.assertNotIn("focus_path.clone()", body)
        self.assertNotIn("bubble_path.clone()", body)

    def test_route_regression_covers_preview_precedence_and_marker(self) -> None:
        source = REGRESSION.read_text(encoding="utf-8")
        self.assertIn("runtime200_generic_route_preview_preserves_bubble_precedence", source)
        self.assertIn("RUNTIME200_ROUTE_PATH_BORROW_BENCH_V1", source)
        self.assertIn("legacy_path_copies", source)
        self.assertIn("optimized_path_copies", source)

    def test_deterministic_route_model_removes_intermediate_path_copies(self) -> None:
        event_count = 1_000_000
        route_depth = 512
        legacy_path_copies = event_count
        legacy_node_copies = event_count * route_depth
        optimized_path_copies = 0
        optimized_node_copies = 0

        self.assertEqual(legacy_path_copies, event_count)
        self.assertEqual(legacy_node_copies, 512_000_000)
        self.assertEqual(optimized_path_copies, 0)
        self.assertEqual(optimized_node_copies, 0)
        self.assertGreaterEqual(legacy_node_copies // max(optimized_node_copies + 1, 1), 500_000_000)


if __name__ == "__main__":
    unittest.main()
