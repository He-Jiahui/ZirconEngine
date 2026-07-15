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
    def test_render_owner_has_no_runtime_naming_migration_debt(self) -> None:
        report = runtime_naming_boundary_audit(REPO_ROOT)

        self.assertEqual(
            0,
            report["legacy"]["migration_debt_count"],
            report["legacy"]["migration_debt"],
        )
        self.assertEqual(
            0,
            report["legacy"]["unclassified_location_count"],
            report["legacy"]["unclassified_locations"],
        )

    def test_render_owner_has_no_hard_cutover_migration_debt(self) -> None:
        report = hard_cutover_migration_smells_audit(REPO_ROOT)

        self.assertEqual(
            0,
            report["hard_cutover_migration_debt_count"],
            report["hard_cutover_migration_debt"],
        )
        self.assertEqual([], report["risks"])

    def test_render_names_describe_current_owner_contracts(self) -> None:
        sources = {
            relative_path: (REPO_ROOT / relative_path).read_text(encoding="utf-8")
            for relative_path in (
                "zircon_runtime/src/core/framework/render/advanced_lighting/material_features.rs",
                "zircon_runtime/src/core/framework/render/scene_extract.rs",
                "zircon_runtime/src/graphics/scene/resources/gpu_texture/gpu_texture_resource_from_asset.rs",
                "zircon_runtime/src/graphics/scene/resources/gpu_texture/gpu_texture_resource_from_asset/tests.rs",
                "zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/build_post_process_params/baked_lighting.rs",
            )
        }

        for relative_path, source in sources.items():
            self.assertNotIn("legacy", source.casefold(), relative_path)

        self.assertIn(
            "hybrid_gi_pre_m4_settings_default_to_dynamic_custom_profile",
            sources["zircon_runtime/src/core/framework/render/scene_extract.rs"],
        )
        texture_source = sources[
            "zircon_runtime/src/graphics/scene/resources/gpu_texture/gpu_texture_resource_from_asset.rs"
        ]
        texture_tests = sources[
            "zircon_runtime/src/graphics/scene/resources/gpu_texture/gpu_texture_resource_from_asset/tests.rs"
        ]
        self.assertIn("page_zero_bind_group_view", texture_source)
        self.assertIn("lightmap_page_zero_bind_group_view_descriptor", texture_source)
        self.assertIn("lightmap_page_zero_bind_group_view_uses_first_page_as_d2", texture_tests)


if __name__ == "__main__":
    unittest.main()
