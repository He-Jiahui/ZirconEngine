"""Static contracts for the Runtime-owned compound model import boundary."""

from pathlib import Path
import unittest


REPO_ROOT = Path(__file__).resolve().parents[2]


class RuntimeModelImportReceiptContractTests(unittest.TestCase):
    def test_runtime_owns_external_model_source_plan(self) -> None:
        source_plan = (
            REPO_ROOT
            / "zircon_runtime/src/asset/project/manager/scan_and_import/source_plan.rs"
        )
        self.assertTrue(source_plan.is_file())
        source = source_plan.read_text(encoding="utf-8")
        self.assertIn("struct ImportSourcePlan", source)
        self.assertIn("prepare_model_import_source", source)
        self.assertIn("PreparedFileWrite::new", source)
        self.assertIn("source_file_snapshots", source)
        self.assertIn("model import does not support .gltf", source)
        self.assertIn("validate_model_source_extension", source)
        self.assertIn("resolve_source_path_for_uri", source)

    def test_model_import_uses_one_runtime_receipt_and_resource_batch(self) -> None:
        contract = (
            REPO_ROOT
            / "zircon_runtime/src/asset/pipeline/manager/asset_manager/asset_manager.rs"
        ).read_text(encoding="utf-8")
        implementation = (
            REPO_ROOT
            / "zircon_runtime/src/asset/pipeline/manager/service_contracts/asset_manager_contract.rs"
        ).read_text(encoding="utf-8")
        publication = (
            REPO_ROOT
            / "zircon_runtime/src/asset/pipeline/manager/project_asset_manager/resource_publication.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("fn import_model_source", contract)
        self.assertIn("ProjectImportReceipt", contract)
        self.assertIn("fn import_model_source", implementation)
        self.assertIn("prepare_compound_project_resource_sync", implementation)
        self.assertIn("commit_compound_project_resource_sync", implementation)
        self.assertIn("PreparedCompoundProjectResourceSync", publication)
        self.assertIn("removed_locators.sort", publication)
        self.assertIn("record_updates.sort_by", publication)

    def test_model_transaction_suppresses_its_own_watcher_echoes(self) -> None:
        implementation = (
            REPO_ROOT
            / "zircon_runtime/src/asset/pipeline/manager/service_contracts/asset_manager_contract.rs"
        ).read_text(encoding="utf-8")
        runtime = (
            REPO_ROOT
            / "zircon_runtime/src/asset/pipeline/manager/project_asset_manager/runtime.rs"
        ).read_text(encoding="utf-8")
        echoes = (
            REPO_ROOT
            / "zircon_runtime/src/asset/pipeline/manager/project_asset_manager/source_write_watch_echo.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("register_transaction_watch_echoes(source_watch_echoes)", implementation)
        self.assertIn("lock_transaction_watch_echoes().filter(changes)", runtime)
        self.assertIn("current_hash == Some(echo.content_hash())", echoes)
        self.assertIn("let Some(echo) = self.entries.get(&change.uri).cloned()", echoes)
        self.assertIn("echo.source_uri().clone()", echoes)

    def test_obj_material_sidecar_is_read_from_the_import_snapshot(self) -> None:
        context = (
            REPO_ROOT / "zircon_runtime/src/asset/importer/contract.rs"
        ).read_text(encoding="utf-8")
        obj_importer = (
            REPO_ROOT / "zircon_runtime/src/asset/importer/ingest/import_obj.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("with_source_file_snapshots", context)
        self.assertIn("source_file_snapshot", context)
        self.assertIn("source_file_snapshot", obj_importer)
        self.assertIn("tobj::load_mtl_buf", obj_importer)

    def test_recovery_accepts_only_source_plan_model_targets(self) -> None:
        recovery = (
            REPO_ROOT
            / "zircon_runtime/src/asset/project/manager/durable_transaction.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("is_import_source_plan_target", recovery)
        self.assertIn("models", recovery)
        self.assertIn('"obj" | "glb" | "mtl"', recovery)

    def test_retained_host_no_longer_stages_or_imports_models_directly(self) -> None:
        workspace = (
            REPO_ROOT / "zircon_editor/src/ui/retained_host/app/assets/workspace.rs"
        ).read_text(encoding="utf-8")
        helpers = (
            REPO_ROOT / "zircon_editor/src/ui/retained_host/app/helpers.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("submit_model_import", workspace)
        self.assertIn("pending_model_import", workspace)
        self.assertIn("try_take", workspace)
        self.assertNotIn("import_model_source", workspace)
        self.assertNotIn("stage_model_source", workspace)
        self.assertNotIn(".import_asset(&model_uri.to_string())", workspace)
        self.assertNotIn("pub(crate) use model_staging::stage_model_source", helpers)

    def test_committed_model_receipt_refreshes_the_editor_catalog_without_watcher_echoes(self) -> None:
        workspace = (
            REPO_ROOT / "zircon_editor/src/ui/retained_host/app/assets/workspace.rs"
        ).read_text(encoding="utf-8")
        completion = workspace.split("fn complete_model_import(", 1)[1].split(
            "pub(in crate::ui::retained_host::app) fn sync_asset_workspace", 1
        )[0]

        self.assertIn("editor_asset_manager_at_use_point", completion)
        self.assertIn("refresh_from_runtime_project", completion)
        self.assertLess(
            completion.index("refresh_from_runtime_project"),
            completion.index("resolve_resource_manager"),
        )
        self.assertNotIn("emit_model_import_receipt_log", completion)

    def test_import_owner_projects_terminal_results_to_the_import_log_channel(self) -> None:
        workspace = (
            REPO_ROOT / "zircon_editor/src/ui/retained_host/app/assets/workspace.rs"
        ).read_text(encoding="utf-8")
        diagnostics = (
            REPO_ROOT / "zircon_editor/src/core/asset/import_flow/diagnostics.rs"
        ).read_text(encoding="utf-8")
        module = (
            REPO_ROOT / "zircon_editor/src/ui/host/module.rs"
        ).read_text(encoding="utf-8")

        self.assertNotIn("emit_model_import_receipt_log", workspace)
        self.assertNotIn("LogSource::import()", workspace)
        self.assertNotIn("LogEntry::new", workspace)
        self.assertIn("EditorAssetImportDiagnostics", diagnostics)
        self.assertIn("LogSource::import()", diagnostics)
        self.assertIn("receipt.generation_sequence()", diagnostics)
        self.assertIn("receipt.committed_records().len()", diagnostics)
        self.assertIn("LogJump::asset", diagnostics)
        self.assertIn("project_asset_result", diagnostics)
        self.assertIn("editor_manager.context().logs_handle()", module)

    def test_model_job_does_not_cancel_a_durable_runtime_receipt_after_commit(self) -> None:
        job = (
            REPO_ROOT / "zircon_editor/src/core/asset/import_flow/job.rs"
        ).read_text(encoding="utf-8")
        model_job = job.split("impl EditorJob for AssetImportModelJob", 1)[1].split(
            "impl AssetImportJob", 1
        )[0]

        self.assertLess(
            model_job.index("context.check_cancelled()?;"),
            model_job.index("self.manager.import_model_source"),
        )
        after_commit = model_job.split("self.manager.import_model_source", 1)[1]
        self.assertNotIn("context.check_cancelled()?;", after_commit)

    def test_project_close_defers_release_until_model_import_reaches_terminal_state(self) -> None:
        workspace = (
            REPO_ROOT / "zircon_editor/src/ui/retained_host/app/assets/workspace.rs"
        ).read_text(encoding="utf-8")
        project_close = (
            REPO_ROOT / "zircon_editor/src/ui/retained_host/app/project_close.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("fn cancel_pending_model_import", workspace)
        self.assertIn(".cancel(pending.ticket.id())", workspace)
        self.assertIn("pending.close_requested = true", workspace)
        self.assertIn("if pending.close_requested", workspace)
        self.assertIn("self.commit_project_close()", workspace)
        self.assertIn("PendingModelImport", project_close)
        self.assertLess(
            project_close.index("if !self.cancel_pending_model_import()"),
            project_close.index("self.editor_manager.commit_project_close()"),
        )

    def test_editor_model_import_uses_one_diagnostic_ticket_and_no_legacy_runtime_access(self) -> None:
        ticket = (
            REPO_ROOT / "zircon_editor/src/core/asset/import_flow/model_ticket.rs"
        ).read_text(encoding="utf-8")
        manager = (
            REPO_ROOT
            / "zircon_editor/src/ui/host/editor_asset_manager/manager/default_editor_asset_manager/mod.rs"
        ).read_text(encoding="utf-8")
        job = (
            REPO_ROOT / "zircon_editor/src/core/asset/import_flow/job.rs"
        ).read_text(encoding="utf-8")
        submit = (
            REPO_ROOT / "zircon_editor/src/core/asset/import_flow/model_submit.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("pub struct EditorModelImportTicket", ticket)
        self.assertIn("ticket.try_take()", ticket)
        self.assertNotIn("project_model_result", ticket)
        self.assertIn("impl Drop for AssetImportModelJob", job)
        self.assertIn("diagnostics.project_result", job)
        self.assertIn("diagnostics.arm()", submit)
        self.assertIn(
            "project_asset_manager: Option<EditorProjectAssetRuntimeAccess>", manager
        )
        self.assertNotIn("ProjectAssetManagerAccess", manager)


if __name__ == "__main__":
    unittest.main()
