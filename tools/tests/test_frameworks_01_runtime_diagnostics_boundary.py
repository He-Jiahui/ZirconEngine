import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]


class Frameworks01RuntimeDiagnosticsBoundaryTests(unittest.TestCase):
    def test_diagnostics_physical_owner_respects_foundation_order_and_is_unique(self) -> None:
        diagnostics_crate = REPO_ROOT / "zircon_runtime/crates/zr_diagnostics"
        kernel_crate = REPO_ROOT / "zircon_runtime/crates/zr_kernel"
        legacy_log_owner = REPO_ROOT / "zircon_runtime/src/diagnostic_log"
        legacy_store_owner = REPO_ROOT / "zircon_runtime/src/core/runtime/diagnostics"

        if not diagnostics_crate.exists():
            self.assertTrue(legacy_log_owner.is_dir())
            self.assertTrue(legacy_store_owner.is_dir())
            return

        self.assertTrue(
            kernel_crate.is_dir(),
            "zr_diagnostics must not precede the planned zr_kernel foundation owner",
        )
        self.assertFalse(
            legacy_log_owner.exists(),
            "zr_diagnostics and diagnostic_log must not survive as dual implementations",
        )
        self.assertFalse(
            legacy_store_owner.exists(),
            "zr_diagnostics and core/runtime/diagnostics must not survive as dual stores",
        )

    def test_products_and_internal_crates_use_the_curated_diagnostics_surface(self) -> None:
        roots = (
            REPO_ROOT / "zircon_app",
            REPO_ROOT / "zircon_editor",
            REPO_ROOT / "zircon_plugins",
            REPO_ROOT / "zircon_runtime/crates",
        )
        violations = []
        for root in roots:
            if not root.is_dir():
                continue
            for path in sorted(root.rglob("*.rs")):
                source = path.read_text(encoding="utf-8")
                if "crate::runtime_diagnostics" in source:
                    violations.append(path.relative_to(REPO_ROOT).as_posix())
        self.assertEqual([], violations)

    def test_manager_resolving_collectors_are_facade_owned(self) -> None:
        core_diagnostics = REPO_ROOT / "zircon_runtime/src/core/runtime/diagnostics"
        facade = REPO_ROOT / "zircon_runtime/src/runtime_diagnostics"

        self.assertFalse((core_diagnostics / "collect.rs").exists())
        self.assertFalse(
            (core_diagnostics / "physics_collection_enabled.rs").exists()
        )
        self.assertFalse(
            (core_diagnostics / "physics_collection_disabled.rs").exists()
        )
        self.assertTrue((facade / "mod.rs").is_file())
        self.assertTrue((facade / "collect.rs").is_file())
        self.assertTrue((facade / "physics_collection_enabled.rs").is_file())
        self.assertTrue((facade / "physics_collection_disabled.rs").is_file())

        core_sources = "\n".join(
            path.read_text(encoding="utf-8")
            for path in core_diagnostics.rglob("*.rs")
        )
        facade_sources = "\n".join(
            path.read_text(encoding="utf-8") for path in facade.rglob("*.rs")
        )
        manager_dependency = re.compile(
            r"crate::core(?:::manager|::\{[^;]*\bmanager(?:::|[,}\s]))"
            r"|crate::\{[^;]*\bcore::(?:manager|\{[^;]*\bmanager)",
            re.DOTALL,
        )
        core_alias_import = re.compile(
            r"use\s+crate::core\s+as\s+([A-Za-z_]\w*)\s*;"
        )
        grouped_core_alias_import = re.compile(
            r"use\s+crate::core::\{[^;]*\bself\s+as\s+([A-Za-z_]\w*)",
            re.DOTALL,
        )
        self.assertIsNone(manager_dependency.search(core_sources))
        core_aliases = core_alias_import.findall(core_sources)
        core_aliases.extend(grouped_core_alias_import.findall(core_sources))
        for alias in core_aliases:
            self.assertNotRegex(core_sources, rf"\b{re.escape(alias)}::manager\b")
        self.assertNotRegex(
            core_sources,
            r"crate::(?:\{[^;]*\b)?runtime_diagnostics\b",
        )
        self.assertIn("crate::core::manager", facade_sources)
        self.assertIn("project_runtime_devtools_snapshot", core_sources)

        lib_source = (REPO_ROOT / "zircon_runtime/src/lib.rs").read_text(
            encoding="utf-8"
        )
        core_mod = (core_diagnostics / "mod.rs").read_text(encoding="utf-8")
        self.assertIn("mod runtime_diagnostics;", lib_source)
        self.assertNotIn("pub mod runtime_diagnostics;", lib_source)
        self.assertNotIn("mod collect;", core_mod)
        self.assertNotIn("pub use collect::collect_runtime_diagnostics;", core_mod)
        self.assertNotIn("collect_runtime_devtools_snapshot", core_mod)

        collector_definitions = []
        collector_pattern = re.compile(
            r"^\s*(?:pub(?:\([^)]*\))?\s+)?fn "
            r"(collect_runtime_(?:diagnostics|devtools_snapshot))\b",
            re.MULTILINE,
        )
        for path in (REPO_ROOT / "zircon_runtime/src").rglob("*.rs"):
            source = path.read_text(encoding="utf-8")
            for match in collector_pattern.finditer(source):
                collector_definitions.append(
                    (path.relative_to(REPO_ROOT).as_posix(), match.group(1))
                )
        self.assertEqual(
            collector_definitions,
            [
                (
                    "zircon_runtime/src/runtime_diagnostics/collect.rs",
                    "collect_runtime_diagnostics",
                ),
                (
                    "zircon_runtime/src/runtime_diagnostics/mod.rs",
                    "collect_runtime_devtools_snapshot",
                ),
            ],
        )

    def test_consumers_do_not_use_retired_core_collector_paths(self) -> None:
        grouped_import = re.compile(
            r"(?:crate|zircon_runtime)::core::diagnostics::\{[^}]*\b"
            r"collect_runtime_(?:diagnostics|devtools_snapshot)\b",
            re.DOTALL,
        )
        direct_alias_import = re.compile(
            r"(?:crate|zircon_runtime)::core::diagnostics\s+as\s+"
            r"([A-Za-z_]\w*)"
        )
        grouped_alias_import = re.compile(
            r"(?:crate|zircon_runtime)::core::\{[^;]*\bdiagnostics\s+as\s+"
            r"([A-Za-z_]\w*)",
            re.DOTALL,
        )
        diagnostics_self_alias_import = re.compile(
            r"use\s+(?:crate|zircon_runtime)::core::diagnostics::\{[^;]*"
            r"\bself\s+as\s+([A-Za-z_]\w*)",
            re.DOTALL,
        )
        grouped_plain_diagnostics_import = re.compile(
            r"use\s+(?:crate|zircon_runtime)::core::\{[^;]*"
            r"\bdiagnostics\s*(?=[,}])",
            re.DOTALL,
        )
        for root in (
            REPO_ROOT / "zircon_runtime/src",
            REPO_ROOT / "zircon_runtime/tests",
            REPO_ROOT / "zircon_app/src",
            REPO_ROOT / "zircon_app/tests",
            REPO_ROOT / "zircon_editor/src",
            REPO_ROOT / "zircon_editor/tests",
            REPO_ROOT / "zircon_plugins",
        ):
            for path in root.rglob("*.rs"):
                source = path.read_text(encoding="utf-8")
                relative = path.relative_to(REPO_ROOT).as_posix()
                self.assertNotIn(
                    "core::diagnostics::collect_runtime_diagnostics",
                    source,
                    f"{relative} uses the retired core diagnostics collector path",
                )
                self.assertNotIn(
                    "core::diagnostics::collect_runtime_devtools_snapshot",
                    source,
                    f"{relative} uses the retired core devtools collector path",
                )
                self.assertIsNone(
                    grouped_import.search(source),
                    f"{relative} imports a collector from the retired core surface",
                )
                aliases = direct_alias_import.findall(source)
                aliases.extend(grouped_alias_import.findall(source))
                aliases.extend(diagnostics_self_alias_import.findall(source))
                if grouped_plain_diagnostics_import.search(source):
                    aliases.append("diagnostics")
                for alias in aliases:
                    self.assertNotRegex(
                        source,
                        rf"\b{re.escape(alias)}::collect_runtime_"
                        rf"(?:diagnostics|devtools_snapshot)\b",
                        f"{relative} accesses a collector through a core diagnostics alias",
                    )


if __name__ == "__main__":
    unittest.main()
