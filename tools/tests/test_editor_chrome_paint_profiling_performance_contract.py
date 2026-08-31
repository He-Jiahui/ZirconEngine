from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
PAINT_RECORDING = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_recording/record.rs"
)
CHROME_EXTRACTION = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/"
    "chrome_command_stream/extraction/entry.rs"
)
PROFILE_MANIFEST = ROOT / "tools/profile-capture-manifest.ps1"


class EditorChromePaintProfilingPerformanceContractTests(unittest.TestCase):
    def test_command_recording_has_one_product_path_scope(self) -> None:
        source = PAINT_RECORDING.read_text(encoding="utf-8")
        body = source.split("fn record_host_frame_commands", 1)[1]

        scope = body.index(
            'profile_scope!("editor", "host_painter", "chrome_record_commands")'
        )
        paint = body.index("draw_workbench_presentation_commands")

        self.assertLess(scope, paint)

    def test_recorded_command_conversion_has_a_separate_scope(self) -> None:
        source = CHROME_EXTRACTION.read_text(encoding="utf-8")
        body = source.split("fn extract_chrome_commands", 1)[1]

        record = body.index("record_host_frame_commands")
        scope = body.index(
            'profile_scope!("editor", "host_painter", "chrome_extract_commands")'
        )
        recorded_commands = body.index("recorded_frame.commands", scope)
        convert = body.index(".into_iter()", recorded_commands)

        self.assertLess(record, scope)
        self.assertLess(scope, recorded_commands)
        self.assertLess(recorded_commands, convert)

    def test_profile_manifest_binds_both_attribution_sources(self) -> None:
        source = PROFILE_MANIFEST.read_text(encoding="utf-8")

        self.assertIn(
            '"zircon_editor/src/ui/retained_host/host_contract/paint_recording/record.rs"',
            source,
        )
        self.assertIn(
            '"zircon_editor/src/ui/retained_host/host_contract/'
            'chrome_command_stream/extraction/entry.rs"',
            source,
        )


if __name__ == "__main__":
    unittest.main()
