"""The real root contract must reject equal-size syntax/cfg/schema drift."""
from __future__ import annotations

import sys
import unittest
from pathlib import Path
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[2]
SCRIPTS = ROOT / ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts"
sys.path.insert(0, str(SCRIPTS))
from runtime_structure_audits import runtime_root_surface as audit


class RuntimeRootSurfaceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = (ROOT / "zircon_runtime/src/lib.rs").read_bytes().decode("utf-8").replace("\r\n", "\n")
        cls.canonical = (ROOT / audit.ROOT_CONTRACT_DOC).read_text(encoding="utf-8")
        cls.cargo = (ROOT / "zircon_runtime/Cargo.toml").read_text(encoding="utf-8")

    def validate(self, source=None, canonical=None, cargo=None):
        return audit.validate_root_contract(
            self.source if source is None else source,
            self.canonical if canonical is None else canonical,
            self.cargo if cargo is None else cargo,
        )

    def assert_rejected(self, source: str) -> None:
        self.assertLessEqual(len(source.splitlines()), 127)
        result = self.validate(source)
        self.assertTrue(result["risks"])
        self.assertNotIn("root exceeds 127 physical lines", result["risks"])
        reader = audit._read_text
        with patch.object(audit, "_read_text", side_effect=lambda path:
                          source if path == ROOT / "zircon_runtime/src/lib.rs" else reader(path)):
            report = audit.runtime_root_surface_audit(ROOT)
        self.assertTrue(report["risks"])
        self.assertNotEqual(report["m1_gate_status"], "classified-and-clear")

    def test_real_root_and_cargo_feature_closures(self) -> None:
        result = self.validate()
        self.assertEqual(result["risks"], [])
        expected = {"core-min": 4, "asset-types": 5, "graphics": 14, "target-client": 21, "target-web": 17}
        self.assertEqual({name: len(items) for name, items in result["profile_public_modules"].items()}, expected)
        self.assertNotIn("browser", result["profile_public_modules"]["target-client"])
        self.assertIn("browser", result["profile_public_modules"]["target-web"])

    def test_physical_lines_are_identical_for_lf_and_crlf(self) -> None:
        lf = self.source.replace("\r\n", "\n")
        for source in (lf, lf.replace("\n", "\r\n")):
            with self.subTest(eol="CRLF" if "\r\n" in source else "LF"):
                self.assertEqual(self.validate(source)["risks"], [])
                self.assertEqual(self.validate(source)["physical_line_count"], 127)

    def test_comment_text_cannot_supply_declarations_or_business_items(self) -> None:
        source = self.source.replace("\n\n", "\n/* fn forbidden() {} pub mod imaginary; pub use plugin::{Forbidden}; /* nested */ */\n", 1)
        self.assertEqual(self.validate(source)["risks"], [])
        reader = audit._read_text
        with patch.object(audit, "_read_text", side_effect=lambda path:
                          source if path == ROOT / "zircon_runtime/src/lib.rs" else reader(path)):
            report = audit.runtime_root_surface_audit(ROOT)
        self.assertEqual(report["public_module_count"], 24)
        self.assertEqual(report["public_use_location_count"], 2)
        self.assertEqual((report["declaration_count"], report["path_attribute_count"], report["cfg_attribute_count"]), (28, 27, 25))
        self.assertEqual(report["risks"], [])
        missing = self.source.replace("pub mod core;", "/* pub mod core; */", 1)
        self.assert_rejected(missing)

    def test_small_business_and_macro_mutations_cannot_pass_by_size(self) -> None:
        for statement in ("fn hidden_business() {}", "mod inline_owner {}", 'include!("hidden.rs");',
                          "unknown_owner!();", "macro_rules! hidden { () => {} }",
                          "const HIDDEN: usize = 0;", "impl Hidden {}", "struct Hidden;"):
            with self.subTest(statement=statement):
                self.assert_rejected(self.source.replace("\n\n", "\n" + statement + "\n", 1))

    def test_equal_count_identity_import_and_cfg_mutations(self) -> None:
        mutations = (
            ("pub mod core;", "pub mod changed;"),
            ('#[path = "asset/portable.rs"]', '#[path = "asset/changed.rs"]'),
            ("zircon_host_function, zircon_host_module, ZirconScriptType,", "zircon_host_function, zircon_host_module, Hidden,"),
            ('all(feature = "asset-types", not(feature = "native-host"))', 'all(feature = "asset-types", feature = "native-host")'),
            ('all(feature = "graphics", not(feature = "native-host"))', 'all(feature = "graphics", feature = "native-host")'),
            ('    target_arch = "wasm32",', '    target_arch = "native",'),
            ('#[cfg(feature = "animation")]', '#[unknown(feature = "animation")]'),
        )
        for old, new in mutations:
            with self.subTest(old=old):
                self.assertIn(old, self.source)
                self.assert_rejected(self.source.replace(old, new, 1))

    def test_canonical_missing_duplicate_visibility_and_cfg_are_rejected(self) -> None:
        row = "| pub | core | core/mod.rs | true |"
        for replacement in ("", row + "\n" + row, row.replace("pub", "pub(crate::hidden)", 1)):
            with self.subTest(replacement=replacement):
                self.assertTrue(self.validate(canonical=self.canonical.replace(row, replacement, 1))["risks"])
        for expression in ('unexpected(feature="animation")', 'feature="unknown-owner"'):
            canonical = self.canonical.replace('| feature="animation" |', "| " + expression + " |", 1)
            source = self.source.replace('#[cfg(feature = "animation")]', "#[cfg(" + expression + ")]", 1)
            self.assertTrue(self.validate(source, canonical)["risks"])

    def test_browser_truth_table_rejects_matching_source_and_table_without_wasm_guard(self) -> None:
        canonical = self.canonical.replace('all(feature="target-web",target_arch="wasm32",not(feature="native-host"))', 'all(feature="target-web",not(feature="native-host"))', 1)
        source = self.source.replace('    target_arch = "wasm32",\n', "", 1)
        self.assertNotEqual(canonical, self.canonical)
        self.assertTrue(any("browser predicate" in risk for risk in self.validate(source, canonical)["risks"]))

    def test_real_cargo_dependency_mutation_breaks_web_profile(self) -> None:
        changed = self.cargo.replace('target-web = ["core-min"', 'target-web = ["native-host", "core-min"', 1)
        self.assertNotEqual(changed, self.cargo)
        self.assertTrue(self.validate(cargo=changed)["risks"])

    def test_one_missing_current_mirror_cannot_be_satisfied_by_other_docs(self) -> None:
        reader = audit._read_text
        for relative in audit.ROOT_CURRENT_MIRRORS:
            with self.subTest(relative=relative):
                def read(path):
                    value = reader(path)
                    return value.replace(audit.ROOT_CURRENT_ANCHOR, "") if path == ROOT / relative else value
                with patch.object(audit, "_read_text", side_effect=read):
                    report = audit.runtime_root_surface_audit(ROOT)
                self.assertTrue(any(relative in risk for risk in report["risks"]))
                self.assertNotEqual(report["m1_gate_status"], "classified-and-clear")

    def test_missing_canonical_or_current_mirror_cannot_report_clear(self) -> None:
        reader = audit._read_text
        for relative in audit.ROOT_CURRENT_MIRRORS:
            with self.subTest(relative=relative):
                def read(path):
                    if path == ROOT / relative:
                        raise FileNotFoundError(relative)
                    return reader(path)
                with patch.object(audit, "_read_text", side_effect=read):
                    report = audit.runtime_root_surface_audit(ROOT)
                self.assertTrue(any(relative in risk for risk in report["risks"]))
                self.assertNotEqual(report["m1_gate_status"], "classified-and-clear")

    def test_correct_declarations_at_128_physical_lines_still_fail(self) -> None:
        source = self.source + "\n"
        self.assertEqual(len(source.splitlines()), 128)
        self.assertEqual(self.validate(source)["risks"], ["root exceeds 127 physical lines"])


if __name__ == "__main__":
    unittest.main()
