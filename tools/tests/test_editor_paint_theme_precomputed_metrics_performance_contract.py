from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
THEME = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_theme.rs"
)


class EditorPaintThemePrecomputedMetricsPerformanceContract(unittest.TestCase):
    def test_snapshot_retains_base_and_prepared_metrics(self) -> None:
        source = THEME.read_text(encoding="utf-8")
        snapshot = source.split("pub(crate) struct HostPaintThemeSnapshot", 1)[1]
        snapshot = snapshot.split("impl HostPaintThemeSnapshot", 1)[0]

        self.assertIn("base_metrics: metrics::HostControlMetrics", snapshot)
        self.assertIn("metrics: metrics::HostControlMetrics", snapshot)

    def test_hot_metric_read_does_not_rescale_the_table(self) -> None:
        source = THEME.read_text(encoding="utf-8")
        reader = source.split("fn host_metrics_for_read()", 1)[1]
        reader = reader.split("pub(crate) fn apply_host_paint_scale_factor", 1)[0]

        self.assertIn("map(|snapshot| snapshot.metrics)", reader)
        self.assertIn("host_paint_theme_authority().load().metrics", reader)
        self.assertNotIn("at_scale", reader)

    def test_metric_and_scale_publication_prepare_scaled_metrics_once(self) -> None:
        source = THEME.read_text(encoding="utf-8")
        appearance = source.split("pub(crate) fn apply_host_appearance_from_tokens", 1)[1]
        appearance = appearance.split("pub(crate) fn capture_host_paint_theme_snapshot", 1)[0]
        replace = source.split("fn replace_host_metrics", 1)[1]
        replace = replace.split("fn replace_host_palette", 1)[0]
        scale = source.split("pub(crate) fn apply_host_paint_scale_factor", 1)[1]
        scale = scale.split("fn host_palette_for_read", 1)[0]

        self.assertIn("metrics: base_metrics.at_scale(current.scale_factor)", appearance)
        self.assertIn("base_metrics", appearance)
        self.assertIn("metrics: base_metrics.at_scale(current.scale_factor)", replace)
        self.assertIn("base_metrics", replace)
        self.assertIn("metrics: current.base_metrics.at_scale(scale_factor)", scale)
        self.assertIn("base_metrics: current.base_metrics", scale)


if __name__ == "__main__":
    unittest.main()
