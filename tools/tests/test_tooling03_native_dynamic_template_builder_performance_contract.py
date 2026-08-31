from __future__ import annotations

import ast
import inspect
import unittest

from tools.zircon_export import native_dynamic_templates
from tools.zircon_export.native_dynamic_contract import (
    NATIVE_DYNAMIC_ABI_STRING_FIELDS,
)


class NativeDynamicTemplateBuilderPerformanceContractTests(unittest.TestCase):
    def test_load_manifest_builder_does_not_grow_strings_inside_package_loop(self) -> None:
        self.assert_no_loop_string_concatenation(
            native_dynamic_templates.native_plugin_load_manifest_template
        )

    def test_payload_builder_does_not_grow_strings_inside_file_loop(self) -> None:
        self.assert_no_loop_string_concatenation(
            native_dynamic_templates.native_dynamic_payload_toml
        )

    def test_joined_builders_preserve_template_output(self) -> None:
        abi = {
            "abi_version": 3,
            **{
                field: f"value_{field}"
                for field in NATIVE_DYNAMIC_ABI_STRING_FIELDS
            },
        }
        package_exports = [
            {
                "package_id": "animation",
                "path": "plugins/animation",
                "manifest": "plugins/animation/plugin.toml",
                "package_report": (
                    "plugins/animation/native_dynamic_package.toml"
                ),
                "abi": abi,
            },
            {
                "package_id": "audio",
                "path": "plugins/audio",
                "manifest": "plugins/audio/plugin.toml",
                "package_report": "plugins/audio/native_dynamic_package.toml",
                "abi": abi,
            },
        ]
        file_manifest = [
            {"path": "native/a.dll", "bytes": 7, "sha256": "a" * 64},
            {"path": "native/b.pdb", "bytes": 11, "sha256": "b" * 64},
        ]

        load_manifest = (
            native_dynamic_templates.native_plugin_load_manifest_template(
                package_exports
            )
        )
        payload = native_dynamic_templates.native_dynamic_payload_toml(
            "payload",
            file_manifest,
        )

        self.assertEqual(load_manifest.count("[[plugins]]"), 2)
        self.assertLess(
            load_manifest.index('id = "animation"'),
            load_manifest.index('id = "audio"'),
        )
        self.assertEqual(payload.count("[[payload.files]]"), 2)
        self.assertLess(
            payload.index('path = "native/a.dll"'),
            payload.index('path = "native/b.pdb"'),
        )
        self.assertIn("file_count = 2\n", payload)

    def assert_no_loop_string_concatenation(self, function: object) -> None:
        function_source = inspect.getsource(function)
        function_tree = ast.parse(function_source)
        loop_augmented_adds = [
            node
            for loop in ast.walk(function_tree)
            if isinstance(loop, (ast.For, ast.While))
            for node in ast.walk(loop)
            if isinstance(node, ast.AugAssign) and isinstance(node.op, ast.Add)
        ]

        self.assertEqual(loop_augmented_adds, [])


if __name__ == "__main__":
    unittest.main()
