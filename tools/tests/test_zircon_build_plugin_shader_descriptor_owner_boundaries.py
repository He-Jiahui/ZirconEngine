import importlib.util
import sys
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory

from tools.zircon_build_plugin_shader_descriptors import (
    collect_geometry_source_descriptors,
    collect_shader_module_specs,
    collect_shader_permutation_id_specs,
)


REPO_ROOT = Path(__file__).resolve().parents[2]
ZIRCON_BUILD = REPO_ROOT / "tools/zircon_build.py"
ZIRCON_BUILD_PLUGIN_SHADER_DESCRIPTORS = (
    REPO_ROOT / "tools/zircon_build_plugin_shader_descriptors.py"
)
ZIRCON_BUILD_PLUGIN_SHADER_DESCRIPTOR_SUPPORT = (
    REPO_ROOT / "tools/zircon_build_plugin_shader_descriptor_support.py"
)


class ZirconBuildPluginShaderDescriptorOwnerBoundaryTests(unittest.TestCase):
    def test_descriptor_owner_imports_in_top_level_script_mode(self):
        spec = importlib.util.spec_from_file_location(
            "zircon_build_plugin_shader_descriptors_standalone",
            ZIRCON_BUILD_PLUGIN_SHADER_DESCRIPTORS,
        )
        self.assertIsNotNone(spec)
        self.assertIsNotNone(spec.loader)
        tools_path = str(REPO_ROOT / "tools")
        sys.path.insert(0, tools_path)
        try:
            module = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(module)
        finally:
            sys.path.remove(tools_path)
        self.assertTrue(callable(module.collect_shader_module_specs))

    def test_plugin_shader_descriptors_live_in_plugin_shader_descriptor_owner(self):
        self.assertTrue(
            ZIRCON_BUILD_PLUGIN_SHADER_DESCRIPTORS.exists(),
            "plugin shader contribution manifest parsing belongs in zircon_build_plugin_shader_descriptors.py",
        )
        build_text = ZIRCON_BUILD.read_text(encoding="utf-8")
        descriptor_text = ZIRCON_BUILD_PLUGIN_SHADER_DESCRIPTORS.read_text(
            encoding="utf-8"
        )
        support_text = ZIRCON_BUILD_PLUGIN_SHADER_DESCRIPTOR_SUPPORT.read_text(
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
        self.assertIn("zircon_build_plugin_shader_descriptor_support", descriptor_text)
        for helper_name in (
            "collect_descriptor_rows",
            "descriptor_id_specs",
            "normalize_optional_string",
            "shader_module_content_hash",
            "shader_module_source_path",
            "unique_in_order",
        ):
            self.assertNotIn(f"def {helper_name}(", descriptor_text)
            self.assertIn(f"def {helper_name}(", support_text)
        self.assertLessEqual(
            len(descriptor_text.splitlines()),
            150,
            "zircon_build_plugin_shader_descriptors.py should stay focused on plugin shader descriptors",
        )
        self.assertLessEqual(
            len(support_text.splitlines()),
            120,
            "plugin shader descriptor support should stay a focused private leaf",
        )

    def test_support_split_preserves_descriptor_and_shader_module_behavior(self):
        with TemporaryDirectory() as directory:
            root = Path(directory)
            manifest = root / "plugin.toml"
            manifest.write_text("[plugin]\nid = 'test'\n", encoding="utf-8")
            shader = root / "shaders" / "surface.wgsl"
            shader.parent.mkdir()
            shader.write_text("@compute @workgroup_size(1) fn main() {}", encoding="utf-8")
            data = {
                "shader_permutation": {
                    "dimensions": [
                        {"token": "QUALITY", "id": 7},
                        {"token": "QUALITY", "id": 7},
                    ],
                    "shader_modules": [
                        {
                            "import_path": "test.surface",
                            "source": "shaders/surface.wgsl",
                        }
                    ],
                },
                "geometry_sources": [{"token": "mesh", "id": 3}],
            }

            self.assertEqual(
                collect_shader_permutation_id_specs(manifest, data, "dimensions"),
                ("QUALITY=7",),
            )
            self.assertEqual(
                collect_geometry_source_descriptors(manifest, data),
                ({"token": "mesh", "id": 3},),
            )
            modules = collect_shader_module_specs(manifest, data)
            self.assertEqual(modules[0]["import_path"], "test.surface")
            self.assertEqual(len(modules[0]["content_hash"]), 64)


if __name__ == "__main__":
    unittest.main()
