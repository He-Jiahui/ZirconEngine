from pathlib import Path
import re
import unittest


ROOT = Path(__file__).resolve().parents[2]
RESULT = ROOT / "zircon_runtime_interface/src/ui/dispatch/input/result.rs"
POINTER_ROUTE = ROOT / "zircon_runtime_interface/src/ui/surface/pointer/route.rs"
POINTER_INPUT = ROOT / "zircon_runtime/src/ui/surface/input/pointer.rs"
ROUTE_POLICY = ROOT / "zircon_runtime/src/ui/surface/input/route_policy.rs"
DIAGNOSTICS_BUDGET = ROOT / "zircon_runtime/src/ui/surface/input/diagnostics_budget.rs"
INPUT_DISPATCH = ROOT / "zircon_runtime/src/ui/surface/input/dispatch.rs"
WINDOW_PUMP = ROOT / "zircon_runtime/src/ui/surface/input/window_pump.rs"
TEXT_POINTER = ROOT / "zircon_runtime/src/ui/surface/input/text_pointer.rs"
RICH_LINK = ROOT / "zircon_runtime/src/ui/surface/input/rich_link.rs"
MOUSE_MOTION = ROOT / "zircon_runtime/src/ui/surface/input/mouse_motion.rs"
ANALOG_INPUT = ROOT / "zircon_runtime/src/ui/surface/input/analog.rs"
NAVIGATION_INPUT = ROOT / "zircon_runtime/src/ui/surface/input/navigation.rs"
OWNER_ROUTE = ROOT / "zircon_runtime/src/ui/surface/input/owner_route.rs"
INPUT_MANAGER = ROOT / "zircon_runtime/src/ui/dispatch/input_manager/manager.rs"
POINTER_TABLE = ROOT / "zircon_runtime/src/ui/dispatch/input_manager/pointer_table.rs"
DYNAMIC_RUNTIME_UI = ROOT / "zircon_runtime/src/dynamic_api/session/runtime_ui.rs"
EDITOR_SHELL = ROOT / "zircon_editor/src/ui/retained_host/shell_pointer/bridge.rs"
SURFACE_ROUTING = ROOT / "zircon_runtime/src/ui/surface/surface/event_routing.rs"


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


