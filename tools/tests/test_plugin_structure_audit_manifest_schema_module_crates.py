import unittest
from pathlib import Path

from tools.plugin_structure_audits.manifest_schema_module_crates import (
    collect_module_workspace_crate_violations_from_index,
)


class PluginStructureAuditManifestSchemaModuleCratesTests(unittest.TestCase):
    def test_manifest_schema_rejects_missing_module_workspace_crate(self):
        violations: list[str] = []

        collect_module_workspace_crate_violations_from_index(
            Path("zircon_plugins"),
            [
                (
                    "zircon_plugins/sound/plugin.toml",
                    plugin_manifest(
                        modules=[
                            module_row(
                                crate_name="zircon_plugin_missing_runtime",
                            )
                        ],
                    ),
                )
            ],
            {
                "zircon_plugin_sound_runtime": workspace_crate(
                    "sound/runtime",
                )
            },
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: modules[0].crate_name "
                "zircon_plugin_missing_runtime must be a zircon_plugins "
                "workspace member",
            ],
            violations,
        )

    def test_manifest_schema_rejects_module_workspace_crate_outside_feature_root(self):
        violations: list[str] = []

        collect_module_workspace_crate_violations_from_index(
            Path("zircon_plugins"),
            [
                (
                    "zircon_plugins/sound/plugin.toml",
                    plugin_manifest(
                        optional_features=[
                            optional_feature(
                                modules=[
                                    module_row(
                                        name="sound.preview.runtime",
                                        crate_name="zircon_plugin_sound_runtime",
                                    )
                                ]
                            )
                        ],
                    ),
                )
            ],
            {
                "zircon_plugin_sound_runtime": workspace_crate(
                    "sound/runtime",
                )
            },
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: "
                "optional_features[0].modules[0].crate_name "
                "zircon_plugin_sound_runtime workspace member sound/runtime "
                "must stay under sound/features/preview",
            ],
            violations,
        )


def plugin_manifest(
    *,
    modules: list[dict[str, object]] | None = None,
    optional_features: list[dict[str, object]] | None = None,
) -> dict[str, object]:
    manifest: dict[str, object] = {"id": "sound"}
    if modules is not None:
        manifest["modules"] = modules
    if optional_features is not None:
        manifest["optional_features"] = optional_features
    return manifest


def optional_feature(*, modules: list[dict[str, object]]) -> dict[str, object]:
    return {
        "id": "sound.preview",
        "modules": modules,
    }


def module_row(
    *,
    name: str = "sound.runtime",
    crate_name: str = "zircon_plugin_sound_runtime",
) -> dict[str, object]:
    return {
        "name": name,
        "crate_name": crate_name,
    }


def workspace_crate(member: str) -> dict[str, object]:
    return {
        "member": member,
        "manifest_path": Path("zircon_plugins") / member / "Cargo.toml",
    }


if __name__ == "__main__":
    unittest.main()
