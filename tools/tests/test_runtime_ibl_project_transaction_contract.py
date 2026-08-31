"""Static ownership contracts for project-owned environment IBL publication."""

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SCAN = ROOT / "zircon_runtime/src/asset/project/manager/scan_and_import.rs"
FULL = ROOT / "zircon_runtime/src/asset/project/manager/scan_and_import/full_generation.rs"
TARGETED = ROOT / "zircon_runtime/src/asset/project/manager/scan_and_import/targeted.rs"
IBL = ROOT / "zircon_runtime/src/asset/importer/environment_ibl.rs"
DURABLE = ROOT / "zircon_runtime/src/asset/project/manager/durable_transaction.rs"


class RuntimeIblProjectTransactionContractTests(unittest.TestCase):
    def test_project_helper_prepares_and_collects_without_stage_side_effects(self) -> None:
        source = SCAN.read_text(encoding="utf-8")
        helper = source.split("fn prepare_environment_ibl_import(", 1)[1].split(
            "fn append_shader_import_path_conflict_diagnostics(", 1
        )[0]

        self.assertIn("prepare_environment_ibl_source", helper)
        self.assertIn("prepare_source_cubemap_texture", helper)
        self.assertIn("into_file_writes", helper)
        self.assertNotIn("stage_environment_ibl_source(", helper)
        self.assertNotIn("stage_source_cubemap_texture(", helper)

    def test_full_and_targeted_generation_append_prepared_ibl_files(self) -> None:
        full = FULL.read_text(encoding="utf-8")
        targeted = TARGETED.read_text(encoding="utf-8")

        self.assertIn("append_prepared_file_writes", full)
        self.assertIn("prepare_environment_ibl_import", full)
        self.assertIn("prepared_ibl_writes", targeted)
        self.assertIn("writes.extend(prepared_ibl_writes)", targeted)

    def test_direct_ibl_entry_points_retain_a_separate_commit_boundary(self) -> None:
        source = IBL.read_text(encoding="utf-8")

        self.assertIn("struct PreparedEnvironmentIblSourceStaging", source)
        self.assertIn("fn commit(self)", source)
        self.assertIn("commit_prepared_bundle_writes(writes)", source)
        self.assertIn("pub(crate) fn into_file_writes", source)
        self.assertIn("prepare_environment_ibl_staged_outputs", source)
        self.assertNotIn("fn write_environment_ibl_staged_outputs", source)

    def test_project_recovery_allows_only_validated_ibl_bundle_targets(self) -> None:
        source = DURABLE.read_text(encoding="utf-8")

        self.assertIn("IblSourceCubemapStagingStore", source)
        self.assertIn("ibl_bundle_store", source)
        self.assertIn("validate_bundle_target(target.operation_path())", source)


if __name__ == "__main__":
    unittest.main()
