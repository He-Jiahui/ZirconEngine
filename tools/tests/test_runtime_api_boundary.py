import sys
import unittest
from pathlib import Path
from shutil import copytree
from tempfile import TemporaryDirectory


REPO_ROOT = Path(__file__).resolve().parents[2]
AUDIT_SCRIPTS = (
    REPO_ROOT
    / ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts"
)
sys.path.insert(0, str(AUDIT_SCRIPTS))

from runtime_structure_audits.runtime_api_boundary import (  # noqa: E402
    RUNTIME_API_DOMAINS,
    RUNTIME_API_OWNER_PATHS,
    runtime_api_boundary_audit,
)


class RuntimeApiBoundaryTests(unittest.TestCase):
    def test_runtime_api_owner_inventory_matches_the_v7_domain_tree(self) -> None:
        expected_domains = (
            "abi",
            "constants",
            "frame",
            "host",
            "session",
        )
        expected_owner_paths = (
            "abi/api_shape.rs",
            "abi/api_table.rs",
            "abi/host_api_shape.rs",
            "constants.rs",
            "frame/frame_demand.rs",
            "frame/frame_shape.rs",
            "frame/highlight_set.rs",
            "frame/viewport_pick.rs",
            "host/clipboard.rs",
            "host/host_requests.rs",
            "host/ui_action.rs",
            "host/ui_host_request.rs",
            "session/camera.rs",
            "session/editor_transform.rs",
            "session/events.rs",
            "session/operation.rs",
            "session/plugin_event_mirror.rs",
            "session/requests.rs",
            "session/session.rs",
            "session/session_identity.rs",
            "session/translated_events.rs",
            "session/viewport.rs",
        )

        audit = runtime_api_boundary_audit(REPO_ROOT)

        self.assertEqual(expected_domains, RUNTIME_API_DOMAINS)
        self.assertEqual(expected_owner_paths, RUNTIME_API_OWNER_PATHS)
        self.assertEqual(audit["expected_domain_count"], len(expected_domains))
        self.assertEqual(audit["expected_module_count"], len(expected_owner_paths))
        self.assertEqual(audit["missing_domains"], [])
        self.assertEqual(audit["missing_modules"], [])
        self.assertEqual(audit["unexpected_modules"], [])
        self.assertEqual(audit["missing_mod_declarations"], [])
        self.assertEqual(audit["missing_reexports"], [])
        self.assertEqual(audit["facade_glob_reexports"], [])
        self.assertEqual(audit["facade_reexport_statements"], 6)
        self.assertEqual(audit["max_facade_reexport_statements"], 6)
        self.assertEqual(audit["domain_facade_glob_reexports"], [])
        self.assertFalse(audit["legacy_facade_exists"])
        self.assertEqual(audit["facade_forbidden_locations"], [])
        self.assertEqual(audit["oversized_modules"], [])
        self.assertEqual(audit["risks"], [])

    def test_runtime_api_audit_rejects_the_superseded_file_facade(self) -> None:
        temporary_root = REPO_ROOT / ".codex" / "test-tmp"
        temporary_root.mkdir(parents=True, exist_ok=True)

        with TemporaryDirectory(dir=temporary_root) as directory:
            fixture_root = Path(directory)
            fixture_interface_source = fixture_root / "zircon_runtime_interface" / "src"
            copytree(
                REPO_ROOT / "zircon_runtime_interface" / "src" / "runtime_api",
                fixture_interface_source / "runtime_api",
            )
            (fixture_interface_source / "runtime_api.rs").write_text(
                "// superseded file facade\n",
                encoding="utf-8",
            )

            audit = runtime_api_boundary_audit(fixture_root)

        self.assertTrue(audit["legacy_facade_exists"])
        self.assertIn(
            "zircon_runtime_interface/src/runtime_api.rs was superseded by the folder-backed facade and must stay absent.",
            audit["risks"],
        )


if __name__ == "__main__":
    unittest.main()
