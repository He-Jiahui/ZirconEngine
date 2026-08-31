from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime/src/plugin/native_plugin_loader/benchmark_harness.rs"


class StreamingNativeBenchmarkRecordPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.emit_body = cls.source.split("pub(super) fn emit", 1)[1].split(
            "/// Values collected only after", 1
        )[0]

    def test_record_fields_use_streaming_display_wrappers(self) -> None:
        self.assertIn("struct JsonString<'a>(&'a str);", self.source)
        self.assertIn("impl std::fmt::Display for JsonString<'_>", self.source)
        self.assertIn("struct BenchmarkCounterFields<'a>", self.source)
        self.assertIn("struct BenchmarkLatencyFields<'a>", self.source)

    def test_emit_does_not_collect_or_join_counter_strings(self) -> None:
        self.assertNotIn(".collect::<Vec<_>>()", self.emit_body)
        self.assertNotIn('.join(",")', self.emit_body)
        self.assertNotIn("let latency = match latency_sample", self.emit_body)
        self.assertIn("latency_sample.map", self.emit_body)

    def test_exact_json_contract_tests_remain_in_rust(self) -> None:
        self.assertIn("json_string_escapes_dynamic_benchmark_metadata", self.source)
        self.assertIn("streamed_benchmark_fields_preserve_exact_json_bytes", self.source)
        self.assertIn("BenchmarkCounterFields(counters)", self.emit_body)
        self.assertIn("BenchmarkLatencyFields(latency.as_ref())", self.emit_body)


if __name__ == "__main__":
    unittest.main()
