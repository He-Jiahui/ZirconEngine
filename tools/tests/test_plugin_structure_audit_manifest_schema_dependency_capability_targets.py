import unittest

from tools.plugin_structure_audits.manifest_schema_dependency_capability_targets import (
    collect_dependency_capability_target_violations,
)


class PluginStructureAuditManifestSchemaDependencyCapabilityTargetsTests(
    unittest.TestCase
):
    def test_manifest_schema_rejects_dependency_capability_not_declared_by_package(
        self,
    ):
        violations: list[str] = []

        collect_dependency_capability_target_violations(
            [
                (
                    "zircon_plugins/native_dynamic_fixture/plugin.toml",
                    plugin_manifest(
                        "native_dynamic_fixture",
                        capabilities=["runtime.plugin.native_dynamic_fixture"],
                        dependencies=[
                            dependency(
                                "animation",
                                "runtime.plugin.missing",
                            )
                        ],
                    ),
                ),
                (
                    "zircon_plugins/animation/plugin.toml",
                    plugin_manifest(
                        "animation",
                        capabilities=["runtime.plugin.animation"],
                    ),
                ),
            ],
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/native_dynamic_fixture/plugin.toml: "
                "dependencies[0].capability runtime.plugin.missing should be "
                "declared by the referenced static plugin package or one of "
                "its feature rows",
            ],
            violations,
        )

    def test_manifest_schema_rejects_external_dependency_non_host_capability(self):
        violations: list[str] = []

        collect_dependency_capability_target_violations(
            [
                (
                    "zircon_plugins/native_dynamic_fixture/plugin.toml",
                    plugin_manifest(
                        "native_dynamic_fixture",
                        capabilities=["runtime.plugin.native_dynamic_fixture"],
                        dependencies=[
                            dependency(
                                "external_editor",
                                "editor.extension.timeline_sequence_authoring",
                            ),
                            dependency(
                                "host_module",
                                "runtime.module.asset_registry",
                            ),
                        ],
                    ),
                )
            ],
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/native_dynamic_fixture/plugin.toml: "
                "dependencies[0].capability "
                "editor.extension.timeline_sequence_authoring references no "
                "static plugin package and should use a runtime.module.* or "
                "runtime.capability.* host namespace",
            ],
            violations,
        )

    def test_manifest_schema_rejects_optional_feature_dependency_capability_target(
        self,
    ):
        violations: list[str] = []

        collect_dependency_capability_target_violations(
            [
                (
                    "zircon_plugins/native_dynamic_fixture/plugin.toml",
                    plugin_manifest(
                        "native_dynamic_fixture",
                        capabilities=["runtime.plugin.native_dynamic_fixture"],
                        optional_features=[
                            {
                                "id": "native_dynamic_fixture.preview",
                                "dependencies": [
                                    feature_dependency(
                                        "native_dynamic_fixture",
                                        "runtime.plugin.native_dynamic_fixture",
                                        True,
                                    ),
                                    feature_dependency(
                                        "animation",
                                        "runtime.plugin.missing",
                                        False,
                                    ),
                                ],
                            }
                        ],
                    ),
                ),
                (
                    "zircon_plugins/animation/plugin.toml",
                    plugin_manifest(
                        "animation",
                        capabilities=["runtime.plugin.animation"],
                    ),
                ),
            ],
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/native_dynamic_fixture/plugin.toml: "
                "optional_features[0].dependencies[1].capability "
                "runtime.plugin.missing should be declared by the referenced "
                "static plugin package or one of its feature rows",
            ],
            violations,
        )

    def test_manifest_schema_rejects_feature_extension_external_dependency_capability(
        self,
    ):
        violations: list[str] = []

        collect_dependency_capability_target_violations(
            [
                (
                    "zircon_plugins/native_dynamic_fixture/plugin.toml",
                    plugin_manifest(
                        "native_dynamic_fixture",
                        capabilities=["runtime.plugin.native_dynamic_fixture"],
                        feature_extensions=[
                            {
                                "id": "native_dynamic_fixture.preview",
                                "dependencies": [
                                    feature_dependency(
                                        "external_editor",
                                        "editor.extension.timeline_sequence_authoring",
                                        False,
                                    ),
                                ],
                            }
                        ],
                    ),
                )
            ],
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/native_dynamic_fixture/plugin.toml: "
                "feature_extensions[0].dependencies[0].capability "
                "editor.extension.timeline_sequence_authoring references no "
                "static plugin package and should use a runtime.module.* or "
                "runtime.capability.* host namespace",
            ],
            violations,
        )


def plugin_manifest(
    package_id: str,
    *,
    capabilities: list[str],
    dependencies: list[dict[str, object]] | None = None,
    optional_features: list[dict[str, object]] | None = None,
    feature_extensions: list[dict[str, object]] | None = None,
) -> dict[str, object]:
    manifest: dict[str, object] = {
        "id": package_id,
        "capabilities": capabilities,
    }
    if dependencies is not None:
        manifest["dependencies"] = dependencies
    if optional_features is not None:
        manifest["optional_features"] = optional_features
    if feature_extensions is not None:
        manifest["feature_extensions"] = feature_extensions
    return manifest


def dependency(dependency_id: str, capability: str) -> dict[str, object]:
    return {
        "id": dependency_id,
        "required": True,
        "capability": capability,
    }


def feature_dependency(
    plugin_id: str,
    capability: str,
    primary: bool,
) -> dict[str, object]:
    return {
        "plugin_id": plugin_id,
        "capability": capability,
        "primary": primary,
    }


if __name__ == "__main__":
    unittest.main()
