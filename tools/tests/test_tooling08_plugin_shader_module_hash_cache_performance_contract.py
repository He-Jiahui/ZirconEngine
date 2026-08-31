import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools import zircon_build_plugin_shader_descriptors as descriptors


class Tooling08PluginShaderModuleHashCachePerformanceContractTests(unittest.TestCase):
    def test_shared_shader_source_is_hashed_once_per_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            root = Path(temporary_directory)
            manifest = root / "plugin.toml"
            source = root / "shaders" / "shared.wgsl"
            source.parent.mkdir()
            source.write_text("@compute @workgroup_size(1) fn main() {}", encoding="utf-8")
            data = {
                "shader_permutation": {
                    "shader_modules": [
                        {
                            "import_path": f"plugin.shared_{index}",
                            "source": "shaders/shared.wgsl",
                        }
                        for index in range(64)
                    ]
                }
            }
            calls = 0

            def hash_source(path: Path) -> str:
                nonlocal calls
                calls += 1
                self.assertEqual(source, path)
                return "a" * 64

            with mock.patch.object(
                descriptors, "_shader_module_content_hash", side_effect=hash_source
            ):
                modules = descriptors.collect_shader_module_specs(manifest, data)

            self.assertEqual(64, len(modules))
            self.assertEqual(1, calls)


if __name__ == "__main__":
    unittest.main()
