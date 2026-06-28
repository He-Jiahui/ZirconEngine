import unittest

from tools.plugin_structure_audits.manifest_schema import (
    collect_manifest_schema_violations,
)


class PluginStructureAuditManifestSchemaTests(unittest.TestCase):
    def test_manifest_schema_rejects_unknown_supported_target(self):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/physics/plugin.toml",
            plugin_manifest(
                supported_targets=["client_runtime", "desktop"],
            ),
            violations,
        )

        self.assertEqual(
            [
                'zircon_plugins/physics/plugin.toml: supported_targets[1] "desktop" is unsupported; expected one of client_runtime, server_runtime, editor_host'
            ],
            violations,
        )

    def test_manifest_schema_rejects_unknown_module_target_mode(self):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/physics/plugin.toml",
            plugin_manifest(
                module_target_modes=["client_runtime", "desktop"],
            ),
            violations,
        )

        self.assertEqual(
            [
                'zircon_plugins/physics/plugin.toml: modules[0].target_modes[1] "desktop" is unsupported; expected one of client_runtime, server_runtime, editor_host'
            ],
            violations,
        )

    def test_manifest_schema_rejects_unknown_supported_platform(self):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/physics/plugin.toml",
            plugin_manifest(
                supported_platforms=["windows", "ios"],
            ),
            violations,
        )

        self.assertEqual(
            [
                'zircon_plugins/physics/plugin.toml: supported_platforms[1] "ios" is unsupported; expected one of windows, linux, macos'
            ],
            violations,
        )

    def test_manifest_schema_rejects_unknown_maturity(self):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/physics/plugin.toml",
            plugin_manifest(
                maturity="preview",
            ),
            violations,
        )

        self.assertEqual(
            [
                'zircon_plugins/physics/plugin.toml: maturity "preview" is unsupported; expected one of stable, beta, experimental'
            ],
            violations,
        )

    def test_manifest_schema_rejects_unknown_module_kind(self):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/physics/plugin.toml",
            plugin_manifest(
                module_kind="sidecar",
            ),
            violations,
        )

        self.assertEqual(
            [
                'zircon_plugins/physics/plugin.toml: modules[0].kind "sidecar" is unsupported; expected one of runtime, editor, native, vm'
            ],
            violations,
        )

    def test_manifest_schema_rejects_optional_feature_module_missing_field(self):
        violations: list[str] = []
        optional_module = feature_module()
        del optional_module["crate_name"]

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(
                optional_features=[
                    plugin_feature(optional_modules=[optional_module]),
                ],
            ),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: missing optional_features[0].modules[0].crate_name"
            ],
            violations,
        )

    def test_manifest_schema_rejects_unknown_optional_feature_module_kind(self):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(
                optional_features=[
                    plugin_feature(optional_modules=[feature_module(kind="sidecar")]),
                ],
            ),
            violations,
        )

        self.assertEqual(
            [
                'zircon_plugins/sound/plugin.toml: optional_features[0].modules[0].kind "sidecar" is unsupported; expected one of runtime, editor, native, vm'
            ],
            violations,
        )

    def test_manifest_schema_rejects_unknown_optional_feature_module_target_mode(self):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(
                optional_features=[
                    plugin_feature(
                        optional_modules=[
                            feature_module(target_modes=["client_runtime", "desktop"]),
                        ],
                    ),
                ],
            ),
            violations,
        )

        self.assertEqual(
            [
                'zircon_plugins/sound/plugin.toml: optional_features[0].modules[0].target_modes[1] "desktop" is unsupported; expected one of client_runtime, server_runtime, editor_host'
            ],
            violations,
        )

    def test_manifest_schema_rejects_optional_feature_missing_owner(self):
        violations: list[str] = []
        feature = plugin_feature()
        del feature["owner_plugin_id"]

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(optional_features=[feature]),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: missing optional_features[0].owner_plugin_id"
            ],
            violations,
        )

    def test_manifest_schema_rejects_optional_feature_enabled_by_default_type(self):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(
                optional_features=[
                    plugin_feature(enabled_by_default="false"),
                ],
            ),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: optional_features[0].enabled_by_default must be a bool"
            ],
            violations,
        )

    def test_manifest_schema_rejects_unknown_optional_feature_default_packaging(self):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(
                optional_features=[
                    plugin_feature(
                        default_packaging=["source_template", "sidecar"],
                    ),
                ],
            ),
            violations,
        )

        self.assertEqual(
            [
                'zircon_plugins/sound/plugin.toml: optional_features[0].default_packaging[1] "sidecar" is unsupported; expected one of source_template, library_embed, native_dynamic'
            ],
            violations,
        )

    def test_manifest_schema_rejects_optional_feature_provider_package_id_type(self):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(
                optional_features=[
                    plugin_feature(provider_package_id=42),
                ],
            ),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: optional_features[0].provider_package_id must be a non-empty trimmed string"
            ],
            violations,
        )

    def test_manifest_schema_rejects_optional_feature_provider_package_id_untrimmed(self):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(
                optional_features=[
                    plugin_feature(provider_package_id=" sound_provider "),
                ],
            ),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: optional_features[0].provider_package_id must be a non-empty trimmed string"
            ],
            violations,
        )

    def test_manifest_schema_rejects_optional_feature_dependency_non_table(self):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(
                optional_features=[
                    plugin_feature(dependencies=["runtime.plugin.sound"]),
                ],
            ),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: optional_features[0].dependencies[0] must be a table"
            ],
            violations,
        )

    def test_manifest_schema_rejects_optional_feature_dependency_missing_plugin_id(self):
        violations: list[str] = []
        dependency = feature_dependency()
        del dependency["plugin_id"]

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(
                optional_features=[
                    plugin_feature(dependencies=[dependency]),
                ],
            ),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: missing optional_features[0].dependencies[0].plugin_id"
            ],
            violations,
        )

    def test_manifest_schema_rejects_optional_feature_dependency_missing_primary(self):
        violations: list[str] = []
        dependency = feature_dependency()
        del dependency["primary"]

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(
                optional_features=[
                    plugin_feature(dependencies=[dependency]),
                ],
            ),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: missing optional_features[0].dependencies[0].primary"
            ],
            violations,
        )

    def test_manifest_schema_rejects_optional_feature_dependency_primary_type(self):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(
                optional_features=[
                    plugin_feature(dependencies=[feature_dependency(primary="true")]),
                ],
            ),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: optional_features[0].dependencies[0].primary must be a bool"
            ],
            violations,
        )

    def test_manifest_schema_rejects_optional_feature_distribution_non_table(self):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(
                optional_features=[
                    plugin_feature(distribution="dist"),
                ],
            ),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: optional_features[0].distribution must be a table"
            ],
            violations,
        )

    def test_manifest_schema_rejects_optional_feature_distribution_missing_abi(self):
        violations: list[str] = []
        distribution = feature_distribution()
        del distribution["abi_version"]

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(
                optional_features=[
                    plugin_feature(distribution=distribution),
                ],
            ),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: missing optional_features[0].distribution.abi_version"
            ],
            violations,
        )

    def test_manifest_schema_rejects_unknown_optional_feature_distribution_form(self):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(
                optional_features=[
                    plugin_feature(
                        distribution=feature_distribution(forms=["dist", "zip"]),
                    ),
                ],
            ),
            violations,
        )

        self.assertEqual(
            [
                'zircon_plugins/sound/plugin.toml: optional_features[0].distribution.forms[1] "zip" is unsupported; expected one of embed, dist'
            ],
            violations,
        )

    def test_manifest_schema_rejects_optional_feature_distribution_missing_entry(self):
        violations: list[str] = []
        distribution = feature_distribution()
        del distribution["runtime_entry"]

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            plugin_manifest(
                optional_features=[
                    plugin_feature(distribution=distribution),
                ],
            ),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: optional_features[0].distribution must declare runtime_entry or editor_entry"
            ],
            violations,
        )


