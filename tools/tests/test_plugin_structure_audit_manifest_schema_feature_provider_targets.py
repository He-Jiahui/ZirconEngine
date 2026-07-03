import unittest

from tools.plugin_structure_audits.manifest_schema_feature_provider_targets import (
    collect_feature_provider_target_identity_violations,
)


class PluginStructureAuditManifestSchemaFeatureProviderTargetsTests(unittest.TestCase):
    def test_manifest_schema_rejects_duplicate_feature_provider_target_ids(self):
        violations: list[str] = []

        collect_feature_provider_target_identity_violations(
            [
                (
                    "zircon_plugins/sound/plugin.toml",
                    plugin_manifest(
                        "sound",
                        optional_features=[
                            plugin_feature(
                                "sound.preview",
                                provider_package_id="sound_preview_provider",
                            ),
                            plugin_feature(
                                "sound.reverb",
                                provider_package_id="sound_preview_provider",
                            ),
                        ],
                    ),
                ),
            ],
            violations,
        )

        self.assertEqual(
            [
                "plugin validate target sound_preview_provider is duplicated by "
                "zircon_plugins/sound/plugin.toml optional_features[0] and "
                "zircon_plugins/sound/plugin.toml optional_features[1]",
            ],
            violations,
        )

    def test_manifest_schema_rejects_feature_provider_target_colliding_with_root_id(
        self,
    ):
        violations: list[str] = []

        collect_feature_provider_target_identity_violations(
            [
                (
                    "zircon_plugins/sound_preview/plugin.toml",
                    plugin_manifest("sound_preview", distribution=root_distribution()),
                ),
                (
                    "zircon_plugins/sound/plugin.toml",
                    plugin_manifest(
                        "sound",
                        optional_features=[
                            plugin_feature("sound.preview"),
                        ],
                    ),
                ),
            ],
            violations,
        )

        self.assertEqual(
            [
                "plugin validate target sound_preview is duplicated by "
                "zircon_plugins/sound_preview/plugin.toml and "
                "zircon_plugins/sound/plugin.toml optional_features[0]",
            ],
            violations,
        )

    def test_manifest_schema_ignores_features_without_distribution_targets(self):
        violations: list[str] = []

        collect_feature_provider_target_identity_violations(
            [
                (
                    "zircon_plugins/sound/plugin.toml",
                    plugin_manifest(
                        "sound",
                        optional_features=[
                            plugin_feature(
                                "sound.preview",
                                include_distribution=False,
                                provider_package_id="sound_preview_provider",
                            ),
                            plugin_feature(
                                "sound.reverb",
                                provider_package_id="sound_preview_provider",
                            ),
                        ],
                    ),
                ),
            ],
            violations,
        )

        self.assertEqual([], violations)


def plugin_manifest(
    package_id: str,
    *,
    distribution: dict[str, object] | None = None,
    optional_features: list[dict[str, object]] | None = None,
) -> dict[str, object]:
    manifest: dict[str, object] = {"id": package_id}
    if distribution is not None:
        manifest["distribution"] = distribution
    if optional_features is not None:
        manifest["optional_features"] = optional_features
    return manifest


def plugin_feature(
    feature_id: str,
    *,
    distribution: dict[str, object] | None = None,
    include_distribution: bool = True,
    provider_package_id: str | None = None,
) -> dict[str, object]:
    feature: dict[str, object] = {"id": feature_id}
    if include_distribution:
        feature["distribution"] = distribution or feature_distribution()
    if provider_package_id is not None:
        feature["provider_package_id"] = provider_package_id
    return feature


def root_distribution() -> dict[str, object]:
    return {"forms": ["dist"]}


def feature_distribution() -> dict[str, object]:
    return {"forms": ["dist"]}


if __name__ == "__main__":
    unittest.main()
