import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
STORE = ROOT / "zircon_editor/src/ui/retained_host/app/native_windows/store.rs"


def function_body(source: str, signature: str) -> str:
    start = source.index(signature)
    opening = source.index("{", start)
    depth = 0
    for index in range(opening, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[opening : index + 1]
    raise AssertionError(f"unterminated function body for {signature}")


class Editor874NativeWindowStaleIdentityCapacityPerformanceContract(
    unittest.TestCase
):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = STORE.read_text(encoding="utf-8")

    def test_sync_uses_the_bounded_stale_identity_collector(self) -> None:
        body = function_body(self.source, "fn sync_targets_inner")
        self.assertIn("stale_native_window_ids(&self.windows, &target_ids)", body)
        self.assertNotIn(".filter(|window_id| !target_ids.contains", body)

    def test_collector_reserves_the_window_upper_bound_only_after_a_match(self) -> None:
        body = function_body(self.source, "fn stale_native_window_ids")
        compact = "".join(body.split())
        self.assertIn("letmutstale=Vec::new()", compact)
        self.assertIn("ifstale.is_empty()", compact)
        self.assertIn("stale.reserve(windows.len())", compact)
        self.assertLess(compact.index("ifstale.is_empty()"), compact.index("stale.push("))

    def test_collector_keeps_btree_order_and_clones_only_stale_ids(self) -> None:
        body = function_body(self.source, "fn stale_native_window_ids")
        self.assertIn("for window_id in windows.keys()", body)
        self.assertIn("target_ids.contains(window_id)", body)
        self.assertIn("stale.push(window_id.clone())", body)

    def test_lower_semantics_and_release_marker_are_wired(self) -> None:
        self.assertIn("editor874_stale_native_window_ids", self.source)
        self.assertIn(
            "EDITOR874_NATIVE_WINDOW_STALE_ID_CAPACITY_BENCH_V1",
            self.source,
        )


if __name__ == "__main__":
    unittest.main()
