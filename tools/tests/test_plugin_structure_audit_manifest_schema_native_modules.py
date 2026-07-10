import unittest

from tools.plugin_structure_audits.manifest_schema_native_modules import (
    collect_native_crate_name_collisions,
)


class PluginStructureAuditNativeModuleTests(unittest.TestCase):
    def test_shared_cdylib_is_valid_when_runtime_and_editor_kinds_are_explicit(self):
        collisions = collect_native_crate_name_collisions(
            [
                (
                    "zircon_plugins/sample/plugin.toml",
                    {
                        "distribution": {"forms": ["dist"]},
                        "modules": [
                            {"kind": "runtime", "crate_name": "sample_dist"},
                            {"kind": "editor", "crate_name": "sample_dist"},
                        ],
                    },
                )
            ]
        )

        self.assertEqual([], collisions)

    def test_same_kind_reuse_is_reported_as_native_crate_name_collision(self):
        collisions = collect_native_crate_name_collisions(
            [
                (
                    "zircon_plugins/sample/plugin.toml",
                    {
                        "distribution": {"forms": ["dist"]},
                        "modules": [
                            {"kind": "runtime", "crate_name": "sample_dist"},
                            {"kind": "runtime", "crate_name": "sample_dist"},
                        ],
                    },
                )
            ]
        )

        self.assertEqual(
            [
                "zircon_plugins/sample/plugin.toml: modules[1].crate_name "
                "sample_dist duplicates runtime ownership from modules[0]"
            ],
            collisions,
        )

    def test_shared_cdylib_without_explicit_kind_is_reported(self):
        collisions = collect_native_crate_name_collisions(
            [
                (
                    "zircon_plugins/sample/plugin.toml",
                    {
                        "distribution": {"forms": ["dist"]},
                        "modules": [
                            {"kind": "runtime", "crate_name": "sample_dist"},
                            {"crate_name": "sample_dist"},
                        ],
                    },
                )
            ]
        )

        self.assertEqual(
            [
                "zircon_plugins/sample/plugin.toml: modules[1].crate_name "
                "sample_dist reuses modules[0] without an explicit distinct kind"
            ],
            collisions,
        )


if __name__ == "__main__":
    unittest.main()
