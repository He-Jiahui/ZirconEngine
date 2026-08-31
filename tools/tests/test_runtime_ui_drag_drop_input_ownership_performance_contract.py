from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
DRAG_DROP = ROOT / "zircon_runtime/src/ui/surface/input/drag_drop.rs"
DISPATCH = ROOT / "zircon_runtime/src/ui/surface/input/dispatch.rs"
EFFECT = ROOT / "zircon_runtime/src/ui/surface/input/effect.rs"
TRANSACTION = ROOT / "zircon_runtime/src/ui/surface/input/effect/transaction.rs"
STATE = ROOT / "zircon_runtime/src/ui/surface/input/state/drag_drop.rs"
INPUT_EVENT = ROOT / "zircon_runtime_interface/src/ui/dispatch/input/event.rs"
INPUT_EFFECT = ROOT / "zircon_runtime_interface/src/ui/dispatch/input/effect.rs"
NORMALIZATION = ROOT / "zircon_runtime_interface/src/ui/window/input/normalization.rs"


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


class RuntimeUiDragDropInputOwnershipPerformanceContractTests(unittest.TestCase):
    def test_payload_authority_is_shared_after_platform_normalization(self) -> None:
        input_event = INPUT_EVENT.read_text(encoding="utf-8")
        input_effect = INPUT_EFFECT.read_text(encoding="utf-8")
        state = STATE.read_text(encoding="utf-8")
        normalization = NORMALIZATION.read_text(encoding="utf-8")

        self.assertIn("use std::sync::Arc;", input_event)
        self.assertIn("pub payload: Option<Arc<UiDragPayload>>", input_event)
        self.assertIn("use std::sync::Arc;", input_effect)
        self.assertIn("payload: Option<Arc<UiDragPayload>>", input_effect)
        self.assertIn("use std::sync::Arc;", state)
        self.assertIn("pub payload: Option<Arc<UiDragPayload>>", state)
        self.assertIn("payload: payload.map(Arc::new)", normalization)

    def test_dispatch_moves_the_input_event_and_forwards_diagnostics_mode(self) -> None:
        drag_drop = DRAG_DROP.read_text(encoding="utf-8")
        dispatch = DISPATCH.read_text(encoding="utf-8")
        body = rust_block(drag_drop, "pub(super) fn dispatch_drag_drop_input")

        self.assertIn("diagnostics_mode: UiInputDiagnosticsMode", drag_drop)
        self.assertNotIn("drag_drop.clone()", body)
        self.assertIn("UiInputEvent::DragDrop(drag_drop)", body)
        self.assertIn("payload: drag_payload", body)
        self.assertIn(
            "dispatch_drag_drop_input(surface, drag_drop, diagnostics_mode)",
            dispatch,
        )

    def test_summary_mode_skips_drag_drop_route_projection_and_strings(self) -> None:
        source = DRAG_DROP.read_text(encoding="utf-8")
        route_policy = rust_block(source, "fn with_drag_drop_route_policy")

        self.assertIn("if !diagnostics_mode.captures_full_trace()", route_policy)
        self.assertIn("return result;", route_policy)
        self.assertNotIn("result.event.clone()", route_policy)
        self.assertIn("take_owned_drag_drop_input_event", route_policy)
        self.assertGreaterEqual(source.count("diagnostics_mode.captures_full_trace()"), 4)

    def test_effect_application_moves_reply_and_applied_effect_ownership(self) -> None:
        source = EFFECT.read_text(encoding="utf-8")
        core = rust_block(source, "fn apply_dispatch_reply_core")
        apply_effect = rust_block(source, "fn apply_dispatch_effect_at_index")

        self.assertIn("diagnostics_mode: UiInputDiagnosticsMode", source)
        self.assertIn("UiInputDispatchResult::new(event, reply)", core)
        self.assertNotIn("reply.clone()", core)
        self.assertIn("diagnostics_mode.captures_full_trace()", core)
        self.assertIn("effect,", apply_effect)
        self.assertNotIn("effect: effect.clone()", apply_effect)

    def test_steady_target_update_skips_the_full_surface_transaction_snapshot(self) -> None:
        source = TRANSACTION.read_text(encoding="utf-8")
        self.assertIn("fn effect_requires_atomic_snapshot", source)
        snapshot_gate = rust_block(source, "fn effect_requires_atomic_snapshot")

        self.assertIn("UiDragDropEffectKind::Update", snapshot_gate)
        self.assertIn("drag.target == *target", snapshot_gate)
        self.assertIn("drag.pointer_id == *pointer_id", snapshot_gate)
        self.assertIn("session_id.is_none_or", snapshot_gate)
        self.assertIn("!steady_target_update", snapshot_gate)


if __name__ == "__main__":
    unittest.main()
