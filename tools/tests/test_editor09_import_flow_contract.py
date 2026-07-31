import pathlib
import re
import unittest


REPO_ROOT = pathlib.Path(__file__).resolve().parents[2]
IMPORT_ROOT = REPO_ROOT / "zircon_editor/src/core/asset/import_flow"


def read(relative: str) -> str:
    return (REPO_ROOT / relative).read_text(encoding="utf-8")


def public_reexports(source: str, module: str) -> set[str]:
    match = re.search(
        rf"pub use {module}::(?P<body>\{{.*?\}}|[A-Za-z0-9_]+);",
        source,
        re.DOTALL,
    )
    if match is None:
        return set()
    body = match.group("body").strip("{}")
    return {token.strip() for token in body.split(",") if token.strip()}


class Editor09ImportFlowContractTests(unittest.TestCase):
    def test_asset_facade_mounts_and_reexports_import_flow(self) -> None:
        facade = read("zircon_editor/src/core/asset/mod.rs")

        self.assertIn("mod import_flow;", facade)
        self.assertNotIn("pub mod import_flow;", facade)
        self.assertEqual(
            public_reexports(facade, "import_flow"),
            {
            "EditorAssetImportAdmissionLimits",
            "EditorAssetImportFlow",
            "EditorAssetImportReason",
            "EditorAssetImportRequest",
            "EditorAssetImportResult",
            "EditorAssetImportSubmitError",
            "EditorAssetImportTicket",
            },
        )

        consumer = read("zircon_editor/tests/editor_asset_facade.rs")
        self.assertIn("use zircon_editor::core::asset::{", consumer)
        self.assertIn("EditorAssetImportFlow", consumer)
        self.assertIn("EditorAssetImportTicket", consumer)

    def test_import_flow_is_folder_backed_and_split_by_responsibility(self) -> None:
        for name in (
            "mod.rs",
            "error.rs",
            "flight.rs",
            "job.rs",
            "state.rs",
            "submit.rs",
            "tests.rs",
        ):
            self.assertTrue((IMPORT_ROOT / name).is_file(), name)
        self.assertTrue((IMPORT_ROOT / "tests/concurrency.rs").is_file())

        facade = read("zircon_editor/src/core/asset/import_flow/mod.rs")
        self.assertIn("mod error;", facade)
        self.assertIn("mod job;", facade)
        self.assertIn("mod state;", facade)
        self.assertLess(len(facade.splitlines()), 360)

    def test_flow_reuses_runtime_asset_manager_and_plan14_jobs(self) -> None:
        facade = read("zircon_editor/src/core/asset/import_flow/mod.rs")
        job = read("zircon_editor/src/core/asset/import_flow/job.rs")
        submit = read("zircon_editor/src/core/asset/import_flow/submit.rs")
        combined = facade + job + submit

        self.assertIn("AssetManager", combined)
        self.assertIn("import_asset", combined)
        self.assertIn("impl EditorJob for", job)
        self.assertIn("JobCategory::Import", submit)
        self.assertIn("with_mutex_group", submit)

    def test_flow_does_not_create_parallel_asset_authority(self) -> None:
        combined = "\n".join(
            read(f"zircon_editor/src/core/asset/import_flow/{name}")
            for name in ("mod.rs", "error.rs", "job.rs", "state.rs", "submit.rs")
        )

        for forbidden in (
            "AssetWorkerPool::new",
            "std::thread::spawn",
            "thread::spawn",
            ".zmeta",
            "source_digest =",
            "blake3::",
            "AssetRegistryIndex::from_entries",
        ):
            self.assertNotIn(forbidden, combined)

    def test_generation_single_flight_and_admission_are_bounded(self) -> None:
        facade = read("zircon_editor/src/core/asset/import_flow/mod.rs")
        flight = read("zircon_editor/src/core/asset/import_flow/flight.rs")
        state = read("zircon_editor/src/core/asset/import_flow/state.rs")
        job = read("zircon_editor/src/core/asset/import_flow/job.rs")
        submit = read("zircon_editor/src/core/asset/import_flow/submit.rs")
        index = read("zircon_editor/src/core/asset/index.rs")

        for contract in (
            "EditorAssetImportReason",
            "EditorAssetImportRequest",
            "EditorAssetImportResult",
            "EditorAssetImportTicket",
        ):
            self.assertIn(contract, facade)
        self.assertIn("EditorAssetImportAdmissionLimits", facade)
        self.assertIn("ImportGenerationKey", state)
        self.assertIn("source_digest: Arc<str>", state)
        self.assertNotIn("DefaultHasher", state)
        self.assertIn("flights", state)
        self.assertIn("active_by_uuid", state)
        self.assertIn("admission_bytes", state)
        self.assertIn("oldest", state)
        self.assertIn("MutexGroup", state)
        self.assertIn("UuidImportPhase", state)
        self.assertIn("UuidLifecycleToken", state)
        self.assertIn("complete_uuid_clear", state)
        self.assertIn("ImportFlight", flight)
        self.assertIn("Condvar", flight)
        self.assertIn("wait_admission", flight)
        self.assertIn("publish_admission", flight)
        self.assertIn("#[derive(Clone)]\npub struct EditorAssetImportTicket", facade)
        self.assertNotIn("JobTicket<EditorAssetImportResult>", facade)
        self.assertIn("impl Drop for ImportLease", job)
        self.assertLess(job.index("self.state.finish"), job.index("self.flight.complete"))
        self.assertIn("begin_import", submit)
        self.assertIn("clear_import", job)
        self.assertIn("EditorAssetImportGeneration", index)
        self.assertIn("import_generation", index)
        self.assertIn("is_current_import_generation", index)

    def test_submission_and_execution_failures_stay_typed(self) -> None:
        error = read("zircon_editor/src/core/asset/import_flow/error.rs")
        job = read("zircon_editor/src/core/asset/import_flow/job.rs")

        self.assertIn("EditorAssetImportSubmitError", error)
        self.assertIn("AssetNotIndexed", error)
        self.assertIn("EditorAssetIndexError", error)
        self.assertIn("JobSubmitError", error)
        self.assertIn("JobError::failed", job)

    def test_import_flow_uses_workspace_edition_2021_syntax(self) -> None:
        combined = "\n".join(
            path.read_text(encoding="utf-8")
            for path in sorted(IMPORT_ROOT.rglob("*.rs"))
        )

        self.assertIsNone(
            re.search(r"if let [^\n]+\n\s*&&", combined),
            "let-chain syntax requires edition 2024 but this workspace is edition 2021",
        )

    def test_rust_contract_covers_success_failure_serialization_and_cancel(self) -> None:
        tests = "\n".join(
            (
                read("zircon_editor/src/core/asset/import_flow/tests.rs"),
                read("zircon_editor/src/core/asset/import_flow/tests/concurrency.rs"),
            )
        )

        for test_name in (
            "successful_import_uses_runtime_backend_and_clears_importing_state",
            "backend_failure_is_typed_and_clears_importing_state",
            "unknown_uri_is_rejected_before_job_submission",
            "duplicate_generation_storm_shares_one_job_and_merges_reasons",
            "failed_generation_can_be_submitted_again",
            "admission_limits_bound_entries_bytes_and_oldest_age",
            "shared_flight_cancel_releases_importing_once",
            "admission_waiter_observes_original_fast_failure",
            "registry_generation_change_retries_before_job_submission",
            "uuid_import_lifecycle_blocks_start_and_stale_clear_boundaries",
            "completed_generation_expires_even_under_hot_key_reuse",
            "completed_result_bytes_are_reclaimed_before_new_admission",
            "backend_panic_releases_import_lifecycle",
            "shutdown_submission_rejection_releases_import_lifecycle",
            "import_job_publishes_zero_to_one_progress_sequence",
            "uuid_importing_survives_registry_path_migration_until_all_uri_jobs_finish",
        ):
            self.assertIn(f"fn {test_name}", tests)


if __name__ == "__main__":
    unittest.main()
