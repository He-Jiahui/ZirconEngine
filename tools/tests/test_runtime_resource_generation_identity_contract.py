"""Static contract for publication-identity based asset-management caching."""

from __future__ import annotations

import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
GENERATION = ROOT / "zircon_runtime/src/asset/pipeline/manager/project_asset_manager/management_generation.rs"
MANAGER = ROOT / "zircon_runtime/src/asset/pipeline/manager/project_asset_manager/management.rs"


class RuntimeResourceGenerationIdentityContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.generation = GENERATION.read_text(encoding="utf-8")
        cls.manager = MANAGER.read_text(encoding="utf-8")

    def test_asset_projection_retains_publication_identity_not_diagnostic_counters(self) -> None:
        struct = self.generation.split("pub struct ProjectAssetManagementGeneration {", 1)[1].split(
            "}\n\nimpl ProjectAssetManagementGeneration", 1
        )[0]
        self.assertIn(
            "resource_generation: Option<ResourceManagementGenerationIdentity>",
            struct,
        )
        self.assertNotIn("resource_generation: u64", struct)
        self.assertIn("Cache validity uses the resource publication identity", self.generation)
        self.assertIn("pub fn resource_generation_identity(&self)", self.generation)

    def test_cache_match_requires_project_generation_and_same_publication_identity(self) -> None:
        body = self.generation.split("pub(crate) fn is_for_generations(", 1)[1].split(
            "    pub(crate) fn has_project_generation", 1
        )[0]
        for anchor in (
            "project_generation: u64",
            "resource_generation: &ResourceManagementGenerationIdentity",
            "self.project_generation == Some(project_generation)",
            "self.resource_generation.as_ref() == Some(resource_generation)",
        ):
            self.assertIn(anchor, body, anchor)
        self.assertNotIn("diagnostics()", body)
        self.assertNotIn("generation_count", body)

    def test_closed_projection_is_empty_while_active_empty_projection_keeps_identity(self) -> None:
        tests = self.generation.split("#[cfg(test)]\nmod tests", 1)[1]
        for name in (
            "empty_generation_has_asset_only_identity_and_indexes",
            "asset_generation_remains_renderer_detail_free",
            "empty_active_project_generation_is_distinct_from_closed_projection",
        ):
            self.assertIn(f"fn {name}(", tests, name)
        for anchor in (
            "resource_generation_identity().is_none()",
            "has_project_generation()",
            "Some(Arc::new(ResourceManagementGeneration::default()).identity())",
            "ProjectAssetManagementGeneration::empty().has_project_generation()",
        ):
            self.assertIn(anchor, tests, anchor)

    def test_refresh_captures_and_reuses_the_current_resource_publication(self) -> None:
        refresh = self.manager.split(
            "pub(crate) fn refresh_asset_management_generation", 1
        )[1].split("    pub fn asset_management_record_sets", 1)[0]
        for anchor in (
            "self.resource_manager().management_generation()",
            "let resource_identity = resource_generation.identity();",
            "is_for_generations(project_generation, &resource_identity)",
            "if self",
            "return;",
            "Some(resource_identity)",
            "ProjectAssetManagementGeneration::empty()",
        ):
            self.assertIn(anchor, refresh, anchor)
        self.assertLess(refresh.index("is_for_generations"), refresh.index("build_asset_management_record_sets"))

    def test_management_scan_uses_one_captured_publication_and_observes_new_identity(self) -> None:
        scan = self.manager.split("fn astra_m10_management_scan_uses_the_captured_publication", 1)[1]
        for anchor in (
            "let captured = manager.resource_manager().management_generation();",
            "asset_ids_for_management_record_sets(&captured)",
            "register_lazy_records",
            "let published = manager.resource_manager().management_generation();",
            "assert_ne!(captured.identity(), published.identity());",
            "asset_ids_for_management_record_sets(&published)",
        ):
            self.assertIn(anchor, scan, anchor)
        self.assertLess(scan.index("&captured"), scan.index("&published"))

    def test_regressions_cover_distinct_publications_lifetime_and_refresh_clear(self) -> None:
        for name in (
            "astra_m10_cache_matches_the_publication_and_project_identity",
            "astra_m10_management_scan_uses_the_captured_publication",
            "refresh_skips_unchanged_resource_generations",
            "refresh_clears_the_asset_projection_when_no_project_is_active",
        ):
            self.assertIn(f"fn {name}(", self.generation + self.manager, name)
        for anchor in (
            "distinct_publication",
            "assert_eq!(resources.diagnostics(), distinct_publication.diagnostics())",
            "let retained = Arc::downgrade(&resources);",
            "drop(generation);",
            "has_project_generation()",
            "ProjectAssetManagementGeneration::empty()",
        ):
            self.assertIn(anchor, self.generation + self.manager, anchor)


if __name__ == "__main__":
    unittest.main()
