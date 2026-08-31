from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]


def source(relative: str) -> str:
    return (ROOT / relative).read_text(encoding="utf-8")


class RuntimeExtractDiagnosticsM0PerformanceContract(unittest.TestCase):
    def record_body(self) -> str:
        extract_stats = source(
            "zircon_runtime/src/dynamic_api/session/extract_stats.rs"
        )
        start = extract_stats.index("pub fn record_diagnostics")
        end = extract_stats.index("pub(super) fn record_frame_extract_stats", start)
        return extract_stats[start:end]

    def test_extract_stats_update_the_diagnostic_store_once(self) -> None:
        body = self.record_body()

        self.assertEqual(body.count("update_diagnostic_store"), 1)

    def test_extract_stats_use_static_metadata_for_all_seven_series(self) -> None:
        body = self.record_body()

        self.assertEqual(body.count("record_static("), 7)

    def test_extract_stats_do_not_lock_through_per_series_runtime_calls(self) -> None:
        body = self.record_body()

        self.assertNotIn("runtime.record_diagnostic", body)


if __name__ == "__main__":
    unittest.main()
