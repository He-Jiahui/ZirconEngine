import unittest

from tools.plugin_structure_audits.dependency_boundary import (
    collect_distribution_matrix_entry,
)


class PluginStructureAuditDistributionFieldsTests(unittest.TestCase):
    def test_distribution_section_rejects_unknown_root_distribution_field(self):
        distribution_violations: list[str] = []

        collect_distribution_matrix_entry(
            "zircon_plugins/sound/plugin.toml",
            "sound",
            {
                "forms": ["embed"],
                "preview_channel": "nightly",
            },
            "distribution",
            [],
            {},
            [],
            [],
            distribution_violations,
            [],
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: "
                "distribution.preview_channel is not a known distribution field"
            ],
            distribution_violations,
        )

    def test_distribution_section_rejects_unknown_feature_distribution_field(self):
        distribution_violations: list[str] = []

        collect_distribution_matrix_entry(
            "zircon_plugins/sound/plugin.toml",
            "sound.preview",
            {
                "forms": ["embed"],
                "preview_channel": "nightly",
            },
            "feature_extensions[0].distribution",
            [],
            {},
            [],
            [],
            distribution_violations,
            [],
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: "
                "feature_extensions[0].distribution.preview_channel "
                "is not a known distribution field"
            ],
            distribution_violations,
        )


if __name__ == "__main__":
    unittest.main()
