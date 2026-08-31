from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
POINTER_CONTEXT = ROOT / "zircon_runtime_interface/src/ui/dispatch/pointer/context.rs"
NAVIGATION_CONTEXT = (
    ROOT / "zircon_runtime_interface/src/ui/dispatch/navigation/context.rs"
)
POINTER_DISPATCHER = ROOT / "zircon_runtime/src/ui/dispatch/pointer/dispatcher.rs"
NAVIGATION_DISPATCHER = (
    ROOT / "zircon_runtime/src/ui/dispatch/navigation/dispatcher.rs"
)
SURFACE_ROUTING = ROOT / "zircon_runtime/src/ui/surface/surface/event_routing.rs"
HIT_PATH = ROOT / "zircon_runtime_interface/src/ui/surface/hit.rs"
POINTER_ROUTE = ROOT / "zircon_runtime_interface/src/ui/surface/pointer/route.rs"


def rust_block(source: str, signature: str) -> str:
    start = source.index(signature)
    opening = source.index("{", start)
    depth = 0
    for index in range(opening, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[opening + 1 : index]
    raise AssertionError(f"unterminated Rust block: {signature}")


class RuntimeUiDispatchRouteSharingPerformanceContractTests(unittest.TestCase):
    def test_dispatch_contexts_borrow_the_event_lifetime_route(self) -> None:
        pointer = POINTER_CONTEXT.read_text(encoding="utf-8")
        navigation = NAVIGATION_CONTEXT.read_text(encoding="utf-8")

        self.assertIn("pub struct UiPointerDispatchContext<'route>", pointer)
        self.assertIn("pub route: &'route UiPointerRoute", pointer)
        self.assertNotIn("Serialize", pointer)
        self.assertNotIn("Deserialize", pointer)

        self.assertIn("pub struct UiNavigationDispatchContext<'route>", navigation)
        self.assertIn("pub route: &'route UiNavigationRoute", navigation)
        self.assertNotIn("Serialize", navigation)
        self.assertNotIn("Deserialize", navigation)

    def test_handlers_accept_any_route_lifetime(self) -> None:
        pointer = POINTER_DISPATCHER.read_text(encoding="utf-8")
        navigation = NAVIGATION_DISPATCHER.read_text(encoding="utf-8")

        self.assertIn("for<'route> Fn(&UiPointerDispatchContext<'route>)", pointer)
        self.assertIn("for<'route> Fn(&UiNavigationDispatchContext<'route>)", navigation)

    def test_dispatchers_never_deep_clone_the_route(self) -> None:
        pointer = POINTER_DISPATCHER.read_text(encoding="utf-8")
        navigation = NAVIGATION_DISPATCHER.read_text(encoding="utf-8")

        self.assertNotIn("route.clone()", pointer)
        self.assertNotIn("route.clone()", navigation)
        self.assertIn("finish(route)", pointer)
        self.assertIn("UiNavigationDispatchResult::new(route)", navigation)

    def test_surface_moves_pointer_route_into_the_result_authority(self) -> None:
        source = SURFACE_ROUTING.read_text(encoding="utf-8")
        body = source.split("fn dispatch_pointer_event_with_query_and_modifiers(", 1)[1].split(
            "fn route_pointer_event_with_details(", 1
        )[0]

        self.assertIn("dispatch_surface_route(&self.tree, route)", body)
        self.assertNotIn("dispatch_surface_route(&self.tree, &route)", body)
        self.assertIn("&result.route", body)

    def test_hit_and_dispatch_paths_have_one_canonical_sequence_each(self) -> None:
        hit_path = HIT_PATH.read_text(encoding="utf-8")
        pointer_route = POINTER_ROUTE.read_text(encoding="utf-8")

        hit_path_body = hit_path.split("pub struct UiHitPath", 1)[1].split(
            "impl UiHitPath", 1
        )[0]
        self.assertIn("pub root_to_leaf: Vec<UiNodeId>", hit_path_body)
        self.assertNotIn("pub bubble_route: Vec<UiNodeId>", hit_path_body)
        self.assertIn("pub fn bubble_route(", hit_path)

        self.assertIn("enum UiPointerRoutingPath", pointer_route)
        self.assertIn("HitPath", pointer_route)
        self.assertIn("ExplicitRootToLeaf", pointer_route)
        self.assertIn("pub fn routing_root_to_leaf(&self)", pointer_route)
        self.assertIn("pub fn bubble_route(", pointer_route)
        self.assertIn("enum UiPointerHitCandidates", pointer_route)
        self.assertNotIn(".filter(move |_| uses_stacked_hits)", pointer_route)

    def test_surface_reuses_the_physical_hit_path_for_uncaptured_routing(self) -> None:
        source = SURFACE_ROUTING.read_text(encoding="utf-8")
        body = source.split("fn route_pointer_event_with_details(", 1)[1].split(
            "pub fn route_navigation_event(", 1
        )[0]

        self.assertIn("UiPointerRoutingPath::HitPath", body)
        self.assertIn("UiPointerRoutingPath::from_bubble_route", body)
        self.assertNotIn("hit.path.bubble_route.clone()", body)

    def test_manual_pointer_route_serde_keeps_defaults_on_the_wire_projection(self) -> None:
        source = POINTER_ROUTE.read_text(encoding="utf-8")
        pointer_route = rust_block(source, "pub struct UiPointerRoute")
        wire_route = rust_block(source, "struct WirePointerRoute")

        self.assertNotIn("#[serde(", pointer_route)
        self.assertIn("impl Serialize for UiPointerRoute", source)
        self.assertIn("impl<'de> Deserialize<'de> for UiPointerRoute", source)
        for field in (
            "modifiers: UiInputModifiers",
            "activation_phase: UiPointerActivationPhase",
            "hit_path: UiHitPath",
            "pressed: Option<UiNodeId>",
            "click_target: Option<UiNodeId>",
            "release_inside_pressed: bool",
        ):
            self.assertIn(f"#[serde(default)]\n            {field}", wire_route)


if __name__ == "__main__":
    unittest.main()
