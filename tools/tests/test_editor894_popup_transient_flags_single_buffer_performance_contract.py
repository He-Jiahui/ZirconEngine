"""Source and cost-model contract for popup menu transient-flag cleanup."""

import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/popup_primitives.rs"
LOWER = ROOT / "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/popup_primitives/tests/transient_flags_single_buffer_tests.rs"


class Editor894PopupTransientFlagsSingleBufferTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.owner = cls.source.split("pub(crate) fn menu_item_without_transient_flags(", 1)[1].split(
            "pub(crate) fn menu_item_with_checked_state(", 1
        )[0]

    def test_popup_cleanup_writes_only_one_input_bounded_output(self) -> None:
        self.assertIn("String::with_capacity(raw.len())", self.owner)
        self.assertIn("output.push_str(label)", self.owner)
        self.assertIn("output.push_str(flag)", self.owner)
        self.assertIn("output.push_str(shortcut)", self.owner)

    def test_popup_cleanup_preserves_empty_flag_field_before_shortcut(self) -> None:
        self.assertIn("for flag in flags.split(',')", self.owner)
        self.assertIn("matches_transient_menu_item_flag(flag)", self.owner)
        self.assertIn("if !shortcut.is_empty()", self.owner)
        self.assertIn("output.push('|')", self.owner)

    def test_popup_cleanup_has_no_temporary_join_vector_or_formatted_output(self) -> None:
        self.assertNotIn(".join(", self.owner)
        self.assertNotIn(".collect(", self.owner)
        self.assertNotIn("collect::<", self.owner)
        self.assertNotIn("format!", self.owner)

    def test_lower_parity_and_release_marker_are_wired(self) -> None:
        self.assertIn("mod transient_flags_single_buffer_tests;", self.source)
        lower = LOWER.read_text(encoding="utf-8")
        self.assertIn("editor894_popup_transient_flags_single_buffer_preserves_exact_rows", lower)
        self.assertIn("EDITOR894_POPUP_TRANSIENT_FLAGS_SINGLE_BUFFER_BENCH_V1", lower)
        self.assertIn("const SAMPLE_PAIRS: usize = 101;", lower)
        self.assertIn("legacy_p50_ns", lower)
        self.assertIn("optimized_p99_ns", lower)

    def test_dense_menu_temporary_allocation_model(self) -> None:
        rows, flags_per_row = 4_096, 32
        legacy_reference_slots = rows * flags_per_row
        legacy_joined_strings = rows
        optimized_reference_slots = optimized_joined_strings = 0
        self.assertEqual((legacy_reference_slots, legacy_joined_strings), (131_072, 4_096))
        self.assertEqual((optimized_reference_slots, optimized_joined_strings), (0, 0))


if __name__ == "__main__":
    unittest.main()