class RuntimeUiInputRoutingReceiptContractTests(unittest.TestCase):
    def test_pointer_receipt_is_behavioral_state_outside_diagnostics(self) -> None:
        source = RESULT.read_text(encoding="utf-8")
        receipt = rust_block(source, "pub struct UiPointerRoutingReceipt")
        result = rust_block(source, "pub struct UiInputDispatchResult")
        diagnostics = rust_block(source, "pub struct UiInputDispatchDiagnostics")

        self.assertIn("pub route_target: Option<UiNodeId>", receipt)
        self.assertIn("pub capture_target: Option<UiNodeId>", receipt)
        self.assertIn("pub physical_hit_path: UiHitPath", receipt)
        self.assertIn("pub dispatch_path: UiPointerRoutingPath", receipt)
        self.assertIn(
            "pub pointer_routing: Option<UiPointerRoutingReceipt>", result
        )
        self.assertNotIn("UiPointerRoutingReceipt", diagnostics)

    def test_receipt_reuses_physical_path_for_ordinary_dispatch(self) -> None:
        source = RESULT.read_text(encoding="utf-8")
        receipt_impl = rust_block(source, "impl UiPointerRoutingReceipt")
        route = POINTER_ROUTE.read_text(encoding="utf-8")

        self.assertIn("pub fn physical_root_to_leaf(&self)", receipt_impl)
        self.assertIn("pub fn dispatch_root_to_leaf(&self)", receipt_impl)
        self.assertIn(
            "self.dispatch_path.root_to_leaf(&self.physical_hit_path)", receipt_impl
        )
        self.assertIn("UiPointerRoutingPath::HitPath", route)
        self.assertIn("UiPointerRoutingPath::ExplicitRootToLeaf", route)

    def test_summary_mode_records_receipt_before_skipping_full_trace(self) -> None:
        result_source = RESULT.read_text(encoding="utf-8")
        policy_source = ROUTE_POLICY.read_text(encoding="utf-8")
        pointer_source = POINTER_INPUT.read_text(encoding="utf-8")
        annotation = rust_block(policy_source, "pub(super) fn annotate_pointer_route_trace")

        self.assertIn("pub enum UiInputDiagnosticsMode", result_source)
        self.assertIn("Summary", result_source)
        self.assertIn("Full", result_source)
        receipt_assignment = annotation.index("result.pointer_routing = Some(receipt)")
        full_gate = annotation.index("if !diagnostics_mode.captures_full_trace()")
        trace_assignment = annotation.index("diagnostics.route_trace")
        self.assertLess(receipt_assignment, full_gate)
        self.assertLess(full_gate, trace_assignment)
        self.assertIn("diagnostics_mode: UiInputDiagnosticsMode", pointer_source)
        self.assertIn("diagnostics_mode.captures_full_trace()", pointer_source)

    def test_active_pointer_behavior_uses_physical_receipt_not_diagnostics(self) -> None:
        manager = INPUT_MANAGER.read_text(encoding="utf-8")
        table = POINTER_TABLE.read_text(encoding="utf-8")
        update = rust_block(manager, "fn update_active_pointer_table")

        self.assertIn("result.pointer_routing.as_ref()", update)
        self.assertIn("routing.physical_bubble_route()", update)
        self.assertIn("routing.route_target", update)
        self.assertIn("routing.capture_target", update)
        self.assertNotIn("result.diagnostics", update)
        self.assertIn("pub fn set_hovered_path_iter", table)

    def test_product_pointer_paths_select_summary_and_consume_receipts(self) -> None:
        dynamic = DYNAMIC_RUNTIME_UI.read_text(encoding="utf-8")
        dynamic_product = dynamic.split("#[cfg(test)]", 1)[0]
        editor = EDITOR_SHELL.read_text(encoding="utf-8")
        surface = SURFACE_ROUTING.read_text(encoding="utf-8")

        self.assertNotIn("input: UiInputManager::default()", dynamic_product)
        self.assertIn("input: UiInputManager::summary()", dynamic_product)
        self.assertRegex(
            dynamic,
            re.compile(r"result\s*\.pointer_routing\s*\.as_ref\(\)", re.MULTILINE),
        )
        self.assertNotIn(
            "result.diagnostics.route_trace.capture_target.is_some()", dynamic
        )
        self.assertGreaterEqual(
            editor.count("dispatch_input_event_with_diagnostics_mode"), 2
        )
        self.assertIn("UiInputDiagnosticsMode::Summary", editor)
        self.assertRegex(
            editor,
            re.compile(r"result\s*\.pointer_routing\s*\.as_ref\(\)", re.MULTILINE),
        )
        self.assertNotIn(".or(result.diagnostics.route_target)", editor)
        self.assertIn("pub fn dispatch_input_event_with_diagnostics_mode", surface)

    def test_full_diagnostics_have_source_level_hard_budgets_and_drop_receipt(self) -> None:
        result_source = RESULT.read_text(encoding="utf-8")
        budget = DIAGNOSTICS_BUDGET.read_text(encoding="utf-8")
        dispatch = INPUT_DISPATCH.read_text(encoding="utf-8")
        policy = ROUTE_POLICY.read_text(encoding="utf-8")
        diagnostics = rust_block(result_source, "pub struct UiInputDispatchDiagnostics")
        truncation = rust_block(
            result_source, "pub struct UiInputDiagnosticsTruncationReceipt"
        )

        for field in (
            "route_nodes_dropped",
            "route_steps_dropped",
            "notes_dropped",
            "popup_entries_dropped",
            "string_bytes_dropped",
        ):
            self.assertIn(f"pub {field}: u64", truncation)
        self.assertIn("pub truncation: UiInputDiagnosticsTruncationReceipt", diagnostics)
        for limit in (
            "MAX_ROUTE_NODES_PER_PATH",
            "MAX_ROUTE_STEPS",
            "MAX_NOTES",
            "MAX_POPUP_ENTRIES",
            "MAX_STRING_BYTES",
        ):
            self.assertIn(f"const {limit}", budget)
        self.assertIn("bounded_node_path", policy)
        self.assertIn("bounded_popup_stack", policy)
        self.assertNotIn("receipt.dispatch_root_to_leaf().to_vec()", policy)
        self.assertNotIn("receipt.dispatch_bubble_route().collect()", policy)
        self.assertIn("enforce_diagnostics_budget(&mut result)", dispatch)
        self.assertIn("&mut diagnostics.route_steps", budget)
        self.assertIn("&mut diagnostics.notes", budget)
        self.assertIn("&mut diagnostics.route_trace.popup_stack", budget)
        self.assertIn("values.truncate(limit)", budget)

    def test_summary_pointer_paths_gate_diagnostic_string_construction(self) -> None:
        budget = DIAGNOSTICS_BUDGET.read_text(encoding="utf-8")
        dispatch = rust_block(
            INPUT_DISPATCH.read_text(encoding="utf-8"), "pub(crate) fn dispatch_input_event"
        )
        pointer = rust_block(
            POINTER_INPUT.read_text(encoding="utf-8"),
            "pub(super) fn dispatch_pointer_input",
        )
        window = WINDOW_PUMP.read_text(encoding="utf-8")
        mark_window = rust_block(window, "fn mark_window_event_result")
        optional_window = rust_block(window, "fn mark_optional_window_event_result")
        append_components = rust_block(window, "fn append_component_events")
        text_pointer_source = TEXT_POINTER.read_text(encoding="utf-8")
        text_pointer = rust_block(
            text_pointer_source,
            "pub(super) fn dispatch_pointer_text_edit",
        )
        rich_link_source = RICH_LINK.read_text(encoding="utf-8")
        rich_link = rust_block(
            rich_link_source,
            "pub(super) fn dispatch_pointer_rich_link_activation",
        )

        self.assertIn(
            "if diagnostics_mode.captures_full_trace() {\n"
            "        annotate_authoritative_input_dispatch(&mut result);\n"
            "        enforce_diagnostics_budget(&mut result);\n"
            "    } else if diagnostics_budget_required(&result) {\n"
            "        enforce_diagnostics_budget(&mut result);\n"
            "    }",
            dispatch,
        )
        self.assertIn("pub(super) fn diagnostics_budget_required", budget)
        budget_required = rust_block(
            budget, "pub(super) fn diagnostics_budget_required"
        )
        self.assertNotIn(".iter()", budget_required)
        self.assertNotIn("for ", budget_required)
        self.assertIn(
            "if diagnostics_mode.captures_full_trace() {\n"
            "        result.diagnostics.handled_phase",
            pointer,
        )
        for body in (mark_window, optional_window, append_components):
            self.assertIn("diagnostics_mode: UiInputDiagnosticsMode", window)
            self.assertIn("diagnostics_mode.captures_full_trace()", body)
        self.assertIn("diagnostics_mode: UiInputDiagnosticsMode", text_pointer_source)
        self.assertIn("diagnostics_mode.captures_full_trace()", text_pointer)
        self.assertIn("diagnostics_mode: UiInputDiagnosticsMode", rich_link_source)
        self.assertIn("diagnostics_mode.captures_full_trace()", rich_link)

    def test_summary_raw_mouse_motion_skips_diagnostic_string_construction(self) -> None:
        dispatch = rust_block(
            INPUT_DISPATCH.read_text(encoding="utf-8"), "pub(crate) fn dispatch_input_event"
        )
        motion_source = MOUSE_MOTION.read_text(encoding="utf-8")
        motion = rust_block(motion_source, "pub(super) fn dispatch_mouse_motion_input")

        self.assertIn(
            "dispatch_mouse_motion_input(surface, motion, diagnostics_mode)", dispatch
        )
        self.assertIn("diagnostics_mode: UiInputDiagnosticsMode", motion_source)
        full_gate = motion.index("if diagnostics_mode.captures_full_trace()")
        note = motion.index('"raw_mouse_motion".to_string()')
        self.assertLess(full_gate, note)

    def test_summary_analog_navigation_gates_trace_and_string_construction(self) -> None:
        dispatch = rust_block(
            INPUT_DISPATCH.read_text(encoding="utf-8"), "pub(crate) fn dispatch_input_event"
        )
        analog_source = ANALOG_INPUT.read_text(encoding="utf-8")
        analog = rust_block(analog_source, "pub(super) fn dispatch_analog_input")
        analog_policy = rust_block(analog_source, "fn with_analog_route_policy")
        navigation_source = NAVIGATION_INPUT.read_text(encoding="utf-8")
        navigation = rust_block(
            navigation_source, "pub(super) fn dispatch_navigation_input"
        )
        owner = OWNER_ROUTE.read_text(encoding="utf-8")
        owner_mode = rust_block(
            owner, "pub(super) fn owner_routed_result_with_diagnostics_mode"
        )

        self.assertGreaterEqual(dispatch.count("move |surface, dispatcher, navigation|"), 2)
        self.assertIn("diagnostics_mode: UiInputDiagnosticsMode", analog_source)
        self.assertIn("diagnostics_mode.captures_full_trace()", analog)
        self.assertIn("diagnostics_mode.captures_full_trace()", analog_policy)
        self.assertIn("diagnostics_mode: UiInputDiagnosticsMode", navigation_source)
        self.assertIn("diagnostics_mode.captures_full_trace()", navigation)
        self.assertIn("diagnostics_mode: UiInputDiagnosticsMode", owner)
        self.assertIn("diagnostics_mode.captures_full_trace()", owner_mode)
        self.assertLess(
            navigation.index("diagnostics_mode.captures_full_trace()"),
            navigation.index("annotate_navigation_route_trace"),
        )


if __name__ == "__main__":
    unittest.main()
