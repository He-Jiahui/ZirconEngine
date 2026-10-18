"""Static M0.2 guards for the Runtime 04 migration resolver generation boundary."""

from __future__ import annotations

from pathlib import Path
import re
import unittest


ROOT = Path(__file__).resolve().parents[2]
MIGRATION = ROOT / "zircon_runtime/src/asset/migration"


def production(source: str) -> str:
    """Return the production portion without test-only fixtures and helpers."""

    source = re.sub(
        r'(?m)^#\[cfg\(test\)\]\n(?:#\[path = "[^"\n]+"\]\n)?mod \w+;\n',
        "",
        source,
    )
    return source.split("#[cfg(test)]", 1)[0]


class Mvp00Runtime04ResolverGenerationContractTests(unittest.TestCase):
    def test_production_ignores_test_module_declarations_before_live_code(self) -> None:
        source = (
            '#[cfg(test)]\n#[path = "sidecar/tests.rs"]\nmod tests;\n'
            'pub(super) fn preflight_sidecars() {}\n'
            '#[cfg(test)]\nmod inline_tests { fn fixture() {} }\n'
        )
        live = production(source)
        self.assertIn("pub(super) fn preflight_sidecars()", live)
        self.assertNotIn("fn fixture()", live)

    @classmethod
    def setUpClass(cls) -> None:
        cls.mod = (MIGRATION / "mod.rs").read_text(encoding="utf-8")
        cls.run_source = production(
            (MIGRATION / "run.rs").read_text(encoding="utf-8")
        )
        cls.scan = production((MIGRATION / "scan.rs").read_text(encoding="utf-8"))
        cls.sidecar = production(
            (MIGRATION / "sidecar.rs").read_text(encoding="utf-8")
        )
        cls.resolver = production(
            (MIGRATION / "resolver.rs").read_text(encoding="utf-8")
        )
        cls.index = production(
            (MIGRATION / "resolver_index.rs").read_text(encoding="utf-8")
        )

    def test_module_registers_and_reexports_the_generation_index(self) -> None:
        self.assertIn("mod resolver_index;", self.mod)
        self.assertIn("MigrationCompoundBinding", self.mod)
        self.assertIn("MigrationResolverIndex", self.mod)
        self.assertIn("MigrationSourceProjection", self.mod)

    def test_run_builds_one_index_after_inventory_and_recovery(self) -> None:
        self.assertIn("MigrationInventory::build(&roots)", self.run_source)
        self.assertIn("let inventory = if pending_recovery.is_empty()", self.run_source)
        self.assertIn("MigrationResolverIndex::build(", self.run_source)
        self.assertEqual(
            1,
            self.run_source.count("MigrationResolverIndex::build("),
            "the migration run must publish one resolver index per final generation",
        )
        self.assertIn("inventory.resolver_projections()", self.run_source)
        self.assertIn(
            "MigrationResolver::new(&sidecars.index, &resolver_index)",
            self.run_source,
        )

    def test_sidecar_preflight_consumes_the_published_inventory(self) -> None:
        self.assertIn("preflight_sidecars(&root_paths, &inventory)", self.run_source)
        self.assertIn("inventory.sidecar_candidates()", self.sidecar)
        self.assertIn("inventory.physical_path_for", self.sidecar)
        self.assertIn("MigrationCompoundBinding", self.sidecar)
        self.assertNotIn("MigrationInventory::build", self.sidecar)
        self.assertNotIn("fs::read_dir(", self.sidecar)

    def test_resolver_is_a_pure_lookup_consumer(self) -> None:
        self.assertIn("sources: &'a MigrationResolverIndex", self.resolver)
        self.assertIn("resolve_project_reference_from_lookup", self.resolver)
        self.assertIn("project_hint_for_locator", self.resolver)
        self.assertNotIn("std::fs", self.resolver)
        self.assertNotIn("std::path", self.resolver)
        self.assertNotIn("PathBuf", self.resolver)
        for forbidden in (
            "read_dir(",
            "read_to_string(",
            "read_to_end(",
            "canonicalize(",
            "symlink_metadata(",
            "persisted_source_path_for_locator",
            "FilesystemProjectSourceLookup",
        ):
            self.assertNotIn(forbidden, self.resolver)

    def test_index_has_no_filesystem_probe_or_second_root_walk(self) -> None:
        self.assertIn("pub(crate) struct MigrationResolverIndex", self.index)
        self.assertIn("pub(crate) fn build(", self.index)
        self.assertIn("impl ProjectSourceLookup for MigrationResolverIndex", self.index)
        self.assertIn("self.record_lookup();", self.index)
        self.assertEqual(2, self.index.count("self.record_lookup();"))
        for forbidden in (
            "std::fs",
            "fs::",
            "File::open(",
            "read_dir(",
            "read_to_string(",
            "symlink_metadata(",
            "canonicalize(",
            "metadata(",
            "FilesystemProjectSourceLookup",
        ):
            self.assertNotIn(forbidden, self.index)

    def test_scan_owns_link_rejection_and_publishes_projections(self) -> None:
        self.assertIn("pub(super) fn build(roots:", self.scan)
        self.assertIn("is_link_or_reparse(&metadata)", self.scan)
        self.assertIn("migration asset root", self.scan)
        self.assertIn("is not a directory", self.scan)
        self.assertIn("pub(super) fn resolver_projections", self.scan)
        self.assertIn("MigrationSourceProjection::new", self.scan)
        self.assertIn("fs::read_dir(path)", self.scan)

    def test_resolver_maps_typed_failures_without_fallback(self) -> None:
        mapping = self.resolver.split("fn resolution_failure", 1)[1]
        for kind in (
            "DanglingReference",
            "MissingPath",
            "AmbiguousPath",
            "RegistryConflict",
            "UnsupportedScheme",
            "UnsafePath",
            "PathIo",
            "InvalidDocument",
        ):
            self.assertIn(f"AssetMigrationIssueKind::{kind}", mapping)
        self.assertNotIn("unwrap_or_default", mapping)
        self.assertNotIn("unwrap_or_else(||", mapping)


if __name__ == "__main__":
    unittest.main()
