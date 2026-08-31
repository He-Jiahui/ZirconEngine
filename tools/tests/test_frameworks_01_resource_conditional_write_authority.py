import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
RUNTIME_SOURCE = REPO_ROOT / "zircon_runtime/src"
EDITOR_SOURCE = REPO_ROOT / "zircon_editor/src"
ZR_RESOURCE_SOURCE = REPO_ROOT / "zircon_runtime/crates/zr_resource/src"
RESOURCE_FACADE = RUNTIME_SOURCE / "core/resource/io/mod.rs"
RESOURCE_IO_OWNER = ZR_RESOURCE_SOURCE / "io/mod.rs"
ATOMIC_FILE = ZR_RESOURCE_SOURCE / "io/atomic_file/mod.rs"
ATOMIC_FILE_OWNER = ZR_RESOURCE_SOURCE / "io/atomic_file"
ATOMIC_TESTS = ZR_RESOURCE_SOURCE / "io/atomic_file/tests/mod.rs"
TOOLKIT_REGISTRY = EDITOR_SOURCE / "core/extension/toolkit/registry.rs"
SOURCE_AUTHORITY = (
    EDITOR_SOURCE / "core/extension/toolkit/save/source_write_authority.rs"
)
SOURCE_AUTHORITY_TESTS = (
    EDITOR_SOURCE / "core/extension/toolkit/save/source_write_authority/tests.rs"
)
PROJECT_PATH_IDENTITY = RUNTIME_SOURCE / "asset/project/paths/identity.rs"
PROJECT_PATH_WINDOWS = RUNTIME_SOURCE / "asset/project/paths/windows.rs"
PROJECT_PATHS = RUNTIME_SOURCE / "asset/project/paths.rs"
PROJECT_PATH_TESTS = RUNTIME_SOURCE / "asset/project/paths/tests.rs"
PROJECT_META_WRITE_AUTHORITY = (
    RUNTIME_SOURCE / "asset/project/meta_write_authority.rs"
)
RESOURCE_TRANSACTION = ZR_RESOURCE_SOURCE / "io/transaction"
PROJECT_IDENTITY_CONSUMERS = (
    RUNTIME_SOURCE / "asset/project/manager/durable_transaction.rs",
    RUNTIME_SOURCE / "asset/project/manager/scan_and_import.rs",
    RUNTIME_SOURCE / "asset/project/manager/scan_and_import/full_generation.rs",
    RUNTIME_SOURCE / "asset/project/manager/scan_and_import/targeted.rs",
    RUNTIME_SOURCE / "asset/project/manager/relocation.rs",
    RUNTIME_SOURCE / "asset/project/meta_preview_state.rs",
    PROJECT_META_WRITE_AUTHORITY,
)
UI_ASSET_SAVE = EDITOR_SOURCE / "ui/host/asset_editor_sessions/save.rs"


def rust_sources(root: Path) -> list[Path]:
    return sorted(root.rglob("*.rs"))


