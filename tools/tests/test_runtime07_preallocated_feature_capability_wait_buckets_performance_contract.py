import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
SOURCE_PATH = (
    ROOT
    / "zircon_runtime"
    / "src"
    / "plugin"
    / "runtime_plugin"
    / "runtime_plugin_catalog"
    / "feature_resolution.rs"
)


class PreallocatedFeatureCapabilityWaitBucketsPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.source = SOURCE_PATH.read_text(encoding="utf-8")

    def test_waiting_index_reserves_pending_feature_count(self):
        self.assertIn(
            "HashMap::<String, Vec<usize>>::with_capacity(pending.len())",
            self.source,
        )

    def test_waiting_index_does_not_start_with_unbounded_growth(self):
        self.assertNotIn(
            "HashMap::<String, Vec<usize>>::new()",
            self.source,
        )

    def test_pending_storage_remains_preallocated(self):
        self.assertIn(
            "Vec::<Option<(PendingFeatureSelection<'a>, FeatureStatus)>>::with_capacity(pending.len())",
            self.source,
        )


if __name__ == "__main__":
    unittest.main()
