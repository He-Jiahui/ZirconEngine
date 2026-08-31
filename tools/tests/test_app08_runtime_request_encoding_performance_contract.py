from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
REQUEST_ENCODING = (
    ROOT
    / "zircon_app/src/entry/runtime_library/runtime_session/request_encoding.rs"
)
PERFORMANCE_TESTS = (
    ROOT
    / "zircon_app/src/entry/runtime_library/runtime_session/request_encoding/performance_tests.rs"
)


def source_between(source: str, start: str, end: str) -> str:
    offset = source.index(start)
    return source[offset : source.index(end, offset)]


class App08RuntimeRequestEncodingPerformanceContractTests(unittest.TestCase):
    def test_writer_preallocates_only_the_bounded_common_request_capacity(self) -> None:
        source = REQUEST_ENCODING.read_text(encoding="utf-8")
        constructor = source_between(
            source,
            "fn new(limit: ZrRuntimePayloadLimitV1)",
            "fn finish(",
        )
        compact_constructor = "".join(constructor.split())

        self.assertIn("REQUEST_WRITER_INITIAL_CAPACITY_BYTES: usize = 4 * 1024", source)
        self.assertIn("limit.max_encoded_bytes.min(", compact_constructor)
        self.assertIn("Vec::with_capacity(initial_capacity)", constructor)
        self.assertNotIn("bytes: Vec::new()", constructor)

    def test_writer_amortizes_clock_reads_but_finish_rechecks_deadline(self) -> None:
        source = REQUEST_ENCODING.read_text(encoding="utf-8")
        write_body = source_between(source, "impl Write for RuntimeRequestWriter", "fn flush(")
        finish_body = source_between(source, "fn finish(", "fn check_deadline(")

        self.assertIn("DEADLINE_CHECK_INTERVAL_BYTES: usize = 1024", source)
        self.assertIn("self.check_deadline_if_due()", write_body)
        self.assertNotIn("if let Err(error) = self.check_deadline()", write_body)
        self.assertIn("self.check_deadline()?", finish_body)

    def test_release_benchmark_covers_clock_checks_and_capacity_growth(self) -> None:
        self.assertTrue(PERFORMANCE_TESTS.is_file())
        benchmark = PERFORMANCE_TESTS.read_text(encoding="utf-8")

        self.assertIn("APP08_REQUEST_ENCODING_PERF", benchmark)
        self.assertIn("legacy_deadline_checks_per_encode", benchmark)
        self.assertIn("optimized_deadline_checks_per_encode", benchmark)
        self.assertIn("legacy_capacity_growths_per_encode", benchmark)
        self.assertIn("optimized_capacity_growths_per_encode", benchmark)


if __name__ == "__main__":
    unittest.main()
