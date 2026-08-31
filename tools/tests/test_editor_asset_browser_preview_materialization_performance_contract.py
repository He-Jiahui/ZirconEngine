from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
ASSET_BROWSER = ROOT / "zircon_editor/src/ui/layouts/views/asset_browser.rs"
THUMBNAIL_NODES = ROOT / (
    "zircon_editor/src/ui/layouts/views/asset_browser/thumbnail_nodes.rs"
)
PIXEL_LOADER = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "visual_assets/loading/pixels.rs"
)
PIXEL_CACHE = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "visual_assets/loading/cache.rs"
)
SVG_TREE_CACHE = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "visual_assets/svg/cache.rs"
)
THUMBNAIL_PAINT = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "template_asset_placeholder_visuals/preview_image.rs"
)
PREVIEW_ARTIFACT = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "visual_assets/preview_artifact.rs"
)
CANDIDATE_QUERY = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "visual_assets/candidates/query.rs"
)
SHELL_PRESENTATION = ROOT / (
    "zircon_editor/src/ui/layouts/windows/workbench_host_window/shell_presentation.rs"
)
VIEWS_MOD = ROOT / "zircon_editor/src/ui/layouts/views/mod.rs"
EAGER_ASSET_PRESENTATION = ROOT / (
    "zircon_editor/src/ui/layouts/views/asset_surface_presentation.rs"
)


class EditorAssetBrowserPreviewMaterializationPerformanceContract(unittest.TestCase):
    def test_asset_projection_publishes_locators_without_loading_pixels(self) -> None:
        sources = "\n".join(
            path.read_text(encoding="utf-8")
            for path in (ASSET_BROWSER, THUMBNAIL_NODES)
        )

        self.assertNotIn("load_preview_image_for_generation", sources)
        self.assertNotIn("workspace_generation ^", sources)
        self.assertIn("preview_artifact_path", sources)
        self.assertIn("preview_image: Default::default()", sources)

    def test_shell_presentation_does_not_build_unconsumed_eager_asset_models(self) -> None:
        shell_source = SHELL_PRESENTATION.read_text(encoding="utf-8")
        views_mod_source = VIEWS_MOD.read_text(encoding="utf-8")

        self.assertNotIn("asset_surface_presentation", shell_source)
        self.assertNotIn("pub activity: AssetSurfacePresentation", shell_source)
        self.assertNotIn("pub browser: AssetSurfacePresentation", shell_source)
        self.assertNotIn("mod asset_surface_presentation;", views_mod_source)
        self.assertFalse(EAGER_ASSET_PRESENTATION.exists())

    def test_visible_paint_is_the_only_target_sized_materialization_authority(self) -> None:
        source = THUMBNAIL_PAINT.read_text(encoding="utf-8")
        paint = source.split("fn push_thumbnail_preview_image_command", 1)[1].split(
            "fn thumbnail_has_real_preview", 1
        )[0]

        self.assertLess(
            paint.index("intersect("), paint.index("preview_artifact_image_pixels(")
        )
        self.assertNotIn("template_image_pixels(", paint)
        self.assertIn("image.width", paint)
        self.assertIn("image.height", paint)
        self.assertIn("fitted_thumbnail_preview_image_rect", paint)

    def test_preview_artifacts_have_a_trusted_path_resolver_without_weakening_templates(
        self,
    ) -> None:
        artifact_source = PREVIEW_ARTIFACT.read_text(encoding="utf-8")
        candidate_source = CANDIDATE_QUERY.read_text(encoding="utf-8")

        self.assertIn("preview_artifact_candidates", artifact_source)
        self.assertIn("load_pixels_from_candidates", artifact_source)
        self.assertIn("PREVIEW_RASTER_BUCKET_EDGE", artifact_source)
        self.assertIn("quantized_up(PREVIEW_RASTER_BUCKET_EDGE)", artifact_source)
        self.assertIn("source_path.is_absolute()", candidate_source)
        self.assertIn("workspace_root().join(source_path)", candidate_source)
        self.assertIn(
            "packaged_image_candidates_remain_inside_the_selected_asset_root",
            candidate_source,
        )

    def test_raster_files_honor_the_same_target_contract_as_svg(self) -> None:
        source = PIXEL_LOADER.read_text(encoding="utf-8")

        self.assertIn(
            "} else {\n            load_image_from_path_for_target(&path, target)",
            source,
        )
        self.assertEqual(source.count("load_image_from_path_for_target(&path, target)"), 1)
        self.assertEqual(source.count("load_image_from_path(&path)"), 1)

    def test_visual_asset_lru_eviction_uses_an_ordered_index(self) -> None:
        source = PIXEL_CACHE.read_text(encoding="utf-8")

        self.assertIn("lru_order: BTreeMap<u64, Arc<str>>", source)
        self.assertIn(".first_key_value()", source)
        self.assertIn("self.lru_order.remove(&entry.last_used)", source)
        self.assertNotIn(".min_by_key(", source)

    def test_svg_tree_lru_eviction_uses_an_ordered_index(self) -> None:
        source = SVG_TREE_CACHE.read_text(encoding="utf-8")

        self.assertIn("lru_order: BTreeMap<u64, PathBuf>", source)
        self.assertIn(".first_key_value()", source)
        self.assertIn("self.lru_order.remove(&entry.last_used)", source)
        self.assertNotIn(".min_by_key(", source)


if __name__ == "__main__":
    unittest.main()
