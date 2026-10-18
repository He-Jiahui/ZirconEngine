import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
CHROME_PROJECTION = (
    ROOT
    / "zircon_editor/src/ui/retained_host/app/viewport/toolbar_pointer/"
    "chrome_projection.rs"
)
STATUS_MODEL = ROOT / "zircon_editor/src/ui/workbench/model/status_bar_model.rs"
LOWER_REGRESSION = (
    ROOT / "zircon_editor/src/ui/workbench/model/tests/status_bar_viewport_chrome_text_tests.rs"
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
                return source[opening : index + 1]
    raise AssertionError(f"unterminated function body for {signature}")


class Editor862ViewportChromeDirectSettingsPerformanceContract(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.chrome_projection = CHROME_PROJECTION.read_text(encoding="utf-8")
        cls.status_model = STATUS_MODEL.read_text(encoding="utf-8")
        cls.lower_regression = (
            LOWER_REGRESSION.read_text(encoding="utf-8")
            if LOWER_REGRESSION.exists()
            else ""
        )

    def test_projection_reads_only_viewport_settings(self) -> None:
        body = function_body(
            self.chrome_projection,
            "fn sync_viewport_chrome_projection",
        )
        compact = "".join(body.split())

        self.assertIn("self.runtime.scene_viewport_settings()", compact)
        self.assertIn("StatusBarModel::viewport_chrome_text(&settings)", compact)
        self.assertIn("scene_viewport_chrome(&settings)", compact)
        self.assertNotIn("chrome_snapshot", body)
        self.assertNotIn("StatusBarModel::from_chrome", body)

    def test_projection_uses_the_shared_grid_and_snap_labels(self) -> None:
        body = function_body(
            self.chrome_projection,
            "fn sync_viewport_chrome_projection",
        )

        self.assertIn("grid_text.as_str()", body)
        self.assertIn("snap_text.as_str()", body)
        self.assertNotIn("status.grid_text", body)
        self.assertNotIn("status.snap_text", body)

    def test_status_model_keeps_one_label_implementation(self) -> None:
        helper = function_body(
            self.status_model,
            "pub(crate) fn viewport_chrome_text",
        )
        from_chrome = function_body(
            self.status_model,
            "pub fn from_chrome",
        )
        compact_helper = "".join(helper.split())
        compact_from_chrome = "".join(from_chrome.split())

        self.assertIn("status_grid_text(settings)", compact_helper)
        self.assertIn("status_snap_text(settings)", compact_helper)
        self.assertIn(
            "Self::viewport_chrome_text(&chrome.scene_viewport_settings)",
            compact_from_chrome,
        )

    def test_lower_semantics_and_release_marker_are_wired(self) -> None:
        self.assertIn(
            "#[path = \"tests/status_bar_viewport_chrome_text_tests.rs\"]",
            self.status_model,
        )
        self.assertIn("mod status_bar_viewport_chrome_text_tests;", self.status_model)
        self.assertTrue(self.lower_regression)
        self.assertIn(
            "editor862_viewport_chrome_text_preserves_grid_and_snap_semantics",
            self.lower_regression,
        )
        self.assertIn(
            "EDITOR862_VIEWPORT_CHROME_DIRECT_SETTINGS_BENCH_V1",
            self.lower_regression,
        )


if __name__ == "__main__":
    unittest.main()
