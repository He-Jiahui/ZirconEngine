import sys
import tempfile
import unittest
from pathlib import Path


TOOLS_DIR = Path(__file__).resolve().parents[1]
if str(TOOLS_DIR) not in sys.path:
    sys.path.insert(0, str(TOOLS_DIR))

from plugin_structure_audits.capability import (  # noqa: E402
    parse_capability_string_constants,
)


class PluginStructureAuditCapabilityTests(unittest.TestCase):
    def test_macro_capabilities_ignore_non_code_and_parse_each_declaration(self):
        source = r'''// zircon_plugin_sdk::declare_plugin! { capabilities: [COMMENT_CAPABILITY = "runtime.comment"], maturity: stable, }
/* outer /* zircon_plugin_sdk::declare_plugin! { capabilities: [BLOCK_CAPABILITY = "runtime.block"], maturity: stable, } */ */
const EXAMPLE: &str = r#"zircon_plugin_sdk::declare_plugin! { capabilities: [STRING_CAPABILITY = \"runtime.string\"], maturity: stable, }"#;

    zircon_plugin_sdk::declare_plugin! {
        pub FIRST {
            // capabilities: [COMMENT_CAPABILITY = "runtime.comment"], maturity: stable,
            module_description: r#"capabilities: [STRING_CAPABILITY = "runtime.string"], maturity: stable, with { braces }"#,
            capabilities: [FIRST_CAPABILITY = "runtime.first"],
            maturity: stable,
        }
    }

    zircon_plugin_sdk::declare_plugin! /* separator */ {
        pub SECOND {
            capabilities: [SECOND_CAPABILITY = "runtime.second"],
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
