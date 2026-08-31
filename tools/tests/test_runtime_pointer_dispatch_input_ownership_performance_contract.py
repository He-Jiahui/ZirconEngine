from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
POINTER = ROOT / "zircon_runtime/src/ui/surface/input/pointer.rs"
POINTER_DISPATCHER = ROOT / "zircon_runtime/src/ui/dispatch/pointer/dispatcher.rs"
RICH_LINK = ROOT / "zircon_runtime/src/ui/surface/input/rich_link.rs"
ROUTE_POLICY = ROOT / "zircon_runtime/src/ui/surface/input/route_policy.rs"


def function_body(source: str, name: str) -> str:
    start = source.index(f"fn {name}(")
    boundaries = (
        source.find(marker, start + 1)
        for marker in (
            "\nfn ",
            "\npub(super) fn ",
            "\npub(crate) fn ",
            "\npub fn ",
            "\n#[cfg(",
        )
    )
    next_functions = [boundary for boundary in boundaries if boundary >= 0]
    return source[start:] if not next_functions else source[start : min(next_functions)]


class RuntimePointerDispatchInputOwnershipPerformanceContractTests(unittest.TestCase):
    def test_normal_pointer_dispatch_does_not_clone_input_or_metadata(self) -> None:
        source = POINTER.read_text(encoding="utf-8")
        body = function_body(source, "dispatch_pointer_input")

        self.assertNotIn("pointer.clone()", body)
        self.assertNotIn("pointer.metadata.clone()", body)
        self.assertIn("let pointer_id = pointer.metadata.pointer_id;", body)
        self.assertIn("let pointer_source = pointer.metadata.pointer_source;", body)
        self.assertIn("let click_count = pointer.event.click_count;", body)
        self.assertIn("UiInputEvent::Pointer(pointer)", body)

    def test_optional_text_work_borrows_input_before_the_single_move(self) -> None:
        source = POINTER.read_text(encoding="utf-8")
        body = function_body(source, "dispatch_pointer_input")

        text_dispatch = body.index("dispatch_pointer_text_edit(")
        event_move = body.index("UiInputEvent::Pointer(pointer)")
        self.assertLess(text_dispatch, event_move)

    def test_pointer_reply_is_moved_into_the_result(self) -> None:
        source = POINTER.read_text(encoding="utf-8")
        body = function_body(source, "dispatch_pointer_input")

        self.assertIn("Vec::with_capacity(reply.effects.len())", body)
        self.assertIn("UiInputDispatchResult::new(event, reply)", body)
        self.assertNotIn("UiInputDispatchResult::new(event, reply.clone())", body)

    def test_dispatcher_borrows_one_route_across_route_nodes_without_cloning(self) -> None:
        source = POINTER_DISPATCHER.read_text(encoding="utf-8")
        dispatch = function_body(source, "dispatch_route")
        phase_dispatch = function_body(source, "dispatch_phase_handlers")

        self.assertNotIn("route.clone()", dispatch)
        self.assertIn("&route,", dispatch)
        self.assertIn("route: &UiPointerRoute", phase_dispatch)
        self.assertIn("let context = UiPointerDispatchContext", phase_dispatch)
        self.assertIn("route,", phase_dispatch)
        self.assertNotIn("result.route.clone()", phase_dispatch)

    def test_dispatcher_borrows_candidate_routes_without_vector_clones(self) -> None:
        source = POINTER_DISPATCHER.read_text(encoding="utf-8")
        dispatch = function_body(source, "dispatch_route")

        self.assertIn("let (injected_candidate, candidate_slice)", dispatch)
        self.assertIn("injected_candidate\n            .into_iter()", dispatch)
        self.assertIn(".chain(candidate_slice.iter().copied())", dispatch)
        self.assertNotIn("route.stacked.clone()", dispatch)
        self.assertNotIn("route.root_targets.clone()", dispatch)
        self.assertNotIn("vec![captured]", dispatch)

    def test_dispatcher_uses_unordered_membership_for_visited_nodes(self) -> None:
        source = POINTER_DISPATCHER.read_text(encoding="utf-8")
        dispatch = function_body(source, "dispatch_route")

        self.assertIn(
            "UiDispatchVisitedNodeSet::with_expected_len(expected_visited_len)",
            dispatch,
        )
        self.assertNotIn("BTreeSet", source)

    def test_downstream_helpers_accept_only_copyable_pointer_facts(self) -> None:
        rich_link = function_body(
            RICH_LINK.read_text(encoding="utf-8"),
            "dispatch_pointer_rich_link_activation",
        )
        trace = function_body(
            ROUTE_POLICY.read_text(encoding="utf-8"),
            "annotate_pointer_route_trace",
        )

        self.assertIn("click_count: u8", rich_link)
        self.assertNotIn("pointer: &UiPointerInputEvent", rich_link)
        self.assertIn("pointer_source: UiPointerSource", trace)
        self.assertIn("pointer_id: Option<UiPointerId>", trace)
        self.assertNotIn("pointer: &UiPointerInputEvent", trace)


if __name__ == "__main__":
    unittest.main()
