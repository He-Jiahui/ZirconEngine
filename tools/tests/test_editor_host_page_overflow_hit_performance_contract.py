from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/host_page_overflow_menu.rs"
)


class EditorHostPageOverflowHitPerformanceContractTests(unittest.TestCase):
    def test_uniform_overflow_rows_probe_at_most_one_candidate(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        start = source.index("fn host_page_overflow_row_hit_in_popup_for_scroll(")
        end = source.index("fn host_page_overflow_popup_frame_contains(", start)
        body = source[start:end]

        self.assertIn("let candidate_row =", body)
        self.assertIn("visible_rows.contains(&candidate_row)", body)
        self.assertIn("host_page_overflow_row_frame_for_scroll(", body)
        self.assertNotIn("for row in", body)


if __name__ == "__main__":
    unittest.main()
