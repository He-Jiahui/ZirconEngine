from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
MESSAGE_ID = ROOT / "zircon_hub/src/state/hub_message/id.rs"
MESSAGE = ROOT / "zircon_hub/src/state/hub_message/message.rs"
MESSAGE_ID_BENCHMARK = (
    ROOT / "zircon_hub/tests/hub04_message_id_lookup_performance.rs"
)


class Hub04AllocationFreeMessageIdLookupPerformanceContractTests(unittest.TestCase):
    def from_str_body(self) -> str:
        source = MESSAGE_ID.read_text(encoding="utf-8")
        return source.split("pub fn from_str_id", 1)[1].split(
            "pub fn param_count", 1
        )[0]

    def test_lookup_routes_by_namespace_without_materializing_all_ids(self) -> None:
        body = self.from_str_body()
        normalized = " ".join(body.split())

        self.assertIn("let (namespace, _) = id.split_once('.')?;", normalized)
        self.assertIn("match namespace", normalized)
        self.assertNotIn("Self::all()", body)
        self.assertNotIn("Vec", body)

    def test_lookup_scans_only_the_matching_static_category(self) -> None:
        body = self.from_str_body()
        normalized = " ".join(body.split())
        categories = (
            ("shell", "ShellMessageId", "Shell"),
            ("project", "ProjectMessageId", "Project"),
            ("engine", "EngineMessageId", "Engine"),
            ("build", "BuildMessageId", "Build"),
            ("delivery", "DeliveryMessageId", "Delivery"),
            ("process", "ProcessMessageId", "Process"),
            ("settings", "SettingsMessageId", "Settings"),
            ("learn", "LearnMessageId", "Learn"),
        )

        for namespace, category, variant in categories:
            with self.subTest(namespace=namespace):
                self.assertIn(f'"{namespace}" => {category}::ALL', normalized)
                self.assertIn(f".map(Self::{variant})", normalized)
        self.assertEqual(body.count("candidate.as_str() == id"), len(categories))
        self.assertIn("_ => None", normalized)

    def test_structured_message_deserialization_keeps_the_stable_lookup(self) -> None:
        source = MESSAGE.read_text(encoding="utf-8")
        deserialize = source.split("HubMessageRepr::Structured", 1)[1]

        self.assertIn("HubMessageId::from_str_id(&id)", deserialize)
        self.assertIn("None if params.is_empty() => Self::RawText(id)", deserialize)
        self.assertIn('None => Self::RawText(format!("{id}: {}"', deserialize)

    def test_native_release_benchmark_measures_the_production_lookup(self) -> None:
        source = MESSAGE_ID_BENCHMARK.read_text(encoding="utf-8")
        normalized = " ".join(source.split())

        self.assertIn("#[global_allocator]", source)
        self.assertIn("HubMessageId::from_str_id", source)
        self.assertIn("legacy_from_str_id", source)
        self.assertIn("hub04_message_id_lookup_release_benchmark_evidence", source)
        self.assertIn("PERF_RESULT hub04_message_id_lookup", source)
        self.assertIn("lookups=8192", source)
        self.assertIn("sample_pairs=21", source)
        self.assertIn("threshold_percent=60", source)
        self.assertIn("assert_eq!(optimized_allocations, 0)", normalized)
        self.assertIn("legacy_raw_ns", source)
        self.assertIn("optimized_raw_ns", source)


if __name__ == "__main__":
    unittest.main()
