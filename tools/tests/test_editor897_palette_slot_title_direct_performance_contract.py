"""Editor897 guards the palette slot-title hot path without invoking Cargo."""

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
OWNER = ROOT / "zircon_editor/src/ui/asset_editor/tree/palette_drop/resolution.rs"
MODULE = ROOT / "zircon_editor/src/ui/asset_editor/tree/palette_drop/resolution/title_case.rs"
TESTS = ROOT / "zircon_editor/src/ui/asset_editor/tree/palette_drop/resolution/title_case/tests/cases.rs"


class Editor897PaletteSlotTitleDirectContract(unittest.TestCase):
    def test_title_has_its_own_narrow_module(self):
        owner = OWNER.read_text(encoding="utf-8")
        self.assertIn('mod title_case;', owner)
        self.assertIn('use self::title_case::title_case_identifier;', owner)
        self.assertNotIn('fn title_case_identifier(value:', owner)

    def test_output_reuses_one_input_bounded_string(self):
        source = MODULE.read_text(encoding="utf-8")
        self.assertIn('String::with_capacity(value.len())', source)
        self.assertIn('first.to_ascii_uppercase()', source)
        self.assertIn('ch.to_ascii_lowercase()', source)
        self.assertNotIn('collect::<Vec<_>>()', source.split('#[cfg(test)]')[0])
        self.assertNotIn('words.join(', source.split('#[cfg(test)]')[0])

    def test_lower_parity_and_release_percentiles_are_wired(self):
        tests = TESTS.read_text(encoding="utf-8")
        self.assertIn('EDITOR897_PALETTE_SLOT_TITLE_DIRECT_BENCH_V1', tests)
        self.assertIn('legacy_title(', tests)
        self.assertIn('101', tests)
        self.assertIn('p99', tests)

    def test_deterministic_per_word_allocation_model(self):
        labels, words_per_label = 4_096, 16
        legacy_children = labels * words_per_label
        optimized_children = 0
        self.assertEqual((legacy_children, optimized_children), (65_536, 0))


if __name__ == "__main__":
    unittest.main()
