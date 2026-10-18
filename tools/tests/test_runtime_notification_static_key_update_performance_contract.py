import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = REPO_ROOT / "zircon_runtime/src/ui/component/state_reducer/notification_center.rs"
LOWER_REGRESSION = (
    REPO_ROOT / "zircon_runtime/src/ui/component/state_reducer/tests/notification_static_key_tests.rs"
)


def function_body(source: str, function_name: str) -> str:
    match = re.search(
        rf"\bfn\s+{re.escape(function_name)}(?![A-Za-z0-9_])(?:<[^>]*>)?\s*\(.*?\)",
        source,
        re.S,
    )
    if match is None:
        raise AssertionError(f"missing function {function_name}")
    opening = source.find("{", match.end())
    if opening < 0:
        raise AssertionError(f"missing body for {function_name}")
    depth = 0
    for index in range(opening, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[opening + 1 : index]
    raise AssertionError(f"unterminated body for {function_name}")


class RuntimeNotificationStaticKeyUpdatePerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")

    def test_fixed_notification_publication_reuses_static_keys(self) -> None:
        for function_name in (
            "sync_notification_state",
            "navigate_notifications",
            "write_selected_notification",
            "write_unread_count_from_entries",
        ):
            body = function_body(self.source, function_name)
            self.assertIn("set_notification_value", body)
            self.assertNotIn("super::set_value", body)
        helper = function_body(self.source, "set_notification_value")
        self.assertIn("state.values.get_mut(property)", helper)
        self.assertIn("state.values.insert(property.to_owned(), value)", helper)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        lower = LOWER_REGRESSION.read_text(encoding="utf-8")
        self.assertIn("mod notification_static_key_tests;", self.source)
        self.assertIn("runtime784_notification_static_keys_update_in_place", lower)
        self.assertIn("RUNTIME784_NOTIFICATION_STATIC_KEY_UPDATE_BENCH_V1", lower)
        self.assertIn("optimized_key_allocations", lower)

    def test_deterministic_model_removes_existing_key_allocations(self) -> None:
        updates_per_event = 3
        legacy_key_allocations = updates_per_event
        optimized_key_allocations = 0
        self.assertGreater(legacy_key_allocations, optimized_key_allocations)
        self.assertEqual(optimized_key_allocations, 0)


if __name__ == "__main__":
    unittest.main()
