from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    ROOT
    / "zircon_runtime/src/dynamic_api/session/scene_asset_reload_diagnostics.rs"
)


class RuntimeSceneAssetReloadDiagnosticsM0PerformanceContract(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        start = cls.source.index("pub(super) fn record_scene_asset_reload_frame_report")
        end = cls.source.index("fn record_count(", start)
        cls.record_body = cls.source[start:end]

    def test_frame_report_updates_the_diagnostic_store_once(self) -> None:
        self.assertEqual(self.record_body.count("update_diagnostic_store"), 1)

    def test_frame_report_records_all_twelve_counts_and_one_boolean(self) -> None:
        self.assertEqual(self.record_body.count("record_count("), 12)
        self.assertEqual(self.record_body.count("record_bool("), 1)

    def test_helpers_use_static_metadata_without_per_series_runtime_locks(self) -> None:
        self.assertEqual(self.source.count("store.record_static("), 2)
        self.assertNotIn("runtime.record_diagnostic", self.source)


if __name__ == "__main__":
    unittest.main()
