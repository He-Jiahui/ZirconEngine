from __future__ import annotations

import time
import tracemalloc
import unittest
from pathlib import Path

from tools.zircon_build_font_sdf import _codepoints


SCRIPT = Path(__file__).resolve().parents[1] / "zircon_build_font_sdf.py"


class FontCodepointRangePerformanceContractTests(unittest.TestCase):
    def test_rejects_surrogate_intersection_before_scalar_expansion(self) -> None:
        source = SCRIPT.read_text(encoding="utf-8")
        function = source[source.index("def _codepoints(") : source.index("def _codepoint_scalar(")]

        intersection = function.index("if start <= 0xDFFF and end >= 0xD800:")
        expansion = function.index("for codepoint in range(start, end + 1)")
        self.assertLess(intersection, expansion)

    def test_merges_ranges_before_expanding_normalized_output(self) -> None:
        source = SCRIPT.read_text(encoding="utf-8")
        function = source[source.index("def _codepoints(") : source.index("def _codepoint_scalar(")]

        merge = function.index("merged_ranges = _merge_codepoint_ranges(selected_ranges)")
        expansion = function.index("for codepoint in range(start, end + 1)")
        self.assertLess(merge, expansion)
        self.assertNotIn("selected.add(codepoint)", function)

    def test_overlapping_range_pressure_is_output_bounded(self) -> None:
        ranges = ["U+0000-U+7FFF"] * 256

        tracemalloc.start()
        started = time.perf_counter()
        try:
            codepoints = _codepoints(ranges)
            elapsed_seconds = time.perf_counter() - started
            _current_bytes, peak_bytes = tracemalloc.get_traced_memory()
        finally:
            tracemalloc.stop()

        self.assertEqual(32_768, len(codepoints))
        self.assertEqual("U+0000", codepoints[0])
        self.assertEqual("U+7FFF", codepoints[-1])
        self.assertLess(elapsed_seconds, 2.0)
        self.assertLess(peak_bytes, 3 * 1024 * 1024)


if __name__ == "__main__":
    unittest.main()
