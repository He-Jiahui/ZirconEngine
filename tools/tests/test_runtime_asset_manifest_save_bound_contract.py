import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SAVE = ROOT / "zircon_runtime/src/asset/project/manifest/save.rs"
LOAD = ROOT / "zircon_runtime/src/asset/project/manifest/load.rs"
LIMITS = ROOT / "zircon_runtime_interface/src/project/manifest_summary/limits.rs"
MANIFEST_TESTS = ROOT / "zircon_runtime/src/asset/tests/project/manifest.rs"


class RuntimeAssetManifestSaveBoundContractTests(unittest.TestCase):
    def test_save_admits_only_documents_with_the_shared_limit_before_replace(self) -> None:
        source = SAVE.read_text(encoding="utf-8")

        self.assertIn(
            "ProjectManifestSummaryError, MAX_PROJECT_MANIFEST_BYTES,",
            source,
        )
        validation = source.index("self.validate()?;")
        serialization = source.index("serialize_current_project_manifest(self)")
        size_guard = source.index("if document.len() > MAX_PROJECT_MANIFEST_BYTES")
        typed_error = source.index("ProjectManifestSummaryError::DocumentTooLarge")
        atomic_replace = source.index("atomic_write_with_fault(")

        self.assertLess(validation, serialization)
        self.assertLess(serialization, size_guard)
        self.assertLess(size_guard, typed_error)
        self.assertLess(typed_error, atomic_replace)
        self.assertNotIn("unwrap_or(now)", source)

    def test_save_regression_covers_exact_limit_plus_one_and_destination_preservation(self) -> None:
        source = SAVE.read_text(encoding="utf-8")

        for anchor in (
            "manifest_save_accepts_exact_limit_and_rejects_limit_plus_one_before_replace",
            'manifest.asset_manifest = Some("a".repeat(MAX_PROJECT_MANIFEST_BYTES - fixed_len))',
            "assert_eq!(\n            serialize_current_project_manifest(&manifest).unwrap().len(),\n            MAX_PROJECT_MANIFEST_BYTES\n        );",
            "manifest.save(&exact_path).unwrap();",
            "let loaded = ProjectManifest::load(&exact_path).unwrap();",
            ".push('a');",
            "ProjectManifestSummaryError::DocumentTooLarge {",
            "assert_eq!(fs::read(&protected_path).unwrap(), old_bytes);",
        ):
            self.assertIn(anchor, source)

    def test_load_and_save_share_the_four_mib_admission_constant(self) -> None:
        limits = LIMITS.read_text(encoding="utf-8")
        load = LOAD.read_text(encoding="utf-8")
        save = SAVE.read_text(encoding="utf-8")

        self.assertIn("pub const MAX_PROJECT_MANIFEST_BYTES: usize = 4 * 1024 * 1024;", limits)
        self.assertGreaterEqual(load.count("MAX_PROJECT_MANIFEST_BYTES"), 2)
        self.assertIn("MAX_PROJECT_MANIFEST_BYTES", save)
        self.assertIn("document.len()", save)

    def test_public_manifest_tests_retain_atomic_fault_coverage(self) -> None:
        tests = MANIFEST_TESTS.read_text(encoding="utf-8")

        for anchor in (
            "project_manifest_atomic_save_faults_keep_the_previous_manifest_readable",
            "AtomicWriteFault::Replace",
            "AtomicWriteFault::Write",
            "assert!(matches!(error, ProjectManifestError::Write { .. }));",
        ):
            self.assertIn(anchor, tests)


if __name__ == "__main__":
    unittest.main()
