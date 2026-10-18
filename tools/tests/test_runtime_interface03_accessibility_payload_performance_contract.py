import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime_interface/src/ui/window/runtime_event_adapter.rs"
BENCH = ROOT / "zircon_runtime_interface/src/ui/window/runtime_event_adapter/tests/accessibility_payload_performance_tests.rs"


class RuntimeInterface03AccessibilityPayloadPerformanceContract(unittest.TestCase):
    def test_accessibility_event_borrows_the_validated_payload(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        start = source.index("fn accessibility_event(")
        end = source.index("\n}\n", start) + len("\n}")
        body = source[start:end]

        self.assertIn("checked_slice", body)
        self.assertIn("ZR_RUNTIME_EVENT_PAYLOAD_MAX_ENCODED_BYTES_V1", body)
        self.assertNotIn("payload_bytes", body)
        self.assertNotIn("to_vec", body)
        self.assertIn("serde_json::from_slice::<UiAccessibilityActionRequest>(payload)", body)

    def test_borrowed_payload_has_behavior_and_release_contract(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        bench = BENCH.read_text(encoding="utf-8")

        self.assertIn("mod accessibility_payload_performance_tests", source)
        self.assertIn("copied_accessibility_payload", bench)
        self.assertIn("borrowed_accessibility_payload", bench)
        self.assertIn("RUNTIME_INTERFACE03_BORROWED_ACCESSIBILITY_PAYLOAD_BENCH_V1", bench)
        self.assertIn("saturating_mul(4)", bench)


if __name__ == "__main__":
    unittest.main()
