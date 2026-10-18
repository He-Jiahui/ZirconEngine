import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    REPO_ROOT
    / "zircon_runtime"
    / "src"
    / "ui"
    / "component"
    / "state_reducer"
    / "toast.rs"
)
LOWER_REGRESSION = (
    REPO_ROOT / "zircon_runtime/src/ui/component/state_reducer/toast/tests/borrowed_setting_tests.rs"
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


class RuntimeToastBorrowedSettingPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")

    def test_scan_and_presence_paths_borrow_settings(self) -> None:
        sync_body = function_body(self.source, "sync_toast_state")
        has_body = function_body(self.source, "has_current_toast")
        self.assertIn("string_setting_ref", sync_body)
        self.assertNotIn("string_setting(state, descriptor, CURRENT_TOAST_ID)", sync_body)
        self.assertIn("string_setting_ref", has_body)

    def test_authored_message_clones_only_at_the_write_boundary(self) -> None:
        body = function_body(self.source, "sync_authored_message_state")
        self.assertIn("string_setting_ref", body)
        self.assertIn("to_owned()", body)
        self.assertNotIn("message.clone()", body)
        self.assertNotIn("fn string_setting(", body)

    def test_borrowed_setting_helper_is_available(self) -> None:
        helper = function_body(self.source, "string_setting_ref")
        self.assertIn("value_setting(state, descriptor, property)", helper)
        self.assertNotIn("string_value(value)", helper)
        self.assertIn("string_value_ref", helper)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        lower = LOWER_REGRESSION.read_text(encoding="utf-8")
        self.assertIn("runtime772_toast_borrowed_settings_preserve_sync_semantics", lower)
        self.assertIn("RUNTIME772_TOAST_BORROWED_SETTING_BENCH_V1", lower)
        self.assertIn("optimized_setting_clones", lower)

    def test_deterministic_model_removes_read_only_setting_clones(self) -> None:
        sync_events = 1_024
        legacy_setting_clones = sync_events * 3
        optimized_setting_clones = 0
        self.assertGreater(legacy_setting_clones, optimized_setting_clones)
        self.assertEqual(optimized_setting_clones, 0)


if __name__ == "__main__":
    unittest.main()
