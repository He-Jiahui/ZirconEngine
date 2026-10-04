from pathlib import Path
import unittest


REPO_ROOT = Path(__file__).resolve().parents[2]
RUNTIME_ABSORPTION_ROOT = (
    REPO_ROOT / "zircon_runtime" / "src" / "tests" / "runtime_absorption"
)
FIXTURE_OWNER = RUNTIME_ABSORPTION_ROOT / "current_source_fixture.rs"
RUNTIME_15_OUTPUT_ARCHIVE = (
    "docs/plans/_archive/zircon_runtime/runtime/15/"
    "2026-07-09-code-structure-and-module-conventions-output-records.md"
)
FIXTURE_SYMBOL = "RUNTIME_ARCHITECTURE_IMPLEMENTATION_OUTPUT"
CONCAT_LITERAL_CONSUMERS = {
    "dynamic_scene/split_layout.rs",
    "job_system/split_layout.rs",
    "plugin_surface_lifecycle/split_layout.rs",
    "rayon_boundary/split_layout.rs",
    "script_absorption/split_layout.rs",
    "script_host_ledger/split_layout.rs",
    "ui_architecture/split_layout.rs",
}


class RuntimeAbsorptionCurrentSourceFixtureTests(unittest.TestCase):
    def test_runtime_absorption_uses_one_tracked_current_source_fixture_owner(self) -> None:
        owner_source = FIXTURE_OWNER.read_text(encoding="utf-8")
        self.assertIn(RUNTIME_15_OUTPUT_ARCHIVE, owner_source)
        self.assertIn(FIXTURE_SYMBOL, owner_source)

        module_source = (RUNTIME_ABSORPTION_ROOT / "mod.rs").read_text(encoding="utf-8")
        self.assertIn("mod current_source_fixture;", module_source)

        evidence_guard = (
            RUNTIME_ABSORPTION_ROOT / "structure_convention" / "evidence_ownership.rs"
        ).read_text(encoding="utf-8")
        self.assertIn(
            'runtime_src_path("tests/runtime_absorption")',
            evidence_guard,
        )
        self.assertNotIn(
            'runtime_src_path("tests/runtime_absorption/structure_convention")',
            evidence_guard,
        )

        consumers = []
        for path in RUNTIME_ABSORPTION_ROOT.rglob("*.rs"):
            source = path.read_text(encoding="utf-8")
            self.assertNotIn(".codex/sessions", source, path.as_posix())
            self.assertNotIn(
                "20260612-0847-runtime-architecture-implementation.md",
                source,
                path.as_posix(),
            )
            if path != FIXTURE_OWNER and FIXTURE_SYMBOL in source:
                consumers.append(path)

        self.assertEqual(15, len(consumers), [path.as_posix() for path in consumers])
        self.assertFalse(
            (RUNTIME_ABSORPTION_ROOT / "plan_status").exists(),
            "the retired status tree must not be restored for fixture consumers",
        )
        for relative_path in CONCAT_LITERAL_CONSUMERS:
            source = (RUNTIME_ABSORPTION_ROOT / relative_path).read_text(encoding="utf-8")
            self.assertIn(RUNTIME_15_OUTPUT_ARCHIVE, source)
            self.assertNotIn(FIXTURE_SYMBOL, source)


if __name__ == "__main__":
    unittest.main()
