import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
RENDER_PATH = (
    REPO_ROOT
    / "zircon_runtime"
    / "src"
    / "ui"
    / "surface"
    / "render"
    / "notification_center.rs"
)
REDUCER_PATH = (
    REPO_ROOT
    / "zircon_runtime"
    / "src"
    / "ui"
    / "component"
    / "state_reducer"
    / "notification_center.rs"
)


def function_body(source: str, signature: str) -> str:
    match = re.search(signature + r"[^\{]*\{", source)
    if match is None:
        raise AssertionError(f"missing Rust function: {signature}")
    depth = 1
    index = match.end()
    while index < len(source) and depth:
        depth += source[index] == "{"
        depth -= source[index] == "}"
        index += 1
    if depth:
        raise AssertionError(f"unterminated Rust function: {signature}")
    return source[match.end() : index - 1]


class RuntimeNotificationVisibleMaterializationPerformanceContractTests(unittest.TestCase):
    def test_render_decoder_stops_recursive_materialization_at_visible_limit(self):
        source = RENDER_PATH.read_text(encoding="utf-8")
        rows = function_body(source, r"fn\s+notification_rows")
        collector = function_body(source, r"fn\s+collect_visible_notification_rows")

        self.assertIn("collect_visible_notification_rows(", rows)
        self.assertIn("rows.len() >= visible_limit", collector)
        self.assertNotIn(".into_iter().take(visible_limit)", rows)
        self.assertNotRegex(source, r"flat_map\(notification_entry_list\)")

    def test_keyboard_decoder_uses_the_same_bounded_depth_first_shape(self):
        source = REDUCER_PATH.read_text(encoding="utf-8")
        visible = function_body(source, r"fn\s+visible_notification_entries")
        collector = function_body(source, r"fn\s+collect_visible_notification_entries")

        self.assertIn("collect_visible_notification_entries(", visible)
        self.assertIn("entries.len() >= visible_limit", collector)
        self.assertNotIn("notification_entries(state, descriptor)", visible)
        self.assertNotIn(".into_iter().take(visible_limit)", visible)


if __name__ == "__main__":
    unittest.main()
