from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    ROOT
    / "zircon_runtime/src/plugin/native_plugin_loader/discovery_refresh/ticket.rs"
)


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
                return source[opening + 1 : index]
    raise AssertionError(f"unterminated function: {signature}")


class BoundedObserverQueueCapacityPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.helper = function_body(
            cls.source,
            "fn reserve_remaining_observer_budget_if_full(",
        )
        cls.on_terminal = function_body(cls.source, "pub fn on_terminal(")

    def test_full_non_empty_queue_reserves_remaining_budget_exactly(self) -> None:
        self.assertIn("!observers.is_empty()", self.helper)
        self.assertIn("observers.len() == observers.capacity()", self.helper)
        self.assertIn("max_observers.saturating_sub(observers.len())", self.helper)
        self.assertIn("observers.reserve_exact(remaining_capacity)", self.helper)

    def test_small_empty_queue_keeps_lazy_vec_allocation(self) -> None:
        self.assertNotIn("Vec::with_capacity", self.helper)
        self.assertNotIn("observers.reserve_exact(max_observers)", self.helper)

    def test_terminal_admission_reserves_before_queue_push(self) -> None:
        reserve = self.on_terminal.index(
            "reserve_remaining_observer_budget_if_full("
        )
        push = self.on_terminal.index("state.observers.push(observer)")
        self.assertLess(reserve, push)
        self.assertIn("self.inner.max_observers", self.on_terminal[reserve:push])

    def test_rust_regression_covers_small_and_saturated_queues(self) -> None:
        self.assertIn(
            "observer_queue_reservation_preserves_small_fan_out_and_jumps_to_budget",
            self.source,
        )
        self.assertIn("SATURATED_OBSERVER_BUDGET", self.source)


if __name__ == "__main__":
    unittest.main()
