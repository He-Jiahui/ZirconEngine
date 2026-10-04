import sys
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
AUDIT_SCRIPTS = (
    REPO_ROOT
    / ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts"
)
sys.path.insert(0, str(AUDIT_SCRIPTS))

from runtime_structure_audits.hard_cutover_migration_smells import (  # noqa: E402
    hard_cutover_migration_smells_audit,
)
from runtime_structure_audits.runtime_naming_boundary import (  # noqa: E402
    runtime_naming_boundary_audit,
)


class RuntimeRenderLegacyNamingTests(unittest.TestCase):
    @staticmethod
    def _is_render_owned_path(path: str) -> bool:
        normalized = path.replace("\\", "/")
        return normalized.startswith(
            (
                "zircon_runtime/src/core/framework/render/",
                "zircon_runtime/src/graphics/",
                "zircon_runtime/src/render_graph/",
            )
        )

    def test_render_owner_has_explicit_runtime_naming_classification(self) -> None:
        report = runtime_naming_boundary_audit(REPO_ROOT)

        render_unclassified = [
            location
            for location in report["legacy"]["unclassified_locations"]
            if self._is_render_owned_path(location["path"])
        ]
        self.assertEqual([], render_unclassified)

    def test_render_owner_has_explicit_hard_cutover_classification(self) -> None:
        report = hard_cutover_migration_smells_audit(REPO_ROOT)

        self.assertEqual(0, report["compat_reference_count"])
        self.assertEqual(0, report["shim_reference_count"])
        self.assertEqual(0, report["migration_bridge_smell_count"])
        render_unclassified = [
            location
            for location in report["unclassified_locations"]
            if self._is_render_owned_path(location["path"])
        ]
        self.assertEqual([], render_unclassified)

    def test_render_names_describe_current_owner_contracts(self) -> None:
        sources = {
            relative_path: (REPO_ROOT / relative_path).read_text(encoding="utf-8")
            for relative_path in (
                "zircon_runtime/src/core/framework/render/advanced_lighting/material_features.rs",
                "zircon_runtime/src/core/framework/render/scene_extract.rs",
                "zircon_runtime/src/core/framework/render/scene_extract/hybrid_gi/tests/cases.rs",
                "zircon_runtime/src/graphics/scene/resources/gpu_texture/gpu_texture_resource_from_asset.rs",
                "zircon_runtime/src/graphics/scene/resources/gpu_texture/gpu_texture_resource_from_asset/tests/cases.rs",
                "zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/build_post_process_params/baked_lighting.rs",
            )
        }

        for relative_path, source in sources.items():
            # Naming ownership is checked for production declarations.  Test
            # fixtures may deliberately retain the word when they exercise a
            # compatibility or migration case, so do not let a cfg(test)
            # function name create a production naming violation.
            production_source = source.split("#[cfg(test)]", 1)[0]
            self.assertNotIn("legacy", production_source.casefold(), relative_path)

        self.assertIn(
            "hybrid_gi_pre_m4_settings_default_to_dynamic_custom_profile",
            sources[
                "zircon_runtime/src/core/framework/render/scene_extract/hybrid_gi/tests/cases.rs"
            ],
        )
        texture_source = sources[
            "zircon_runtime/src/graphics/scene/resources/gpu_texture/gpu_texture_resource_from_asset.rs"
        ]
        texture_tests = sources[
            "zircon_runtime/src/graphics/scene/resources/gpu_texture/gpu_texture_resource_from_asset/tests/cases.rs"
        ]
        self.assertIn("page_zero_bind_group_view", texture_source)
        self.assertIn("lightmap_page_zero_bind_group_view_descriptor", texture_source)
        self.assertIn("lightmap_page_zero_bind_group_view_uses_first_page_as_d2", texture_tests)


if __name__ == "__main__":
    unittest.main()
