from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
POINTER = ROOT / "zircon_runtime/src/ui/surface/input/pointer.rs"
ROUTE_POLICY = ROOT / "zircon_runtime/src/ui/surface/input/route_policy.rs"


def function_body(source: str, name: str) -> str:
    start = source.index(f"fn {name}(")
    boundaries = (
        source.find(marker, start + 1)
        for marker in ("\nfn ", "\npub(super) fn ", "\npub(crate) fn ", "\npub fn ")
    )
    next_functions = [boundary for boundary in boundaries if boundary >= 0]
    return source[start:] if not next_functions else source[start : min(next_functions)]


class RuntimePointerRouteTraceOwnershipPerformanceContractTests(unittest.TestCase):
    def test_pointer_trace_materializer_takes_route_ownership(self) -> None:
        source = ROUTE_POLICY.read_text(encoding="utf-8")
        body = function_body(source, "annotate_pointer_route_trace")

        self.assertIn("route: UiPointerRoute", body)
        self.assertNotIn("route: &UiPointerRoute", body)
        self.assertIn("let UiPointerRoute {", body)
        self.assertIn("physical_hit_path: hit_path", body)
        self.assertIn("dispatch_path: routing_path", body)
        receipt_assignment = body.index("result.pointer_routing = Some(receipt)")
        summary_gate = body.index("if !diagnostics_mode.captures_full_trace()")
        trace_projection = body.index("receipt.dispatch_bubble_route()")
        self.assertLess(receipt_assignment, summary_gate)
        self.assertLess(summary_gate, trace_projection)
        self.assertIn("bounded_node_path", body)
        self.assertIn("root_targets,", body)
        self.assertNotIn("route.bubbled", body)
        self.assertNotIn("route.root_targets.clone()", body)
        self.assertNotIn("route.hit_path.clone()", body)

    def test_pointer_dispatch_moves_completed_route_into_trace_materializer(self) -> None:
        source = POINTER.read_text(encoding="utf-8")
        body = function_body(source, "dispatch_pointer_input")

        self.assertIn("routed_result.route,", body)
        self.assertNotIn("&routed_result.route,\n        &pointer_for_text,", body)

    def test_bounded_focus_trace_uses_the_retained_arranged_node_index(self) -> None:
        source = ROUTE_POLICY.read_text(encoding="utf-8")
        body = function_body(source, "focused_route_bounded")

        self.assertIn("arranged_node_indexed", body)
        self.assertIn("&surface.arranged_node_indices", body)
        self.assertIn("MAX_ROUTE_NODES_PER_PATH", body)
        self.assertNotIn("arranged_focus_path", source)


if __name__ == "__main__":
    unittest.main()
