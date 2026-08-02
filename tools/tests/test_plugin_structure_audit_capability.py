import sys
import tempfile
import unittest
from pathlib import Path


TOOLS_DIR = Path(__file__).resolve().parents[1]
if str(TOOLS_DIR) not in sys.path:
    sys.path.insert(0, str(TOOLS_DIR))

from plugin_structure_audits.capability import (  # noqa: E402
    collect_native_abi_projection_violations,
    parse_capability_string_constants,
)


class PluginStructureAuditCapabilityTests(unittest.TestCase):
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
