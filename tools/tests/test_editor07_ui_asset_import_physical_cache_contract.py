from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]


def source(relative: str) -> str:
    path = ROOT / relative
    return path.read_text(encoding="utf-8") if path.exists() else ""


def sources(*relative_paths: str) -> str:
    return "\n".join(source(relative) for relative in relative_paths)


class Editor07UiAssetImportPhysicalCacheContractTests(unittest.TestCase):
    def test_traversal_owns_one_generation_scoped_physical_cache(self) -> None:
        imports = sources(
            "zircon_editor/src/ui/host/asset_editor_sessions/imports/mod.rs",
            "zircon_editor/src/ui/host/asset_editor_sessions/imports/generation.rs",
            "zircon_editor/src/ui/host/asset_editor_sessions/imports/traversal.rs",
            "zircon_editor/src/ui/host/asset_editor_sessions/imports/collect.rs",
        )

        for required in [
            "UiAssetImportTraversal",
            "parsed_by_physical_path",
            "expanded_physical_paths",
            "Arc<ParsedUiAssetImportDocument>",
            "canonical_ui_asset_import_path",
            "load_physical_document",
            "materialize_reference",
        ]:
            self.assertIn(required, imports)
        self.assertNotIn("visited: &mut BTreeSet<String>", imports)
        self.assertFalse(
            (ROOT / "zircon_editor/src/ui/host/asset_editor_sessions/imports.rs").exists()
        )

    def test_hydration_and_lossy_refresh_share_a_traversal_per_generation(self) -> None:
        hydration = source(
            "zircon_editor/src/ui/host/asset_editor_sessions/hydration.rs"
        )
        refresh = sources(
            "zircon_editor/src/ui/host/asset_editor_sessions/refresh/imports.rs",
            "zircon_editor/src/ui/host/asset_editor_sessions/refresh/pipeline/job.rs",
        )

        self.assertIn("collect_ui_asset_imports_lossy", hydration)
        self.assertIn("replace_dependencies", hydration)
        self.assertIn("UiAssetImportTraversal::default()", refresh)
        self.assertIn("&mut traversal", refresh)
        self.assertNotIn("let mut visited = BTreeSet::new()", hydration)
        self.assertNotIn("let mut visited = BTreeSet::new()", refresh)
        self.assertFalse(
            (ROOT / "zircon_editor/src/ui/host/asset_editor_sessions/refresh.rs").exists()
        )

    def test_canonical_path_owns_read_target_and_parser_mode(self) -> None:
        imports = sources(
            "zircon_editor/src/ui/host/asset_editor_sessions/imports/collect.rs",
            "zircon_editor/src/ui/host/asset_editor_sessions/imports/parsed_document.rs",
        )

        self.assertIn("physical_path.to_string_lossy()", imports)
        self.assertIn("fs::read_to_string(&physical_path)", imports)
        self.assertNotIn("fs::read_to_string(&source_path)", imports)

    def test_behavior_suite_locks_cache_alias_and_cycle_boundaries(self) -> None:
        imports = source("zircon_editor/src/ui/host/asset_editor_sessions/imports/tests.rs")

        for test_name in [
            "physical_document_is_loaded_once_across_generation_traversals",
            "fragment_aliases_keep_logical_rows_and_expand_physical_source_once",
            "failed_physical_parse_is_cached_for_the_generation",
        ]:
            self.assertIn(f"fn {test_name}", imports)


if __name__ == "__main__":
    unittest.main()
