import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
DISPATCHER = ROOT / "zircon_runtime/src/ui/dispatch/navigation/dispatcher.rs"
NAVIGATION = ROOT / "zircon_runtime/src/ui/surface/input/navigation.rs"
ROUTE_POLICY = ROOT / "zircon_runtime/src/ui/surface/input/route_policy.rs"


class RuntimeNavigationDispatchOwnershipPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = DISPATCHER.read_text(encoding="utf-8")
        cls.dispatch = cls.source.split("pub fn dispatch(", 1)[1]
        cls.navigation = NAVIGATION.read_text(encoding="utf-8")
        cls.route_policy = ROUTE_POLICY.read_text(encoding="utf-8")

    def test_candidate_routes_are_borrowed(self) -> None:
        self.assertIn("let candidates: &[UiNodeId]", self.dispatch)
        self.assertIn("&route.bubbled", self.dispatch)
        self.assertIn("&route.root_targets", self.dispatch)
        self.assertNotIn("route.bubbled.clone()", self.dispatch)
        self.assertNotIn("route.root_targets.clone()", self.dispatch)

    def test_visited_membership_is_capacity_sized_hash_set(self) -> None:
        self.assertIn(
            "UiDispatchVisitedNodeSet::with_expected_len(candidates.len())",
            self.dispatch,
        )
        self.assertNotIn("BTreeSet", self.source)

    def test_handler_context_borrows_the_owned_route(self) -> None:
        self.assertEqual(self.dispatch.count("UiNavigationDispatchContext {"), 1)
        self.assertIn("route: &route", self.dispatch)

    def test_consumed_route_moves_without_a_handler_context_copy(self) -> None:
        self.assertIn("UiNavigationDispatchResult::new(route)", self.dispatch)
        self.assertNotIn("route.clone()", self.dispatch)

    def test_effect_is_moved_into_the_invocation(self) -> None:
        self.assertNotIn("effect: effect.clone()", self.dispatch)

    def test_navigation_event_is_not_cloned_for_route_annotation(self) -> None:
        self.assertNotIn("let event = result.event.clone()", self.navigation)
        self.assertIn(
            "annotate_navigation_route_trace(surface, routed_reply.route, &mut result)",
            " ".join(self.navigation.split()),
        )

    def test_navigation_route_vectors_move_into_the_trace(self) -> None:
        annotation = self.route_policy.split("pub(super) fn annotate_navigation_route_trace", 1)[
            1
        ].split("fn populate_generic_route_trace", 1)[0]
        self.assertIn("route: UiNavigationRoute", annotation)
        self.assertIn("let focus_path = route.bubbled.clone()", annotation)
        self.assertIn("bubble_path: route.bubbled", annotation)
        self.assertIn("root_targets: route.root_targets", annotation)
        self.assertNotIn("route.root_targets.clone()", annotation)

    def test_navigation_reply_moves_into_the_input_result(self) -> None:
        self.assertIn("UiInputDispatchResult::new(event, reply)", self.navigation)
        self.assertIn("Vec::with_capacity(reply.effects.len())", self.navigation)


if __name__ == "__main__":
    unittest.main()
