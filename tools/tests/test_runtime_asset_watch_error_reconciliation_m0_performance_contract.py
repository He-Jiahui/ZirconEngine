from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
WATCH_LOOP = (
    ROOT / "zircon_runtime" / "src" / "asset" / "watch" / "watch_loop.rs"
)
WATCHER_TESTS = ROOT / "zircon_runtime" / "src" / "asset" / "tests" / "watcher.rs"


class AssetWatchErrorReconciliationPerformanceContractTests(unittest.TestCase):
    def test_provider_error_enters_the_debounced_reconciliation_window(self) -> None:
        source = WATCH_LOOP.read_text(encoding="utf-8")
        error_branch = source.split("Err(error) =>", 1)[1].split(
            "\n                    }\n                }\n                Err(_) =>", 1
        )[0]

        self.assertIn("on_error(", error_branch)
        self.assertIn("requires_reconciliation = true;", error_branch)
        self.assertIn("started_at.get_or_insert(message.received_at);", error_branch)
        self.assertIn("last_event_at = Some(Instant::now());", error_branch)
        self.assertNotIn("prepare_watch_file_generation", error_branch)
        self.assertNotIn("process_watch_batch_in_generation", error_branch)

    def test_behavior_contract_requires_error_and_reconciliation(self) -> None:
        tests = WATCHER_TESTS.read_text(encoding="utf-8")

        self.assertIn(
            "fn watcher_failure_on_removed_directory_surfaces_observable_error()",
            tests,
        )
        self.assertIn("assert!(batch.changes.is_empty());", tests)
        self.assertIn("assert!(batch.requires_reconciliation);", tests)


if __name__ == "__main__":
    unittest.main()
