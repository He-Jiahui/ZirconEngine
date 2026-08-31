from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
TARGETS = ROOT / "zircon_runtime/src/ui/surface/binding_targets.rs"


def function_body(source: str, name: str, next_name: str) -> str:
    start = source.index(f"    fn {name}(")
    return source[start : source.index(f"    fn {next_name}(", start)]


class RuntimeUiBindingTargetPassthroughPerformanceContractTests(unittest.TestCase):
    def test_empty_target_events_bypass_receipt_and_string_work(self) -> None:
        source = TARGETS.read_text(encoding="utf-8")
        body = function_body(
            source,
            "apply_pointer_binding_target_event",
            "binding_target_event_profile",
        )

        passthrough = body.index("binding_target_event_profile(event)")
        timer = body.index("Instant::now()")
        self.assertLess(passthrough, timer)
        self.assertNotIn("event.binding_id.clone()", body)
        self.assertNotIn(".to_string()", body)
        self.assertNotIn(".binding(handle)", body)
        self.assertIn("ui.binding.target_passthrough_count", body)

    def test_profile_authority_keeps_stale_and_targeted_endpoints_on_the_diagnostic_path(self) -> None:
        source = TARGETS.read_text(encoding="utf-8")
        body = function_body(
            source,
            "binding_target_event_profile",
            "apply_pointer_binding_target_event_inner",
        )

        self.assertIn("Some(handle)", body)
        self.assertIn("binding.targets.is_empty()", body)
        self.assertIn("self.compiled_binding_matches_event(binding, event)", body)
        self.assertIn(".raw_event_has_targets(event)", body)
        self.assertEqual(body.count(".binding(handle)"), 1)
        self.assertNotIn("unwrap_or(true)", body)


if __name__ == "__main__":
    unittest.main()
