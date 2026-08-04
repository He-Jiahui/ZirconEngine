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
        self.assertNotIn("NativeHostApiV3RegistrationScope", host_api_symbols)
        self.assertIn("NativeHostApiV4RegistrationPolicy", host_api_symbols)
        self.assertIn("NativeHostApiV4RegistrationScope", host_api_symbols)
        self.assertEqual([], report["risks"])

    def test_v2_descriptor_entry_dtos_aliases_and_fixture_feature_are_hard_cut(self) -> None:
        report = plugin_surface_lifecycle_boundary_audit(REPO_ROOT)

        self.assertEqual(
            {
                "native_loader_v1_v2_files": [],
                "plugin_v1_v2_usage_files": [],
                "native_v3_alias_files": [],
                "retired_host_api_adapter_files": [],
                "v2_fixture_feature_files": [],
            },
            {
                key: report[key]
                for key in (
                    "native_loader_v1_v2_files",
                    "plugin_v1_v2_usage_files",
                    "native_v3_alias_files",
                    "retired_host_api_adapter_files",
                    "v2_fixture_feature_files",
                )
            },
        )

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
            "use self::native as legacy_native;\npub use legacy_native::NativePluginLoader;",
            (
                "use crate::plugin as plugin_alias;\n"
                "use plugin_alias::native as native_alias;\n"
                "pub use native_alias::NativePluginLoader;"
            ),
            (
                "use crate::plugin::{self as plugin_alias};\n"
                "use plugin_alias::native as native_alias;\n"
                "pub use native_alias::NativePluginLoader;"
            ),
            (
                "macro_rules! token_eater { ($($tokens:tt)*) => {}; }\n"
                "token_eater! {\n    use swallowed\n}\n"
                "pub use native::NativePluginLoader;"
            ),
            (
                "extern crate self as runtime_alias;\n"
                "pub use self::runtime_alias::plugin::native::NativePluginLoader;"
            ),
            (
                "use crate::plugin as r#type;\n"
                "pub use self::r#type::native::NativePluginLoader;"
            ),
            (
                "use crate::plugin as 插件;\n"
                "pub use 插件::native::NativePluginLoader;"
            ),
            "pub /* visibility comment */ use self::native /* route comment */ ::NativePluginLoader;",
            "pub /* visibility comment */ (crate) use self::native::NativePluginLoader;",
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

    def test_root_hard_cut_ignores_unrelated_nested_native_module(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            plugin_root = root / "zircon_runtime/src/plugin"
            plugin_root.mkdir(parents=True)
            (plugin_root / "mod.rs").write_text(
                "pub mod native;\n"
                "pub use crate::other::{native::UnrelatedThing};\n"
                "use crate::other as other_alias;\n"
                "use other_alias::native as unrelated_native;\n"
                "pub use unrelated_native::OtherUnrelatedThing;\n",
                encoding="utf-8",
            )
            (plugin_root / "native.rs").write_text(
                "pub use super::native_plugin_loader::{NativePluginLoader};\n",
                encoding="utf-8",
            )

            report = native_plugin_public_surface_audit(root)

            self.assertEqual([], report["root_reexport_symbols"])
            self.assertEqual([], report["root_public_reexport_locations"])

    def test_root_hard_cut_ignores_extern_fn_local_aliases(self) -> None:
        variants = (
            (
                "use crate::plugin as owner_alias;\n"
                'extern "C" fn callback() {\n'
                "    let _unit = ();\n"
                "    use crate::other as owner_alias;\n"
                "}\n"
                "pub use owner_alias::native::NativePluginLoader;\n",
                True,
            ),
            (
                "use crate::other as owner_alias;\n"
                'extern "C" fn callback() {\n'
                "    let _unit = ();\n"
                "    use crate::plugin as owner_alias;\n"
                "}\n"
                "pub use owner_alias::native::UnrelatedThing;\n",
                False,
            ),
        )
        for root_reexports, expected_native in variants:
            with self.subTest(root_reexports=root_reexports), tempfile.TemporaryDirectory() as temp:
                root = Path(temp)
                plugin_root = root / "zircon_runtime/src/plugin"
                plugin_root.mkdir(parents=True)
                (plugin_root / "mod.rs").write_text(
                    f"pub mod native;\n{root_reexports}",
                    encoding="utf-8",
                )
                (plugin_root / "native.rs").write_text(
                    "pub use super::native_plugin_loader::{NativePluginLoader};\n",
                    encoding="utf-8",
                )

                report = native_plugin_public_surface_audit(root)

                self.assertEqual(expected_native, report["root_reexport_count"] > 0)
                self.assertEqual(
                    expected_native,
                    report["root_public_reexport_location_count"] > 0,
                )

    def test_root_hard_cut_resolves_unicode_alias_without_native_false_positive(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            plugin_root = root / "zircon_runtime/src/plugin"
            plugin_root.mkdir(parents=True)
            (plugin_root / "mod.rs").write_text(
                "pub mod native;\n"
                "use crate::other as 非原生;\n"
                "pub use 非原生::native::UnrelatedThing;\n",
                encoding="utf-8",
            )
            (plugin_root / "native.rs").write_text(
                "pub use super::native_plugin_loader::{NativePluginLoader};\n",
                encoding="utf-8",
            )

            report = native_plugin_public_surface_audit(root)

            self.assertEqual([], report["root_reexport_symbols"])
            self.assertEqual([], report["root_public_reexport_locations"])

    def test_root_hard_cut_normalizes_unicode_aliases_like_rust(self) -> None:
        variants = (
            (
                "use crate::plugin as \u00e9;\n"
                "pub use e\u0301::native::NativePluginLoader;\n",
                True,
            ),
            (
                "use crate::other as \u00e9;\n"
                "pub use e\u0301::native::UnrelatedThing;\n",
                False,
            ),
        )
        for root_reexports, expected_native in variants:
            with self.subTest(root_reexports=root_reexports), tempfile.TemporaryDirectory() as temp:
                root = Path(temp)
                plugin_root = root / "zircon_runtime/src/plugin"
                plugin_root.mkdir(parents=True)
                (plugin_root / "mod.rs").write_text(
                    f"pub mod native;\n{root_reexports}",
                    encoding="utf-8",
                )
                (plugin_root / "native.rs").write_text(
                    "pub use super::native_plugin_loader::{NativePluginLoader};\n",
                    encoding="utf-8",
                )

                report = native_plugin_public_surface_audit(root)

                self.assertEqual(expected_native, report["root_reexport_count"] > 0)

    def test_root_hard_cut_rejects_macro_generated_root_surface(self) -> None:
        variants = (
            (
                "macro_rules! export_native {\n"
                "    () => { pub use crate::plugin::native::NativePluginLoader; };\n"
                "}\n"
                "export_native!();\n"
            ),
            (
                "macro_rules! bind_native {\n"
                "    () => { use crate::plugin::native as generated_native; };\n"
                "}\n"
                "bind_native!();\n"
                "pub use generated_native::NativePluginLoader;\n"
            ),
            (
                "macro_rules! r#macro_rules {\n"
                "    () => { pub use crate::plugin::native::NativePluginLoader; };\n"
                "}\n"
                "r#macro_rules!();\n"
            ),
        )
        for root_reexports in variants:
            with self.subTest(root_reexports=root_reexports), tempfile.TemporaryDirectory() as temp:
                root = Path(temp)
                plugin_root = root / "zircon_runtime/src/plugin"
                plugin_root.mkdir(parents=True)
                (plugin_root / "mod.rs").write_text(
                    f"pub mod native;\n{root_reexports}",
                    encoding="utf-8",
                )
                (plugin_root / "native.rs").write_text(
                    "pub use super::native_plugin_loader::{NativePluginLoader};\n",
                    encoding="utf-8",
                )

                report = native_plugin_public_surface_audit(root)

                self.assertEqual(1, report["root_macro_invocation_count"])
                self.assertTrue(report["root_macro_invocation_locations"])
                self.assertNotEqual("classified-and-clear", report["m4_gate_status"])
                self.assertTrue(report["risks"])

    def test_root_hard_cut_ignores_macro_definitions_and_nested_invocations(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            plugin_root = root / "zircon_runtime/src/plugin"
            plugin_root.mkdir(parents=True)
            (plugin_root / "mod.rs").write_text(
                "pub mod native;\n"
                "macro_rules! unused { () => { let _unit = (); }; }\n"
                "fn invoke_inside_function() { unused!(); }\n"
                "mod nested {\n"
                "    macro_rules! nested_export {\n"
                "        () => { pub use crate::plugin::native::NativePluginLoader; };\n"
                "    }\n"
                "    nested_export!();\n"
                "}\n",
                encoding="utf-8",
            )
            (plugin_root / "native.rs").write_text(
                "pub use super::native_plugin_loader::{NativePluginLoader};\n",
                encoding="utf-8",
            )

            report = native_plugin_public_surface_audit(root)

            self.assertEqual(0, report["root_macro_invocation_count"])
            self.assertEqual([], report["root_macro_invocation_locations"])
            self.assertEqual(0, report["root_reexport_count"])
            self.assertEqual("classified-and-clear", report["m4_gate_status"])
            self.assertEqual([], report["risks"])

    def test_root_hard_cut_resolves_aliases_only_in_the_root_scope(self) -> None:
        variants = (
            (
                "use crate::plugin as owner_alias;\n"
                "mod nested {\n    use crate::other as owner_alias;\n}\n"
                "pub use owner_alias::native::NativePluginLoader;\n",
                True,
            ),
            (
                "use crate::other as owner_alias;\n"
                "mod nested {\n    use crate::plugin as owner_alias;\n}\n"
                "pub use owner_alias::native::UnrelatedThing;\n",
                False,
            ),
        )
        for root_reexports, expected_native in variants:
            with self.subTest(root_reexports=root_reexports), tempfile.TemporaryDirectory() as temp:
                root = Path(temp)
                plugin_root = root / "zircon_runtime/src/plugin"
                plugin_root.mkdir(parents=True)
                (plugin_root / "mod.rs").write_text(
                    f"pub mod native;\n{root_reexports}",
                    encoding="utf-8",
                )
                (plugin_root / "native.rs").write_text(
                    "pub use super::native_plugin_loader::{NativePluginLoader};\n",
                    encoding="utf-8",
                )

                report = native_plugin_public_surface_audit(root)

                self.assertEqual(expected_native, report["root_reexport_count"] > 0)
                self.assertEqual(
                    expected_native,
                    report["root_public_reexport_location_count"] > 0,
                )

    def test_root_hard_cut_preserves_cfg_exclusive_alias_candidates(self) -> None:
        aliases = (
            (
                '#[cfg(feature = "native-owner")]\n'
                "use crate::plugin as owner_alias;\n"
                '#[cfg(not(feature = "native-owner"))]\n'
                "use crate::other as owner_alias;\n"
            ),
            (
                '#[cfg(not(feature = "native-owner"))]\n'
                "use crate::other as owner_alias;\n"
                '#[cfg(feature = "native-owner")]\n'
                "use crate::plugin as owner_alias;\n"
            ),
        )
        for alias_declarations in aliases:
            with self.subTest(alias_declarations=alias_declarations), tempfile.TemporaryDirectory() as temp:
                root = Path(temp)
                plugin_root = root / "zircon_runtime/src/plugin"
                plugin_root.mkdir(parents=True)
                (plugin_root / "mod.rs").write_text(
                    "pub mod native;\n"
                    f"{alias_declarations}"
                    "pub use owner_alias::native::NativePluginLoader;\n",
                    encoding="utf-8",
                )
                (plugin_root / "native.rs").write_text(
                    "pub use super::native_plugin_loader::{NativePluginLoader};\n",
                    encoding="utf-8",
                )

                report = native_plugin_public_surface_audit(root)

                self.assertGreater(report["root_reexport_count"], 0)
                self.assertGreater(report["root_public_reexport_location_count"], 0)

    def test_root_hard_cut_reports_the_public_use_token_line(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            plugin_root = root / "zircon_runtime/src/plugin"
            plugin_root.mkdir(parents=True)
            (plugin_root / "mod.rs").write_text(
                "pub mod native;\n\n/* masked line */\npub use native::NativePluginLoader;\n",
                encoding="utf-8",
            )
            (plugin_root / "native.rs").write_text(
                "pub use super::native_plugin_loader::{NativePluginLoader};\n",
                encoding="utf-8",
            )

            report = native_plugin_public_surface_audit(root)

            self.assertEqual(4, report["root_public_reexport_locations"][0]["line"])

    def test_root_hard_cut_ignores_literals_without_hiding_following_code(self) -> None:
        variants = (
            (
                'const NORMAL: &str = "\npub use native::NativePluginLoader;\n";\n'
                'const RAW: &str = r#"\npub use native::NativePluginLoader;\n"#;\n',
                False,
            ),
            (
                'const RAW_COMMENT: &str = r#""/*"#;\n'
                "const OPEN_BRACE: char = '{';\n"
                "pub use native::NativePluginLoader;\n",
                True,
            ),
        )
        for root_reexports, expected_native in variants:
            with self.subTest(root_reexports=root_reexports), tempfile.TemporaryDirectory() as temp:
                root = Path(temp)
                plugin_root = root / "zircon_runtime/src/plugin"
                plugin_root.mkdir(parents=True)
                (plugin_root / "mod.rs").write_text(
                    f"pub mod native;\n{root_reexports}",
                    encoding="utf-8",
                )
                (plugin_root / "native.rs").write_text(
                    "pub use super::native_plugin_loader::{NativePluginLoader};\n",
                    encoding="utf-8",
                )

                report = native_plugin_public_surface_audit(root)

                self.assertEqual(expected_native, report["root_reexport_count"] > 0)
                self.assertEqual(
                    expected_native,
                    report["root_public_reexport_location_count"] > 0,
                )

    def test_v2_hard_cut_scans_runtime_native_public_owner(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            native_owner = root / "zircon_runtime/src/plugin/native.rs"
            native_owner.parent.mkdir(parents=True)
            native_owner.write_text(
                "pub struct NativePluginAbiV2;\n"
                "pub type NativePluginByteSliceV3 = NativePluginByteSliceV2;\n",
                encoding="utf-8",
            )

            report = plugin_surface_lifecycle_boundary_audit(root)

            self.assertIn(
                "zircon_runtime/src/plugin/native.rs",
                report["native_loader_v1_v2_files"],
            )
            self.assertIn(
                "zircon_runtime/src/plugin/native.rs",
                report["native_v3_alias_files"],
            )
            self.assertTrue(report["risks"])

    def test_v2_hard_cut_scans_descriptor_and_entry_function_pointer_aliases(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            native_owner = root / "zircon_runtime/src/plugin/native_plugin_loader/abi.rs"
            native_owner.parent.mkdir(parents=True)
            native_owner.write_text(
                'type NativePluginDescriptorFnV2 = unsafe extern "C" fn();\n'
                'type NativePluginEntryFnV2 = unsafe extern "C" fn();\n',
                encoding="utf-8",
            )

            report = plugin_surface_lifecycle_boundary_audit(root)

            self.assertEqual(
                ["zircon_runtime/src/plugin/native_plugin_loader/abi.rs"],
                report["native_loader_v1_v2_files"],
            )

    def test_v2_hard_cut_inventory_branches_have_isolated_mutations(self) -> None:
        cases = (
            (
                "zircon_plugins/example/Cargo.toml",
                "[features]\nabi_v2_only = []\n",
                "plugin_v1_v2_usage_files",
            ),
            (
                "zircon_runtime/src/plugin/native.rs",
                "pub struct NativeHostApiV3RegistrationScope;\n",
                "retired_host_api_adapter_files",
            ),
            (
                "zircon_plugins/native_dynamic_fixture/Cargo.toml",
                "[features]\nabi_v2_only = []\n",
                "v2_fixture_feature_files",
            ),
        )
        for relative_path, source, report_key in cases:
            with self.subTest(report_key=report_key), tempfile.TemporaryDirectory() as temp:
                root = Path(temp)
                owner = root / relative_path
                owner.parent.mkdir(parents=True)
                owner.write_text(source, encoding="utf-8")

                report = plugin_surface_lifecycle_boundary_audit(root)

                self.assertEqual([relative_path], report[report_key])


if __name__ == "__main__":
    unittest.main()
