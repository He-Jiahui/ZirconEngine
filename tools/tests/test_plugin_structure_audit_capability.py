import sys
import tempfile
import unittest
from pathlib import Path


TOOLS_DIR = Path(__file__).resolve().parents[1]
if str(TOOLS_DIR) not in sys.path:
    sys.path.insert(0, str(TOOLS_DIR))

from plugin_structure_audits.capability import (  # noqa: E402
    FIRST_PARTY_EDITOR_RUNTIME_MIRROR_ROOTS,
    FIRST_PARTY_RUNTIME_CAPABILITY_ROOTS,
    collect_navigation_runtime_mirror_contract_violations,
    collect_native_abi_projection_violations,
    parse_capability_string_constants,
)


class PluginStructureAuditCapabilityTests(unittest.TestCase):
    def test_neural_editor_mirror_is_in_the_first_party_audit(self):
        self.assertIn("neural", FIRST_PARTY_EDITOR_RUNTIME_MIRROR_ROOTS)

    def test_neural_runtime_capability_owner_is_in_the_first_party_audit(self):
        self.assertIn("neural", FIRST_PARTY_RUNTIME_CAPABILITY_ROOTS)

    def test_navigation_overlay_frame_contract_is_the_current_mirror_audit_owner(self):
        with tempfile.TemporaryDirectory() as temporary_directory:
            repo_root = Path(temporary_directory)
            editor_plugin = repo_root / "navigation" / "editor" / "src" / "plugin.rs"
            runtime_mirror = (
                repo_root / "navigation" / "editor" / "src" / "runtime_mirror.rs"
            )
            runtime_plugin = repo_root / "navigation" / "runtime" / "src" / "plugin.rs"
            editor_plugin.parent.mkdir(parents=True)
            runtime_plugin.parent.mkdir(parents=True)
            editor_plugin.write_text(
                "navigation_runtime_event_consumers_with_mirror(pie_mirror.clone())",
                encoding="utf-8",
            )
            runtime_mirror.write_text(
                "EditorRuntimeEventConsumerState\n"
                "NAVIGATION_OVERLAY_CONSUMER_ID\n"
                "NAVIGATION_OVERLAY_FRAME_PAYLOAD_SCHEMA\n",
                encoding="utf-8",
            )
            runtime_plugin.write_text(
                "register_mirrored_event::<NavigationOverlayFrame>", encoding="utf-8"
            )

            self.assertEqual(
                [],
                collect_navigation_runtime_mirror_contract_violations(
                    repo_root, editor_plugin, runtime_mirror, runtime_plugin
                ),
            )

            runtime_plugin.write_text(
                "register_mirrored_event::<NavAgentTickReport>", encoding="utf-8"
            )
            violations = collect_navigation_runtime_mirror_contract_violations(
                repo_root, editor_plugin, runtime_mirror, runtime_plugin
            )
            self.assertEqual(1, len(violations))
            self.assertIn("NavigationOverlayFrame", violations[0])

    def test_dist_abi_projection_audit_rejects_only_hand_written_bindings(self):
        with tempfile.TemporaryDirectory() as temporary_directory:
            repo_root = Path(temporary_directory)
            legacy_dist = repo_root / "zircon_plugins" / "legacy" / "dist" / "src"
            generated_dist = (
                repo_root / "zircon_plugins" / "generated" / "dist" / "src"
            )
            legacy_native = (
                repo_root / "zircon_plugins" / "native_fixture" / "native" / "src"
            )
            legacy_dist.mkdir(parents=True)
            generated_dist.mkdir(parents=True)
            legacy_native.mkdir(parents=True)
            (legacy_dist / "lib.rs").write_text(
                r'''const PLUGIN_ID: &[u8] = b"legacy\0";
const RUNTIME_ENTRY: &[u8] = b"zircon_plugin_legacy_runtime_entry_v3\0";
const REQUESTED_CAPABILITIES: &[u8] = b"runtime.plugin.legacy\0";
const RUNTIME_REGISTRATION_MANIFEST: &[u8] = b"capabilities = []\n\0";
''',
                encoding="utf-8",
            )
            (generated_dist / "lib.rs").write_text(
                r'''// const PLUGIN_ID: &[u8] = b"comment\0";
const EXAMPLE: &str = r#"const REQUESTED_CAPABILITIES: &[u8] = b\"string\0\";"#;
zircon_plugin_sdk::native_dist_runtime_plugin_v3! {
    declaration: zircon_plugin_generated_runtime::PLUGIN_DECLARATION,
}
''',
                encoding="utf-8",
            )
            (legacy_native / "lib.rs").write_text(
                r'''const PLUGIN_ID: &[u8] = b"native_fixture\0";
const EDITOR_ENTRY: &[u8] = b"zircon_native_fixture_editor_entry_v3\0";
const REQUESTED_CAPABILITIES: &[u8] = b"editor.extension.native_fixture\0";
''',
                encoding="utf-8",
            )

            self.assertEqual(
                [
                    "zircon_plugins/legacy/dist/src/lib.rs: hand-written native ABI "
                    "projections [PLUGIN_ID, RUNTIME_ENTRY, REQUESTED_CAPABILITIES, "
                    "RUNTIME_REGISTRATION_MANIFEST] must be generated from the plugin "
                    "declaration",
                    "zircon_plugins/native_fixture/native/src/lib.rs: hand-written native ABI "
                    "projections [PLUGIN_ID, EDITOR_ENTRY, REQUESTED_CAPABILITIES] must be "
                    "generated from the plugin declaration",
                ],
                collect_native_abi_projection_violations(repo_root),
            )

    def test_macro_capabilities_ignore_non_code_and_parse_each_declaration(self):
        source = r'''// zircon_plugin_sdk::declare_plugin! { capabilities: [COMMENT_CAPABILITY = "runtime.comment" => runtime_registration], maturity: stable, }
/* outer /* zircon_plugin_sdk::declare_plugin! { capabilities: [BLOCK_CAPABILITY = "runtime.block" => runtime_registration], maturity: stable, } */ */
const EXAMPLE: &str = r#"zircon_plugin_sdk::declare_plugin! { capabilities: [STRING_CAPABILITY = \"runtime.string\" => runtime_registration], maturity: stable, }"#;

    zircon_plugin_sdk::declare_plugin! {
        pub FIRST {
            // capabilities: [COMMENT_CAPABILITY = "runtime.comment" => runtime_registration], maturity: stable,
            module_description: r#"capabilities: [STRING_CAPABILITY = "runtime.string" => runtime_registration], maturity: stable, with { braces }"#,
            capabilities: [FIRST_CAPABILITY = "runtime.first" => runtime_registration],
            maturity: stable,
        }
    }

    zircon_plugin_sdk::declare_plugin! /* separator */ {
        pub SECOND {
            capabilities: [SECOND_CAPABILITY = "runtime.second" => runtime_registration],
            maturity: beta,
        }
    }
'''
        with tempfile.TemporaryDirectory() as temporary_directory:
            capability_path = Path(temporary_directory) / "capability.rs"
            capability_path.write_text(source, encoding="utf-8")

            self.assertEqual(
                {
                    "FIRST_CAPABILITY": "runtime.first",
                    "SECOND_CAPABILITY": "runtime.second",
                },
                parse_capability_string_constants(capability_path),
            )


if __name__ == "__main__":
    unittest.main()
