import sys
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
AUDIT_SCRIPTS = (
    REPO_ROOT
    / ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts"
)
sys.path.insert(0, str(AUDIT_SCRIPTS))

from runtime_structure_audits.non_network_server_naming import (  # noqa: E402
    non_network_server_references,
)


class NonNetworkServerNamingTests(unittest.TestCase):
    def test_runtime_profile_server_definition_is_a_server_runtime_owner(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = (
                root
                / "zircon_runtime/src/core/framework/project/runtime_profile_id.rs"
            )
            source.parent.mkdir(parents=True)
            source.write_text(
                "pub enum RuntimeProfileId {\n    Server,\n}\n",
                encoding="utf-8",
            )

            report = non_network_server_references(root, [source])

        self.assertEqual([], report["unclassified_locations"])
        self.assertEqual([], report["non_network_server_migration_debt"])
        self.assertEqual("classified-and-clear", report["m1_gate_status"])

    def test_runtime_profile_owner_does_not_hide_unrelated_server_names(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = (
                root
                / "zircon_runtime/src/core/framework/project/runtime_profile_id.rs"
            )
            source.parent.mkdir(parents=True)
            source.write_text(
                "pub fn build() {\n    let render_server = ();\n}\n",
                encoding="utf-8",
            )

            report = non_network_server_references(root, [source])

        self.assertEqual(1, report["unclassified_location_count"])
        self.assertEqual("render_server", report["unclassified_locations"][0]["tokens"][0])


if __name__ == "__main__":
    unittest.main()
