import unittest

from tools.plugin_structure_audits.manifest_schema_option_capability_gates import (
    collect_option_required_capability_gate_violations,
)


class PluginStructureAuditManifestSchemaOptionCapabilityGatesTests(unittest.TestCase):
    def test_manifest_schema_rejects_undeclared_option_required_capability(self):
        violations: list[str] = []

        collect_option_required_capability_gate_violations(
            [
                (
                    "zircon_plugins/native_dynamic_fixture/plugin.toml",
                    plugin_manifest(
                        "native_dynamic_fixture",
                        capabilities=["runtime.plugin.native_dynamic_fixture"],
                        options=[
                            plugin_option(
                                "runtime.plugin.missing_feature",
                            ),
                            plugin_option(
                                "runtime.capability.asset_registry",
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
                "options[0].required_capability runtime.plugin.missing_feature "
                "should reference a declared static package/feature capability "
                "or an explicitly host-owned capability",
            ],
            violations,
        )

    def test_manifest_schema_accepts_optional_feature_option_required_capability(self):
        violations: list[str] = []

        collect_option_required_capability_gate_violations(
            [
                (
                    "zircon_plugins/native_dynamic_fixture/plugin.toml",
                    plugin_manifest(
                        "native_dynamic_fixture",
                        capabilities=["runtime.plugin.native_dynamic_fixture"],
                        optional_features=[
                            {
                                "id": "native_dynamic_fixture.preview_options",
                                "capabilities": [
                                    "runtime.feature.native_dynamic_fixture.preview_options",
                                ],
                            }
                        ],
                        options=[
                            plugin_option(
                                "runtime.feature.native_dynamic_fixture.preview_options",
                            )
                        ],
                    ),
                )
            ],
            violations,
        )

        self.assertEqual([], violations)


def plugin_manifest(
    package_id: str,
    *,
    capabilities: list[str],
    options: list[dict[str, object]],
    optional_features: list[dict[str, object]] | None = None,
) -> dict[str, object]:
    manifest: dict[str, object] = {
        "id": package_id,
        "capabilities": capabilities,
        "options": options,
    }
    if optional_features is not None:
        manifest["optional_features"] = optional_features
    return manifest


def plugin_option(required_capability: str) -> dict[str, object]:
    return {
        "key": "native_dynamic_fixture.debug",
        "required_capability": required_capability,
    }


if __name__ == "__main__":
    unittest.main()
