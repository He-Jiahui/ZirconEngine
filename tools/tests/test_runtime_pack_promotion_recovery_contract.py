import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
INSTALL = ROOT / "zircon_runtime/src/asset/pack/install"
PROMOTION = INSTALL / "promotion.rs"
JOURNAL = INSTALL / "promotion_journal.rs"
WORKFLOW = INSTALL / "delta_workflow.rs"
INSTALLER = INSTALL / "installer.rs"
RECEIPT = INSTALL / "receipt_io.rs"
PROMOTION_TESTS = INSTALL / "promotion_tests.rs"
HOT_UPDATE = (
    ROOT
    / "zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/hot_update_application.rs"
)


class RuntimePackPromotionRecoveryContractTests(unittest.TestCase):
    def test_promotion_is_a_digest_checked_durable_transaction(self) -> None:
        source = PROMOTION.read_text(encoding="utf-8")

        for anchor in (
            "commit_prepared_files",
            "PreparedFileWrite",
            "let staged_digest = blake3::hash(&staged_bytes).to_hex().to_string();",
            ".retiring_with_expected_digest(&paths.staged, staged_digest)",
            "super::promotion_journal::TRANSACTION_TAG",
            "DurableCommitDisposition::CommitRecoveryDeferred",
            "ZrPackDeltaInstallError::RecoveryRequired",
        ):
            self.assertIn(anchor, source)

        self.assertLess(source.index("paths.recover()?"), source.index("read_pack_file(&paths.staged)?"))
        self.assertLess(
            source.index("let staged_digest"),
            source.rindex("commit_prepared_files("),
        )

    def test_promotion_paths_bound_recovery_to_requested_files(self) -> None:
        source = JOURNAL.read_text(encoding="utf-8")

        for anchor in (
            "ProjectPaths::resolve_root",
            "paths_alias(&staged, &installed)?",
            "staged, installed and backup packs must have distinct identities",
            'journal_name.push(".promotion-journal")',
            "recover_pending_transactions",
            "struct PromotionRecoveryPolicy",
            "pack promotion journal target",
            "document.retired_paths()",
        ):
            self.assertIn(anchor, source)

        validation = source.index("fn validate_document")
        self.assertLess(source.index("document.target() == paths.installed", validation), validation + 1200)
        self.assertLess(source.index("paths.backup.as_deref() == Some(document.target())", validation), validation + 1600)

    def test_delta_retry_recovers_before_rebuild_and_validates_backup(self) -> None:
        source = WORKFLOW.read_text(encoding="utf-8")

        for anchor in (
            "paths.recover()?",
            "ZrPackDeltaReader::from_bytes",
            "installed.manifest() == &delta.manifest().target",
            "ZrPackDeltaInstallError::BackupPackMismatch",
            "backup_reader.manifest() != &delta.manifest().base",
            "ZrPackPromotionMethod::AlreadyInstalled",
            "staging::rebuild_to_staging(base_pack, delta_pack, staged_pack)?",
            "promotion::promote_staged_pack(staged_pack, installed_pack, backup_pack)?",
        ):
            self.assertIn(anchor, source)

        self.assertLess(source.index("paths.recover()?"), source.index("if paths\n        .installed"))
        self.assertLess(source.index("backup_reader.manifest()"), source.index("ZrPackPromotionMethod::AlreadyInstalled"))

    def test_public_installer_exposes_recovery_and_composed_install(self) -> None:
        source = INSTALLER.read_text(encoding="utf-8")

        for anchor in (
            "pub fn recover_pending_promotion(",
            "PromotionPaths::new(",
            ".recover()",
            "pub fn install_delta(",
            "super::delta_workflow::install_delta(",
            "pub fn write_install_receipt(",
            "pub fn read_install_receipt(",
        ):
            self.assertIn(anchor, source)

    def test_receipts_are_atomic_and_cannot_alias_pack_paths(self) -> None:
        source = RECEIPT.read_text(encoding="utf-8")

        for anchor in (
            "validate_receipt_reports(staging_report, promotion_report)?",
            "paths_alias(receipt_path, pack_path)?",
            "receipt must not overwrite a pack input, installed pack or backup",
            "serde_json::to_vec_pretty(&receipt)",
            "crate::core::resource::io::atomic_write(receipt_path, &bytes)",
            "ZRPACK_INSTALL_RECEIPT_FORMAT_VERSION",
            "ReceiptDecode",
        ):
            self.assertIn(anchor, source)

    def test_native_hot_update_installs_pack_before_receipt_and_reload(self) -> None:
        source = HOT_UPDATE.read_text(encoding="utf-8")

        install = source.index("ZrPackDeltaInstaller::install_delta(")
        receipt = source.index("ZrPackDeltaInstaller::write_install_receipt", install)
        reload_call = source.index("hot_reload_runtime_plugins", receipt)

        self.assertLess(install, receipt)
        self.assertLess(receipt, reload_call)
        self.assertIn("zrpack delta install failed before hot update", source)

    def test_regression_matrix_covers_failure_restart_retry_and_backup_paths(self) -> None:
        source = PROMOTION_TESTS.read_text(encoding="utf-8")

        for test_name in (
            "pack_promotion_failure_without_backup_preserves_installed_bytes",
            "pack_promotion_restart_rolls_back_uncommitted_replacement_and_staging_retirement",
            "pack_promotion_restart_finishes_committed_publication",
            "pack_promotion_rejects_changed_recovery_authority_without_touching_files",
            "pack_promotion_rejects_aliased_destination_without_deleting_source",
            "pack_promotion_automatically_recovers_before_retry",
            "delta_install_restart_after_commit_resumes_receipt_without_overwriting_backup",
            "delta_install_restart_before_commit_recovers_base_before_rebuilding",
            "delta_install_rejects_new_backup_path_after_no_backup_commit",
            "delta_install_rejects_wrong_backup_after_committed_publication",
            "delta_install_rejects_corrupt_backup_after_committed_publication",
            "pack_promotion_corrupt_journal_preserves_installed_and_staged",
        ):
            self.assertIn(f"fn {test_name}(", source)


if __name__ == "__main__":
    unittest.main()