class ResourceConditionalWriteAuthorityTests(unittest.TestCase):
    def test_cas_like_resource_api_is_hard_cut_from_runtime_and_editor(self) -> None:
        forbidden = ("atomic_write_if_unchanged", "AtomicWriteCompare")
        violations = []
        for path in rust_sources(RUNTIME_SOURCE) + rust_sources(EDITOR_SOURCE):
            source = path.read_text(encoding="utf-8")
            for symbol in forbidden:
                if symbol in source:
                    violations.append(
                        f"{path.relative_to(REPO_ROOT).as_posix()}: {symbol}"
                    )
        self.assertEqual([], violations)

        facade = RESOURCE_FACADE.read_text(encoding="utf-8")
        self.assertIn(
            "pub use zr_resource::io::{atomic_write, atomic_write_new};", facade
        )
        resource_io_owner = RESOURCE_IO_OWNER.read_text(encoding="utf-8")
        self.assertIn(
            "pub use atomic_file::{atomic_write, atomic_write_new};", resource_io_owner
        )
        atomic_file = ATOMIC_FILE.read_text(encoding="utf-8")
        self.assertIn("pub fn atomic_write(", atomic_file)
        self.assertIn("pub fn atomic_write_new(", atomic_file)

    def test_toolkit_registry_owns_one_normalized_source_write_authority(self) -> None:
        registry = TOOLKIT_REGISTRY.read_text(encoding="utf-8")
        self.assertIn("source_writes: DocumentSourceWriteAuthority", registry)
        self.assertIn("self.source_writes.acquire(project_root, source_path)", registry)

        authority = SOURCE_AUTHORITY.read_text(encoding="utf-8")
        required_contract = (
            "active_sources: Mutex<BTreeSet<ResolvedProjectPathIdentity>>",
            "source_released: Condvar",
            "ProjectPaths::resolve_existing(project_root)",
            "if !project_root.operation_path().is_dir()",
            "ProjectPaths::resolve_path_from(&project_root, source_path)",
            "source_identity.is_within(&project_identity)",
            "self.source_path.operation_path()",
            "ensure_source_is_writable(source_path)",
            "publisher(source_path, replacement)",
            "DocumentSourceWriteOutcome::DurableBestEffort",
        )
        for fragment in required_contract:
            with self.subTest(fragment=fragment):
                self.assertIn(fragment, authority)

    def test_same_source_test_proves_wait_then_release(self) -> None:
        authority_tests = SOURCE_AUTHORITY_TESTS.read_text(encoding="utf-8")
        test_start = authority_tests.index(
            "fn same_source_waits_until_the_current_write_lease_is_released()"
        )
        test_end = authority_tests.index("\n#[test]", test_start + 1)
        test_source = authority_tests[test_start:test_end]

        self.assertIn("acquire_with_wait_hook", test_source)
        self.assertIn("must enter the Condvar wait path", test_source)
        self.assertIn("drop(active);", test_source)
        self.assertLess(
            test_source.index("must enter the Condvar wait path"),
            test_source.index("drop(active);"),
        )

        stale_start = authority_tests.index(
            "fn stale_save_after_a_cooperating_external_effect_reports_source_changed()"
        )
        stale_end = authority_tests.index("\n#[test]", stale_start + 1)
        stale_source = authority_tests[stale_start:stale_end]
        self.assertRegex(stale_source, r"result_receive\s*\.recv_timeout")
        release = stale_source.index("drop(external_effect);")
        self.assertLess(release, stale_source.index("result_receive", release))

    def test_uncreated_windows_case_aliases_share_one_physical_identity(self) -> None:
        authority = SOURCE_AUTHORITY.read_text(encoding="utf-8")
        authority_tests = SOURCE_AUTHORITY_TESTS.read_text(encoding="utf-8")
        self.assertIn(
            "active_sources: Mutex<BTreeSet<ResolvedProjectPathIdentity>>",
            authority,
        )
        self.assertIn(
            "uncreated_windows_case_aliases_share_one_write_lease",
            authority_tests,
        )
        self.assertTrue(PROJECT_PATH_IDENTITY.is_file())
        identity = PROJECT_PATH_IDENTITY.read_text(encoding="utf-8")
        self.assertIn("pub struct ResolvedProjectPathIdentity", identity)
        self.assertIn("impl Ord for ResolvedProjectPathIdentity", identity)
        self.assertIn("pub fn is_within", identity)
        self.assertIn("windows::compare_paths_ignore_case", identity)
        self.assertNotIn("to_lowercase", identity)
        self.assertNotIn("impl Hash for ResolvedProjectPathIdentity", identity)
        windows = PROJECT_PATH_WINDOWS.read_text(encoding="utf-8")
        self.assertIn("CompareStringOrdinal", windows)
        self.assertIn("// SAFETY:", windows)

    def test_project_identity_has_no_lossy_legacy_key(self) -> None:
        project_paths = PROJECT_PATHS.read_text(encoding="utf-8")
        self.assertNotIn("filesystem_identity_key", project_paths)
        self.assertNotIn("to_string_lossy().replace", project_paths)

        for path in PROJECT_IDENTITY_CONSUMERS:
            with self.subTest(path=path.relative_to(REPO_ROOT).as_posix()):
                source = path.read_text(encoding="utf-8")
                self.assertNotIn("filesystem_identity_key", source)

        self.assertTrue(PROJECT_META_WRITE_AUTHORITY.is_file())
        meta_authority = PROJECT_META_WRITE_AUTHORITY.read_text(encoding="utf-8")
        self.assertIn("BTreeSet<ResolvedProjectPathIdentity>", meta_authority)
        self.assertIn("waiters: VecDeque<MetaWriteWaiter>", meta_authority)
        self.assertIn("state_changed: Condvar", meta_authority)
        self.assertIn("next_ticket: u64", meta_authority)
        self.assertIn("earlier_waiter_conflicts", meta_authority)
        self.assertNotIn("DefaultHasher", meta_authority)
        self.assertNotIn("META_WRITE_STRIPE", meta_authority)
        self.assertIn("one_resolved_meta_identity_admits_only_one_writer", meta_authority)
        self.assertIn(
            "unrelated_meta_identities_do_not_share_a_false_lock_stripe",
            meta_authority,
        )
        self.assertIn(
            "earlier_conflicting_multi_path_waiter_cannot_be_barged_by_a_later_writer",
            meta_authority,
        )
        self.assertIn("later_waiting_receive", meta_authority)
        self.assertIn(
            "later_disjoint_writer_can_pass_an_earlier_blocked_waiter",
            meta_authority,
        )

        durable = PROJECT_IDENTITY_CONSUMERS[0].read_text(encoding="utf-8")
        prepared_writes = PROJECT_IDENTITY_CONSUMERS[1].read_text(encoding="utf-8")
        targeted = PROJECT_IDENTITY_CONSUMERS[3].read_text(encoding="utf-8")
        self.assertIn("artifact_root: ResolvedProjectPathIdentity", durable)
        self.assertIn("registry_root: ResolvedProjectPathIdentity", durable)
        self.assertIn("recovery_parent_identity", durable)
        self.assertIn("is_relocatable_project_entry", durable)
        self.assertIn(
            "self.is_registry_entry(document.target(), &target)?", durable
        )
        self.assertIn(
            "self.is_artifact_manifest_entry(document.target(), &target)?", durable
        )
        self.assertIn(
            "self.is_import_source_plan_entry(document.target(), &target)?", durable
        )
        self.assertIn(
            ".validate_bundle_target(target.operation_path())", durable
        )
        self.assertIn("is_artifact_manifest_relative", durable)
        self.assertIn("is_import_source_plan_relative", durable)
        self.assertIn("path.relative_to(root)", durable)
        self.assertIn("BTreeMap<ResolvedProjectPathIdentity, usize>", prepared_writes)
        self.assertIn("let mut unique_meta_paths = BTreeMap::new()", targeted)

        self.assertIn(
            "resolve_identity_rejects_a_broken_symlink_in_the_uncreated_tail",
            PROJECT_PATH_TESTS.read_text(encoding="utf-8"),
        )
        self.assertIn("fs::metadata(&candidate)", project_paths)
        self.assertIn("fs::symlink_metadata(&candidate)", project_paths)
        self.assertIn(
            "split_at_deepest_existing_project_ancestor(&absolute)?", project_paths
        )
        self.assertIn(
            '"project path has no accessible physical ancestor: {}"', project_paths
        )

    def test_path_and_editor_authority_tests_are_folder_backed(self) -> None:
        source_authority = SOURCE_AUTHORITY.read_text(encoding="utf-8")
        for path, source in (
            (PROJECT_PATHS, PROJECT_PATHS.read_text(encoding="utf-8")),
            (SOURCE_AUTHORITY, source_authority),
        ):
            with self.subTest(path=path.relative_to(REPO_ROOT).as_posix()):
                self.assertLess(len(source.splitlines()), 800)
                self.assertIn("#[cfg(test)]\nmod tests;", source)

        self.assertTrue(PROJECT_PATH_TESTS.is_file())
        self.assertTrue(SOURCE_AUTHORITY_TESTS.is_file())
        project_path_tests = PROJECT_PATH_TESTS.read_text(encoding="utf-8")
        self.assertIn('var_os("ZIRCON_TEST_OUTPUT_ROOT")', project_path_tests)
        self.assertIn('var_os("CARGO_TARGET_DIR")', project_path_tests)
        self.assertNotIn(
            "std::env::temp_dir().join(format!", project_path_tests
        )

    def test_all_project_document_writers_route_through_the_source_authority(self) -> None:
        animation_save = (
            EDITOR_SOURCE / "ui/host/animation_editor_sessions/save.rs"
        ).read_text(encoding="utf-8")
        external_effects = (
            EDITOR_SOURCE / "ui/host/asset_editor_sessions/editing.rs"
        ).read_text(encoding="utf-8")

        for source in (animation_save, external_effects):
            self.assertIn(".with_source_write", source)
            self.assertNotIn("atomic_write(", source)
        self.assertNotIn("fs::remove_file", external_effects)
        self.assertIn("remove_if_exists", external_effects)
        self.assertIn("disk_source", animation_save)

        self.assertNotIn("})??;", external_effects)
        self.assertEqual(
            4,
            external_effects.count(
                ".map_err(|source| EditorError::UiAssetSaveIo"
            ),
            "authority admission and the nested filesystem result need explicit typed errors",
        )

    def test_post_publication_error_and_save_guarantee_are_explicit(self) -> None:
        authority = SOURCE_AUTHORITY.read_text(encoding="utf-8")
        authority_tests = SOURCE_AUTHORITY_TESTS.read_text(encoding="utf-8")
        authority_production = authority.split("\n#[cfg(test)]\nmod tests", maxsplit=1)[0]
        context = (
            EDITOR_SOURCE / "core/extension/toolkit/save/context.rs"
        ).read_text(encoding="utf-8")
        report = (
            EDITOR_SOURCE / "core/extension/toolkit/save/report.rs"
        ).read_text(encoding="utf-8")
        ui_save = UI_ASSET_SAVE.read_text(encoding="utf-8")

        self.assertIn("PublishedNotDurable", authority)
        self.assertIn("commit_if_matches_with_publisher", authority)
        self.assertIn("fs::read(source_path)", authority)
        self.assertIn("DocumentSourceWriteReceipt", authority_production)
        self.assertIn("source_before_publication", authority_production)
        self.assertIn("SourceBeforePublication::Unknown", authority_production)
        self.assertIn("replace_with_publisher_and_observer", authority_production)
        self.assertIn(
            "unknown_prepublication_observation_does_not_block_a_successful_replace",
            authority_tests,
        )
        self.assertIn(
            "matching_bytes_do_not_prove_that_a_failed_publisher_replaced_the_source",
            authority_tests,
        )
        self.assertIn("recv_timeout", authority_tests)
        self.assertIn("record_serialized_project_source_write", context)
        self.assertNotIn("pub fn record_serialized_project_source_write", context)
        self.assertIn("receipt: DocumentSourceWriteReceipt", context)
        self.assertIn("cooperating_source_writes_are_serialized", report)
        self.assertIn("external_conflict_detection_is_best_effort", report)
        self.assertIn("write_outcome.into_publication()", ui_save)
        self.assertIn("DocumentSourceWritePublication::Durable(receipt)", ui_save)
        self.assertIn("UiAssetSaveStage::DurabilityBarrier", ui_save)

    def test_ui_save_holds_source_lease_through_disk_baseline_commit(self) -> None:
        save = UI_ASSET_SAVE.read_text(encoding="utf-8")
        acquire = save.index(".with_source_write(project.paths().root(), &source_path")
        publish = save.index(".commit_if_matches(expected_source.as_bytes(), saved.as_bytes())")
        baseline = save.index("entry.update_disk_baseline(saved.clone())")
        persisted = save.index(".mark_canonical_source_persisted(source_revision, saved.clone())")
        release = save.index("\n            })\n            .map_err", acquire)

        self.assertLess(acquire, publish)
        self.assertLess(publish, baseline)
        self.assertLess(baseline, persisted)
        self.assertLess(persisted, release)

    def test_atomic_new_contention_stages_both_writers_before_publication(self) -> None:
        tests = ATOMIC_TESTS.read_text(encoding="utf-8")
        test_start = tests.index(
            "fn concurrent_atomic_write_new_publication_has_exactly_one_winner()"
        )
        test_end = tests.index("\n#[test]", test_start + 1)
        test_source = tests[test_start:test_end]

        self.assertIn("Barrier::new(2)", test_source)
        self.assertIn("stage_atomic_write(path.as_path(), payload)", test_source)
        self.assertIn("pending.commit_new()", test_source)
        self.assertIn("std::io::ErrorKind::AlreadyExists", test_source)
        self.assertIn("outcome.is_ok()", test_source)

    def test_atomic_file_presence_and_backup_allocation_fail_closed(self) -> None:
        production_sources = [
            path
            for path in rust_sources(ATOMIC_FILE_OWNER)
            if path.name != "tests.rs" and "tests" not in path.parts
        ]
        violations = []
        for path in production_sources:
            source = path.read_text(encoding="utf-8")
            if ".exists()" in source or (
                path != ATOMIC_FILE and ".is_dir()" in source
            ):
                violations.append(path.relative_to(REPO_ROOT).as_posix())
        self.assertEqual([], violations)

        atomic_file = ATOMIC_FILE.read_text(encoding="utf-8")
        self.assertIn("pub(super) enum PathEntry", atomic_file)
        self.assertIn("fs::symlink_metadata(path)", atomic_file)
        self.assertIn(
            "Err(error) if error.kind() == io::ErrorKind::NotFound", atomic_file
        )
        self.assertIn("Err(error) => Err(error)", atomic_file)

        transaction = (ATOMIC_FILE_OWNER / "transaction.rs").read_text(
            encoding="utf-8"
        )
        platform = (ATOMIC_FILE_OWNER / "platform.rs").read_text(encoding="utf-8")
        self.assertNotIn("fs::copy(", transaction)
        self.assertIn("copy_file_create_new", transaction)
        self.assertIn("create_backup_file_new", transaction)
        self.assertIn("error.kind() == io::ErrorKind::AlreadyExists", transaction)
        self.assertIn("platform::replace_existing_staged_file", transaction)
        self.assertNotIn("platform::replace_file_with_backup", transaction)
        self.assertIn("enum StagedPublicationState", transaction)
        self.assertIn("NotPublished", transaction)
        self.assertIn("MayHavePublished", transaction)
        self.assertIn("publish_staged_file_for_transaction", transaction)
        self.assertNotIn(
            "MOVEFILE_REPLACE_EXISTING", platform
        )
        self.assertIn(
            "#[cfg(windows)]\nfn publish_new_staging_observed(", transaction
        )
        self.assertIn(
            "return publish_new_staging(staging, target, AtomicWriteFault::None);",
            transaction,
        )
        self.assertIn(
            "return publish_new_staging_observed(staging, target, AtomicWriteFault::None);",
            transaction,
        )
        self.assertNotIn("REPLACEFILE_WRITE_THROUGH", platform)
        self.assertIn("std::ptr::null(),\n            0,", platform)

        recovery = (ATOMIC_FILE_OWNER / "recovery.rs").read_text(encoding="utf-8")
        self.assertIn("is_atomic_write_transaction_path", recovery)
        self.assertIn("path_entry(&candidate)?", recovery)

        tests = ATOMIC_TESTS.read_text(encoding="utf-8")
        self.assertIn(
            "atomic_file_presence_treats_only_not_found_as_missing", tests
        )
        self.assertIn(
            "backup_create_new_never_overwrites_existing_evidence", tests
        )
        self.assertIn(
            "concurrent_expected_missing_transaction_publication_has_exactly_one_winner",
            tests,
        )
        self.assertIn("for round in 0..32", tests)
        self.assertIn(
            "publish_staged_file_for_transaction(&staging, target.as_path(), false)",
            tests,
        )
        self.assertIn("recovery_ignores_unreserved_backup_lookalikes", tests)
        self.assertIn("recovery_rejects_non_file_reserved_backup", tests)

        durable_commit = (RESOURCE_TRANSACTION / "commit.rs").read_text(
            encoding="utf-8"
        )
        self.assertIn("staged.target_existed", durable_commit)
        self.assertIn("error.rollback_required()", durable_commit)
        self.assertIn(
            "prepublication_conflict_preserves_external_target_and_skips_rollback",
            durable_commit,
        )

    def test_durable_transaction_presence_checks_fail_closed(self) -> None:
        production_sources = [
            path
            for path in rust_sources(RESOURCE_TRANSACTION)
            if path.name != "tests.rs" and "tests" not in path.parts
        ]
        violations = []
        for path in production_sources:
            source = path.read_text(encoding="utf-8")
            if ".exists()" in source:
                violations.append(path.relative_to(REPO_ROOT).as_posix())
        self.assertEqual([], violations)

        stage = (RESOURCE_TRANSACTION / "stage.rs").read_text(encoding="utf-8")
        self.assertIn("pub(super) enum FilePresence", stage)
        self.assertIn("pub(super) fn file_presence", stage)
        self.assertIn("Err(error) if error.kind() == io::ErrorKind::NotFound", stage)
        self.assertIn("Err(error) => Err(error)", stage)
        self.assertIn("file_presence_treats_only_not_found_as_missing", stage)

        engine_tests = "\n".join(
            path.read_text(encoding="utf-8")
            for path in (
                RESOURCE_TRANSACTION / "engine/tests.rs",
                *rust_sources(RESOURCE_TRANSACTION / "engine/tests"),
            )
        )
        self.assertNotIn('.join("target/zircon-test-output")', engine_tests)
        self.assertIn('std::env::var_os("ZIRCON_TEST_OUTPUT_ROOT")', engine_tests)
        self.assertIn('std::env::var_os("CARGO_TARGET_DIR")', engine_tests)

        evidence = (RESOURCE_TRANSACTION / "recovery/evidence.rs").read_text(
            encoding="utf-8"
        )
        self.assertIn("file_presence", evidence)
        self.assertIn("FilePresence::Missing", evidence)
        self.assertIn("FilePresence::Present", evidence)

    def test_restart_recovery_attempts_every_document_after_restore_failure(self) -> None:
        replay = (RESOURCE_TRANSACTION / "recovery/replay.rs").read_text(
            encoding="utf-8"
        )
        recovery_tests = (RESOURCE_TRANSACTION / "recovery/tests.rs").read_text(
            encoding="utf-8"
        )

        self.assertIn("let mut first_restore_error = None;", replay)
        self.assertIn("if let Err(source) = restore(document)", replay)
        self.assertIn("if let Some(error) = first_restore_error", replay)
        self.assertIn(
            "recovery_attempts_every_document_after_a_restore_failure",
            recovery_tests,
        )
        self.assertIn(
            "assert_eq!(&*restore_attempts.borrow(), &[2, 1, 0]);",
            recovery_tests,
        )

    def test_durable_intent_publication_never_replaces_existing_journal(self) -> None:
        intent = (RESOURCE_TRANSACTION / "journal/intent.rs").read_text(
            encoding="utf-8"
        )
        journal_tests = (RESOURCE_TRANSACTION / "journal/tests.rs").read_text(
            encoding="utf-8"
        )

        self.assertIn("use crate::io::{atomic_write_new, sync_parent_directory};", intent)
        self.assertIn("atomic_write_new(path, &frame)", intent)
        self.assertNotIn("atomic_write(path, &frame)", intent)
        self.assertIn(
            "immutable_intent_does_not_replace_an_existing_journal",
            journal_tests,
        )

    def test_pre_active_cleanup_remains_restart_recoverable(self) -> None:
        schema = (RESOURCE_TRANSACTION / "schema.rs").read_text(encoding="utf-8")
        engine = (RESOURCE_TRANSACTION / "engine.rs").read_text(encoding="utf-8")
        engine_tests = "\n".join(
            path.read_text(encoding="utf-8")
            for path in (
                RESOURCE_TRANSACTION / "engine/tests.rs",
                *rust_sources(RESOURCE_TRANSACTION / "engine/tests"),
            )
        )
        replay = (RESOURCE_TRANSACTION / "recovery/replay.rs").read_text(
            encoding="utf-8"
        )
        recovery_tests = (RESOURCE_TRANSACTION / "recovery/tests.rs").read_text(
            encoding="utf-8"
        )

        self.assertIn("CleanupIntent", schema)
        transition = engine.index("record_phase(journal, JournalPhase::CleanupIntent)")
        artifacts = engine.index("cleanup_intents(intents)", transition)
        journal_delete = engine.index("remove_reserved_if_exists(journal)", artifacts)
        append_guard = engine.index("if !journal_append_safe")
        self.assertLess(append_guard, transition)
        self.assertLess(transition, artifacts)
        self.assertLess(artifacts, journal_delete)
        commit = engine.index("pub fn commit_prepared_files")
        intent_plan = engine.index("plan_intent(", commit)
        journal_owner = engine.index("ensure_journal_directory(", intent_plan)
        owner_lock = engine.index("TransactionOwnerLock::acquire(", journal_owner)
        pending_recovery = engine.index("reject_pending_recovery(", owner_lock)
        intent_persist = engine.index("persist_intent(", pending_recovery)
        self.assertLess(intent_plan, journal_owner)
        self.assertLess(journal_owner, owner_lock)
        self.assertLess(owner_lock, pending_recovery)
        self.assertLess(pending_recovery, intent_persist)
        self.assertIn(
            "pre_active_abort_uses_journal_first_cleanup_for_uncertain_tail",
            engine_tests,
        )
        self.assertIn(
            "pre_active_abort_keeps_journal_when_artifact_cleanup_fails",
            engine_tests,
        )
        self.assertIn(
            "pre_active_abort_preserves_original_operation_when_cleanup_transition_fails",
            engine_tests,
        )

        self.assertIn("JournalPhase::Intent => recover_intent_journal", replay)
        self.assertIn("record_phase(JournalPhase::CleanupIntent).is_err()", replay)
        self.assertIn("cleanup_documents_journal_first(path, documents)", replay)
        self.assertIn(
            "intent_recovery_does_not_cleanup_after_transition_append_failure",
            recovery_tests,
        )


if __name__ == "__main__":
    unittest.main()