def plugin_manifest(
    *,
    supported_targets: list[str] | None = None,
    supported_platforms: list[str] | None = None,
    maturity: str = "experimental",
    module_kind: str = "runtime",
    module_target_modes: list[str] | None = None,
    optional_features: list[dict[str, object]] | None = None,
) -> dict[str, object]:
    return {
        "id": "physics",
        "version": "0.1.0",
        "sdk_api_version": "0.1",
        "display_name": "Physics",
        "category": "runtime",
        "description": "Physics plugin.",
        "supported_targets": supported_targets or ["client_runtime"],
        "supported_platforms": supported_platforms or ["windows"],
        "capabilities": ["physics"],
        "maturity": maturity,
        "modules": [
            {
                "name": "physics.runtime",
                "kind": module_kind,
                "crate_name": "zircon_plugin_physics_runtime",
                "target_modes": module_target_modes or ["client_runtime"],
                "capabilities": ["physics"],
            }
        ],
        "optional_features": optional_features or [],
    }


def plugin_feature(
    *,
    optional_modules: list[dict[str, object]] | None = None,
    dependencies: list[object] | None = None,
    distribution: object | None = None,
    provider_package_id: object | None = None,
    default_packaging: list[str] | None = None,
    enabled_by_default: object = False,
) -> dict[str, object]:
    feature: dict[str, object] = {
        "id": "sound.timeline_animation_track",
        "display_name": "Sound Timeline Animation Track",
        "owner_plugin_id": "sound",
        "capabilities": ["runtime.feature.sound.timeline_animation_track"],
        "default_packaging": default_packaging
        or ["source_template", "library_embed"],
        "enabled_by_default": enabled_by_default,
        "dependencies": dependencies or [],
        "modules": optional_modules or [feature_module()],
    }
    if distribution is not None:
        feature["distribution"] = distribution
    if provider_package_id is not None:
        feature["provider_package_id"] = provider_package_id
    return feature


def feature_module(
    *,
    kind: str = "runtime",
    target_modes: list[str] | None = None,
) -> dict[str, object]:
    return {
        "name": "sound.timeline_animation_track.runtime",
        "kind": kind,
        "crate_name": "zircon_plugin_sound_timeline_animation_runtime",
        "target_modes": target_modes or ["client_runtime"],
        "capabilities": ["runtime.feature.sound.timeline_animation_track"],
    }


def feature_dependency(*, primary: object = True) -> dict[str, object]:
    return {
        "plugin_id": "sound",
        "capability": "runtime.plugin.sound",
        "primary": primary,
    }


def feature_distribution(
    *,
    forms: list[str] | None = None,
    default_packaging: list[str] | None = None,
) -> dict[str, object]:
    return {
        "forms": forms or ["dist"],
        "default_packaging": default_packaging or ["native_dynamic"],
        "abi_version": 3,
        "engine_compat": ">=0.1, <0.2",
        "dist_crate": "zircon_plugin_sound_timeline_animation_dist",
        "descriptor_symbol": "zircon_native_plugin_descriptor_v3",
        "runtime_entry": "zircon_plugin_sound_timeline_animation_runtime_entry_v3",
    }


if __name__ == "__main__":
    unittest.main()
