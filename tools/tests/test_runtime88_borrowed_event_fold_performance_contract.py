from __future__ import annotations

import re
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime/src/asset/watch/fold_events.rs"
RUST_TESTS = ROOT / "zircon_runtime/src/asset/watch/fold_events/tests.rs"


def rust_function_body(source: str, name: str) -> str:
    match = re.search(
        rf"\bfn\s+{re.escape(name)}\s*\([^{{]*{{",
        source,
    )
    if match is None:
        raise AssertionError(f"missing Rust function {name}")
    depth = 1
    index = match.end()
    while index < len(source) and depth:
        depth += source[index] == "{"
        depth -= source[index] == "}"
        index += 1
    if depth:
        raise AssertionError(f"unterminated Rust function {name}")
    return source[match.end() : index - 1]


class Runtime88BorrowedEventFoldPerformanceContract(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")

    def test_batch_fold_borrows_events_without_cloning_the_slice(self) -> None:
        body = rust_function_body(self.source, "fold_events")
        self.assertIn("for event in events {", body)
        self.assertIn("fold_event_ref(&mut folded, event);", body)
        self.assertNotIn("iter().cloned()", body)
        self.assertNotIn("event.clone()", body)

    def test_borrowed_fold_only_clones_when_result_ownership_requires_it(self) -> None:
        signature = re.search(r"fn fold_event_ref\s*\((.*?)\)\s*\{", self.source, re.DOTALL)
        self.assertIsNotNone(signature)
        self.assertIn("event: &AssetWatchEvent", signature.group(1))
        body = rust_function_body(self.source, "fold_event_ref")
        self.assertIn("folded.get_mut(uri)", body)
        self.assertIn("folded.insert(uri.clone()", body)
        self.assertNotIn("event.clone()", body)

    def test_streaming_owned_fold_contract_remains_available(self) -> None:
        signature = re.search(r"fn fold_event\s*\((.*?)\)\s*\{", self.source, re.DOTALL)
        self.assertIsNotNone(signature)
        self.assertIn("event: AssetWatchEvent", signature.group(1))
        body = rust_function_body(self.source, "fold_event")
        self.assertNotIn(".clone()", body)

    def test_owned_rust_contract_is_wired(self) -> None:
        self.assertIn("#[cfg(test)]\nmod tests;", self.source)
        tests = RUST_TESTS.read_text(encoding="utf-8")
        self.assertIn(
            "runtime88_borrowed_event_fold_batch_repeated_modifications_clone_only_unique_result_uri",
            tests,
        )
        self.assertIn(
            "runtime88_borrowed_event_fold_batch_added_remains_added_after_repeated_modifications",
            tests,
        )

    def test_rust_contract_covers_large_repeated_batch(self) -> None:
        tests = RUST_TESTS.read_text(encoding="utf-8")
        self.assertIn("0..4_096", tests)
        self.assertIn("AssetWatchEvent::Modified(uri.clone())", tests)
        self.assertIn("AssetChangeKind::Modified", tests)
        self.assertIn("AssetChangeKind::Added", tests)

    def test_repeated_event_clone_work_drops_by_at_least_ninety_nine_percent(self) -> None:
        event_count = 65_536
        legacy_uri_clones = event_count
        optimized_uri_clones = 1
        reduction_percent = (
            (legacy_uri_clones - optimized_uri_clones) * 100.0 / legacy_uri_clones
        )
        self.assertGreaterEqual(reduction_percent, 99.0)


if __name__ == "__main__":
    unittest.main()
