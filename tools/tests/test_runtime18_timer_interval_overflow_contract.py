"""Static contract guards for Runtime18 timer interval overflow handling."""

from __future__ import annotations

import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
TIMER_SOURCE = REPO_ROOT / "zircon_runtime/src/core/runtime/tasks/timer.rs"
INTERVAL_TESTS = (
    REPO_ROOT / "zircon_runtime/src/core/runtime/tasks/timer/tests/astra_interval_tests.rs"
)


def _function_body(source: str, name: str) -> str:
    match = re.search(rf"\bfn\s+{re.escape(name)}\s*\([^{{]*\)\s*(?:->[^{{]+)?\{{", source)
    if match is None:
        raise AssertionError(f"missing Rust function {name}")
    opening = source.find("{", match.start(), match.end())
    depth = 1
    cursor = opening + 1
    while cursor < len(source) and depth:
        if source[cursor] == "{":
            depth += 1
        elif source[cursor] == "}":
            depth -= 1
        cursor += 1
    if depth:
        raise AssertionError(f"unterminated Rust function {name}")
    return source[opening + 1 : cursor - 1]


class Runtime18TimerIntervalOverflowContractTests(unittest.TestCase):
    def test_first_deadline_overflow_is_typed_and_fail_closed(self) -> None:
        source = TIMER_SOURCE.read_text(encoding="utf-8")
        body = _function_body(source, "schedule_interval")

        self.assertIn(
            "checked_add(interval)\n            .ok_or(CoreError::DeadlineOutOfRange)?",
            body,
        )
        self.assertNotIn("unwrap_or(now)", body)

    def test_recurring_overflow_retires_registration_without_repeat(self) -> None:
        source = TIMER_SOURCE.read_text(encoding="utf-8")
        body = _function_body(source, "next_callbacks")

        self.assertIn("TimerSchedule::Interval(interval)", body)
        self.assertIn("next_deadline.checked_add(interval)", body)
        self.assertIn(
            "registration.cancelled.store(true, Ordering::Release);", body
        )
        self.assertRegex(body, r"registration\.cancelled\.store\([^;]+\);\s*continue;")

    def test_overflow_regression_fixture_is_wired_and_checks_capacity_release(self) -> None:
        timer_source = TIMER_SOURCE.read_text(encoding="utf-8")
        interval_tests = INTERVAL_TESTS.read_text(encoding="utf-8")

        self.assertIn(
            "#[path = \"timer/tests/astra_interval_tests.rs\"]\nmod astra_interval_tests;",
            timer_source,
        )
        self.assertIn("astra_m26_first_deadline_overflow_is_typed", interval_tests)
        self.assertIn("astra_m26_recurring_overflow_retires_without_immediate_repeat", interval_tests)
        self.assertIn("DeadlineOutOfRange", interval_tests)
        self.assertIn("recurring overflow must release its timer slot", interval_tests)
        self.assertIn("callback_count.load", interval_tests)


if __name__ == "__main__":
    unittest.main()
