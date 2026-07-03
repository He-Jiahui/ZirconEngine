import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
ZIRCON_BUILD = REPO_ROOT / "tools/zircon_build.py"
ZIRCON_BUILD_PLUGIN_SHADER_DESCRIPTORS = (
    REPO_ROOT / "tools/zircon_build_plugin_shader_descriptors.py"
)


class ZirconBuildPluginShaderDescriptorOwnerBoundaryTests(unittest.TestCase):
    def test_plugin_shader_descriptors_live_in_plugin_shader_descriptor_owner(self):
        self.assertTrue(
            ZIRCON_BUILD_PLUGIN_SHADER_DESCRIPTORS.exists(),
            "plugin shader contribution manifest parsing belongs in zircon_build_plugin_shader_descriptors.py",
        )
        build_text = ZIRCON_BUILD.read_text(encoding="utf-8")
        descriptor_text = ZIRCON_BUILD_PLUGIN_SHADER_DESCRIPTORS.read_text(
            encoding="utf-8"
        )

        self.assertIn(
            "from .zircon_build_plugin_shader_descriptors import (",
            build_text,
        )
        self.assertIn(
            "from zircon_build_plugin_shader_descriptors import (",
            build_text,
        )
        for function_name in (
            "collect_shader_permutation_id_specs",
            "collect_geometry_source_descriptors",
            "collect_geometry_source_descriptor_id_specs",
            "geometry_source_descriptor_id_specs",
            "collect_shading_model_descriptors",
            "shading_model_descriptor_id_specs",
            "collect_shading_model_descriptor_id_specs",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                build_text,
                f"{function_name} belongs in zircon_build_plugin_shader_descriptors.py",
            )
            self.assertIn(f"def {function_name}(", descriptor_text)

        self.assertIn("shader_permutation", descriptor_text)
        self.assertIn("geometry_sources", descriptor_text)
        self.assertIn("shading_models", descriptor_text)
        self.assertLessEqual(
            len(descriptor_text.splitlines()),
            150,
            "zircon_build_plugin_shader_descriptors.py should stay focused on plugin shader descriptors",
        )


if __name__ == "__main__":
    unittest.main()
