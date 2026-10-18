import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


class RuntimeUiDispatchHashLookupPerformanceContractTests(unittest.TestCase):
    def test_pointer_dispatcher_uses_hash_lookup_for_exact_handler_keys(self):
        source = (
            ROOT / "zircon_runtime/src/ui/dispatch/pointer/dispatcher.rs"
        ).read_text(encoding="utf-8")
        self.assertIn("use std::collections::HashMap;", source)
        self.assertIn(
            "handlers: HashMap<(UiNodeId, UiPointerEventKind), Vec<PointerHandler>>",
            source,
        )
        self.assertIn(
            "phase_handlers: HashMap<",
            source,
        )
        self.assertIn("self.phase_handlers.get(&(node_id, route.kind, phase))", source)
        self.assertIn("self.handlers.get(&(node_id, route.kind))", source)

    def test_navigation_dispatcher_uses_hash_lookup_for_exact_handler_keys(self):
        source = (
            ROOT / "zircon_runtime/src/ui/dispatch/navigation/dispatcher.rs"
        ).read_text(encoding="utf-8")
        self.assertIn("use std::collections::HashMap;", source)
        self.assertIn(
            "handlers: HashMap<(UiNodeId, UiNavigationEventKind), Vec<NavigationHandler>>",
            source,
        )
        self.assertIn("self.handlers.get(&(node_id, route_kind))", source)

    def test_dispatch_key_enums_are_hashable_without_changing_order_traits(self):
        pointer_kind = (
            ROOT
            / "zircon_runtime_interface/src/ui/surface/pointer/event_kind.rs"
        ).read_text(encoding="utf-8")
        navigation_kind = (
            ROOT
            / "zircon_runtime_interface/src/ui/surface/navigation/event_kind.rs"
        ).read_text(encoding="utf-8")
        dispatch_phase = (
            ROOT / "zircon_runtime_interface/src/ui/dispatch/input/reply.rs"
        ).read_text(encoding="utf-8")
        for source in (pointer_kind, navigation_kind, dispatch_phase):
            derive = source.split("enum ", 1)[0]
            self.assertIn("PartialOrd", derive)
            self.assertIn("Ord", derive)
            self.assertIn("Hash", derive)

    def test_dispatchers_do_not_iterate_handler_maps_for_delivery(self):
        pointer = (
            ROOT / "zircon_runtime/src/ui/dispatch/pointer/dispatcher.rs"
        ).read_text(encoding="utf-8")
        navigation = (
            ROOT / "zircon_runtime/src/ui/dispatch/navigation/dispatcher.rs"
        ).read_text(encoding="utf-8")
        for source in (pointer, navigation):
            implementation = source.split("#[cfg(test)]", 1)[0]
            self.assertNotIn("self.handlers.iter()", implementation)
            self.assertNotIn("self.phase_handlers.iter()", implementation)


if __name__ == "__main__":
    unittest.main()
