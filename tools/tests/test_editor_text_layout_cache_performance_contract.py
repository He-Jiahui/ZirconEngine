from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
CACHE = (
    ROOT
    / "zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/layout/cache.rs"
)
LAYOUT = ROOT / "zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/layout.rs"


class EditorTextLayoutCachePerformanceContractTests(unittest.TestCase):
    def test_stable_hit_uses_borrowed_text_and_only_miss_owns_the_key(self) -> None:
        source = CACHE.read_text(encoding="utf-8")
        lookup_start = source.index("pub(super) fn cached_paint_text_layout")
        build_start = source.index("let layout = Arc::new(build())", lookup_start)
        hit_path = source[lookup_start:build_start]
        miss_path = source[build_start:]

        self.assertIn("PaintTextLayoutCacheLookup", hit_path)
        self.assertIn(".get(&lookup)", hit_path)
        self.assertNotIn("text.to_string()", hit_path)
        self.assertIn("text.to_string()", miss_path)

    def test_capacity_pressure_evicts_incrementally_instead_of_clearing_all_layouts(self) -> None:
        source = CACHE.read_text(encoding="utf-8")

        self.assertNotIn("cache.clear()", source)
        self.assertIn("swap_remove_index(0)", source)
        self.assertIn("IndexMap<PaintTextLayoutCacheKey", source)

    def test_cache_miss_projects_one_runtime_artifact_without_host_relayout(self) -> None:
        source = LAYOUT.read_text(encoding="utf-8")
        uncached_start = source.index("fn layout_text_run_uncached(")
        next_function = source.index("\nfn display_text_from_lines(", uncached_start)
        uncached = source[uncached_start:next_function]

        self.assertIn("display_text_from_lines(&lines)", uncached)
        self.assertIn("positioned_artifact_glyphs(", uncached)
        self.assertIn("(Vec::new(), Vec::new())", uncached)
        self.assertNotIn("fontdue", source.lower())
        self.assertNotIn("shape_text_line", source)

    def test_display_text_join_has_one_preallocated_output(self) -> None:
        source = LAYOUT.read_text(encoding="utf-8")
        function_start = source.index("fn display_text_from_lines(")
        next_function = source.index("\n#[cfg(test)]", function_start)
        function = source[function_start:next_function]

        self.assertIn("String::with_capacity", function)
        self.assertIn("display_text.push_str", function)
        self.assertNotIn(".join(", function)
        self.assertNotIn("collect::<Vec", function)


if __name__ == "__main__":
    unittest.main()
