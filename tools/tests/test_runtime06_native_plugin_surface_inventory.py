from __future__ import annotations

import sys
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
AUDIT_SCRIPTS = (
    REPO_ROOT
    / ".codex"
    / "skills"
    / "zircon-project-skills"
    / "zr-runtime-interface-convergence"
    / "scripts"
)
sys.path.insert(0, str(AUDIT_SCRIPTS))

from runtime_structure_audits.native_plugin_public_surface import (  # noqa: E402
    native_plugin_public_surface_audit,
)
from runtime_structure_audits.plugin_surface_lifecycle_boundary import (  # noqa: E402
    plugin_surface_lifecycle_boundary_audit,
)


EXPECTED_NATIVE_SYMBOL_GROUPS = {
    "native-abi-contract-public-debt",
    "native-behavior-report-public-debt",
    "native-bridge-method-public-debt",
    "native-host-api-adapter-public-debt",
    "native-live-host-runtime-public-debt",
    "native-loader-discovery-public-debt",
}
EXPECTED_APP_NATIVE_PLUGIN_FILES = [
    "zircon_app/src/entry/entry_runner/bootstrap.rs",
    "zircon_app/src/entry/entry_runner/editor/tests/gui_startup.rs",
    "zircon_app/src/entry/entry_runner/mod.rs",
    "zircon_app/src/entry/export_bootstrap.rs",
    "zircon_app/src/entry/mod.rs",
    "zircon_app/src/entry/tests/profile_bootstrap.rs",
    "zircon_app/src/lib.rs",
    "zircon_app/src/prelude.rs",
]


class Runtime06NativePluginSurfaceInventoryTests(unittest.TestCase):
    def test_v4_host_registration_symbols_are_classified_without_widening_root(self) -> None:
        report = native_plugin_public_surface_audit(REPO_ROOT)

        self.assertEqual(0, report["root_reexport_count"])
        self.assertEqual(74, report["native_namespace_reexport_count"])
        self.assertEqual([], report["unclassified_native_namespace_symbols"])
        self.assertEqual(
            EXPECTED_NATIVE_SYMBOL_GROUPS,
            set(report["native_namespace_symbol_decision_groups"]),
        )
        host_api_symbols = report["native_namespace_symbol_decision_groups"][
            "native-host-api-adapter-public-debt"
        ]
        self.assertIn("NativeHostApiV3RegistrationScope", host_api_symbols)
        self.assertIn("NativeHostApiV4RegistrationPolicy", host_api_symbols)
        self.assertIn("NativeHostApiV4RegistrationScope", host_api_symbols)
        self.assertEqual([], report["risks"])

    def test_runtime06_lifecycle_inventory_tracks_all_current_app_call_sites(self) -> None:
        report = plugin_surface_lifecycle_boundary_audit(REPO_ROOT)

        self.assertEqual(74, report["native_namespace_reexport_count"])
        self.assertEqual(74, report["expected_native_namespace_reexport_count"])
        self.assertEqual(17, len(report["source_files"]))
        self.assertEqual(17, report["expected_source_file_count"])
        self.assertEqual(6, report["expected_native_namespace_symbol_group_count"])
        self.assertEqual(6, report["native_namespace_symbol_group_count"])
        self.assertEqual(EXPECTED_APP_NATIVE_PLUGIN_FILES, report["app_native_plugin_files"])
        self.assertEqual(8, report["app_native_plugin_file_count"])
        self.assertEqual(8, report["expected_app_native_plugin_file_count"])
        self.assertEqual([], report["risks"])

    def test_root_hard_cut_detects_alternate_native_reexport_syntax(self) -> None:
        variants = (
            "pub use native::{NativePluginLoader};",
            "pub use self::native::*;",
            "pub use crate::plugin::native::NativePluginLoader;",
            "pub use crate::{plugin::native::NativePluginLoader};",
        )
        for root_reexport in variants:
            with self.subTest(root_reexport=root_reexport), tempfile.TemporaryDirectory() as temp:
                root = Path(temp)
                plugin_root = root / "zircon_runtime/src/plugin"
                plugin_root.mkdir(parents=True)
                (plugin_root / "mod.rs").write_text(
                    f"pub mod native;\n{root_reexport}\n",
                    encoding="utf-8",
                )
                (plugin_root / "native.rs").write_text(
                    "pub use super::native_plugin_loader::{NativePluginLoader};\n",
                    encoding="utf-8",
                )

                report = native_plugin_public_surface_audit(root)

                self.assertGreater(report["root_reexport_count"], 0)
                self.assertGreater(report["root_public_reexport_location_count"], 0)
                self.assertTrue(report["risks"])


if __name__ == "__main__":
    unittest.main()
