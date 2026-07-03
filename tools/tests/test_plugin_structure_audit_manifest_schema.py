import unittest

from tools.plugin_structure_audits.manifest_schema import (
    collect_manifest_schema_violations,
)


class PluginStructureAuditManifestSchemaTests(unittest.TestCase):
    def test_manifest_schema_rejects_unknown_root_field(self):
        violations: list[str] = []
        manifest = plugin_manifest()
        manifest["legacy_sidecar"] = True

        collect_manifest_schema_violations(
            "zircon_plugins/physics/plugin.toml",
            manifest,
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/physics/plugin.toml: "
                "legacy_sidecar is not a known manifest root field"
            ],
            violations,
        )

    def test_manifest_schema_rejects_missing_root_default_packaging(self):
        violations: list[str] = []
        manifest = plugin_manifest()
        del manifest["default_packaging"]

        collect_manifest_schema_violations(
            "zircon_plugins/physics/plugin.toml",
            manifest,
            violations,
        )

        self.assertEqual(
            ["zircon_plugins/physics/plugin.toml: missing default_packaging"],
            violations,
        )

    def test_manifest_schema_rejects_malformed_root_default_packaging(self):
        violations: list[str] = []
        manifest = plugin_manifest()
        manifest["default_packaging"] = [
            "source_template",
            "nightly_dynamic",
            "native_dynamic",
            "native_dynamic",
        ]

        collect_manifest_schema_violations(
            "zircon_plugins/physics/plugin.toml",
            manifest,
            violations,
        )

        self.assertEqual(
            [
                'zircon_plugins/physics/plugin.toml: default_packaging[1] "nightly_dynamic" is unsupported; expected one of source_template, library_embed, native_dynamic',
                "zircon_plugins/physics/plugin.toml: default_packaging[3] native_dynamic duplicates default_packaging[2]",
            ],
            violations,
        )

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
                supported_platforms=["windows", "playdate"],
            ),
            violations,
        )

        self.assertEqual(
            [
                'zircon_plugins/physics/plugin.toml: supported_platforms[1] "playdate" is unsupported; expected one of windows, linux, macos, android, ios, web_gpu, wasm, headless, windows-x86_64, linux-x86_64, macos-aarch64'
            ],
            violations,
        )

    def test_manifest_schema_accepts_extended_supported_platform_values(self):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/physics/plugin.toml",
            plugin_manifest(
                supported_platforms=[
                    "android",
                    "ios",
                    "web_gpu",
                    "wasm",
                    "headless",
                    "windows-x86_64",
                    "linux-x86_64",
                    "macos-aarch64",
                ],
            ),
            violations,
        )

        self.assertEqual([], violations)

    def test_manifest_schema_rejects_supported_platform_alias_duplicates(self):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/physics/plugin.toml",
            plugin_manifest(
                supported_platforms=[
                    "windows",
                    "windows-x86_64",
                    "linux",
                    "linux-x86_64",
                    "macos",
                    "macos-aarch64",
                ],
            ),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/physics/plugin.toml: supported_platforms[1] windows-x86_64 duplicates supported_platforms[0]",
                "zircon_plugins/physics/plugin.toml: supported_platforms[3] linux-x86_64 duplicates supported_platforms[2]",
                "zircon_plugins/physics/plugin.toml: supported_platforms[5] macos-aarch64 duplicates supported_platforms[4]",
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

    def test_manifest_schema_rejects_padded_required_root_string(self):
        violations: list[str] = []
        manifest = plugin_manifest()
        manifest["display_name"] = " Physics "

        collect_manifest_schema_violations(
            "zircon_plugins/physics/plugin.toml",
            manifest,
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/physics/plugin.toml: display_name must be a non-empty trimmed string"
            ],
            violations,
        )

    def test_manifest_schema_rejects_padded_required_string_array_entry(self):
        violations: list[str] = []
        manifest = plugin_manifest()
        manifest["capabilities"] = [" physics "]

        collect_manifest_schema_violations(
            "zircon_plugins/physics/plugin.toml",
            manifest,
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/physics/plugin.toml: capabilities[0] must be a non-empty trimmed string"
            ],
            violations,
        )

    def test_manifest_schema_rejects_root_capability_semantics(self):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/physics/plugin.toml",
            plugin_manifest(
                capabilities=[
                    "physics",
                    "runtime..physics",
                    "Runtime.Physics",
                    "runtime.plugin.physics",
                    "runtime.plugin.physics",
                ],
            ),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/physics/plugin.toml: capabilities[0] physics should use at least two dot-separated namespace segments",
                "zircon_plugins/physics/plugin.toml: capabilities[1] runtime..physics should not contain empty namespace segments",
                "zircon_plugins/physics/plugin.toml: capabilities[2] Runtime.Physics should contain only lowercase ASCII letters, digits, underscores, and dots",
                "zircon_plugins/physics/plugin.toml: capabilities[4] runtime.plugin.physics duplicates capabilities capabilities[3]",
                "zircon_plugins/physics/plugin.toml: modules[0].capabilities[0] physics should start with runtime.",
                "zircon_plugins/physics/plugin.toml: modules[0].capabilities[2] Runtime.Physics should start with runtime.",
            ],
            violations,
        )

    def test_manifest_schema_reports_padded_allowed_array_entry_before_value_set(self):
        violations: list[str] = []

        collect_manifest_schema_violations(
            "zircon_plugins/physics/plugin.toml",
            plugin_manifest(supported_targets=[" client_runtime "]),
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/physics/plugin.toml: supported_targets[0] must be a non-empty trimmed string"
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

def plugin_manifest(
    *,
    package_id: str = "sound",
    capabilities: list[str] | None = None,
    supported_targets: list[str] | None = None,
    supported_platforms: list[str] | None = None,
    maturity: str = "experimental",
    module_kind: str = "runtime",
    module_target_modes: list[str] | None = None,
) -> dict[str, object]:
    manifest = {
        "id": package_id,
        "version": "0.1.0",
        "sdk_api_version": "0.1.0",
        "display_name": "Physics",
        "category": "runtime",
        "description": "Physics plugin.",
        "supported_targets": supported_targets or ["client_runtime"],
        "supported_platforms": supported_platforms or ["windows"],
        "capabilities": capabilities or ["runtime.plugin.sound"],
        "maturity": maturity,
        "default_packaging": ["source_template", "library_embed", "native_dynamic"],
        "modules": [
            {
                "name": f"{package_id}.runtime",
                "kind": module_kind,
                "crate_name": "zircon_plugin_physics_runtime",
                "target_modes": module_target_modes or ["client_runtime"],
                "capabilities": capabilities or ["runtime.plugin.sound"],
            }
        ],
    }
    return manifest


if __name__ == "__main__":
    unittest.main()
