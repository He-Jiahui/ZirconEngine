"""Static contract for Hub retention, bounded GC, and recovery ownership.

The local service's retention path is intentionally guarded without invoking
Cargo: the managed Windows lane is currently blocked by an unmanaged fixture
and a dirty external VM checkout.  These checks keep the durable CAS/SQLite
boundaries visible while the native validation receipt is pending.
"""

from __future__ import annotations

import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
HUB = ROOT / "zircon_hub"
RETENTION = HUB / "src/service/cloud/retention"
POLICY = RETENTION / "policy.rs"
MAINTENANCE = RETENTION / "maintenance.rs"
GC = RETENTION / "gc.rs"
RETENTION_MOD = RETENTION / "mod.rs"
RECOVERY = HUB / "src/service/cloud/storage_recovery.rs"
ROUTES = HUB / "src/service/http/cloud_routes.rs"
HTTP = HUB / "src/service/http/mod.rs"
SCHEMA = HUB / "src/service/storage/schema_v6.sql"
RETENTION_TESTS = HUB / "src/service/cloud/tests/retention.rs"
MAINTENANCE_TESTS = HUB / "src/service/cloud/tests/maintenance.rs"
STORAGE_BINDING_TESTS = HUB / "src/service/cloud/tests/storage_binding.rs"
ROUTE_TESTS = HUB / "src/service/http/cloud_routes/tests/cases.rs"


def production(source: str) -> str:
    """Ignore Rust test modules when asserting production ownership."""

    return source.split("#[cfg(test)]", 1)[0]


class HubRetentionRecoveryContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.policy = production(POLICY.read_text(encoding="utf-8"))
        cls.maintenance = production(MAINTENANCE.read_text(encoding="utf-8"))
        cls.gc = production(GC.read_text(encoding="utf-8"))
        cls.retention_mod = production(RETENTION_MOD.read_text(encoding="utf-8"))
        cls.recovery = production(RECOVERY.read_text(encoding="utf-8"))
        cls.routes = production(ROUTES.read_text(encoding="utf-8"))
        cls.http = production(HTTP.read_text(encoding="utf-8"))
        cls.schema = SCHEMA.read_text(encoding="utf-8")
        cls.retention_tests = RETENTION_TESTS.read_text(encoding="utf-8")
        cls.maintenance_tests = MAINTENANCE_TESTS.read_text(encoding="utf-8")
        cls.storage_binding_tests = STORAGE_BINDING_TESTS.read_text(encoding="utf-8")
        cls.route_tests = ROUTE_TESTS.read_text(encoding="utf-8")

    def test_policy_uses_admin_authority_revision_and_replayable_cas(self) -> None:
        for anchor in (
            "const MAX_TRASH_SECONDS: u64 = 30 * 86_400;",
            "keep_latest: Option<u64>",
            "trash_seconds: u64",
            "legal_hold: bool",
            "receipt::validate_id(&request.operation_id)?",
            "request.expected_revision",
            "TransactionBehavior::Immediate",
            "authorize_admin(&transaction, principal, organization, project)?",
            "receipt::replay(&transaction, principal, &request.operation_id, &fingerprint)?",
            'current.revision == request.expected_revision',
            'organization::audit(',
            'receipt::commit(',
            'transaction.commit()?;',
        ):
            self.assertIn(anchor, self.policy, anchor)
        self.assertIn("keep == 0 || keep > MAX_REVISIONS", self.policy)
        self.assertIn("request.policy.trash_seconds > MAX_TRASH_SECONDS", self.policy)
        self.assertIn('RetentionOutcome::RetentionConflict', self.policy)

    def test_maintenance_has_two_phase_prepare_finish_and_unknown_outcome(self) -> None:
        for anchor in (
            "fn run(",
            "cloud_maintenance_runs",
            "TransactionBehavior::Immediate",
            "authorize_admin(&transaction, principal, organization, project)",
            "receipt::replay(&transaction, principal, &request.operation_id, &fingerprint)?",
            "transaction.commit()?;",
            "// A prepared row survives response loss",
            "fn finish(",
            "gc::collect(&transaction, store, Some((organization, project)))?",
            "receipt::commit(&transaction, principal, operation_id, fingerprint, &report)?",
            "DELETE FROM cloud_maintenance_runs",
        ):
            self.assertIn(anchor, self.maintenance, anchor)
        self.assertIn("ServiceError::Storage | ServiceError::Capacity => ServiceError::OutcomeUnknown", self.maintenance)
        self.assertIn("Some((stored, _)) if stored != fingerprint", self.maintenance)

    def test_gc_batches_recheck_references_and_retain_retryable_tombstones(self) -> None:
        self.assertIn("pub(super) const GC_BATCH: u64 = 64;", self.retention_mod)
        self.assertIn("pub(super) const GC_BATCH_BYTES: u64 = 64 * 1024 * 1024;", self.retention_mod)
        for anchor in (
            "ORDER BY digest LIMIT ?3",
            "cloud_gc_pending",
            "NOT EXISTS(SELECT 1 FROM cloud_blobs WHERE digest=?1)",
            "DELETE FROM cloud_upload_leases",
            "INSERT OR IGNORE INTO cloud_gc_pending",
            "SELECT EXISTS(SELECT 1 FROM cloud_blobs WHERE digest=?1)",
            "OR EXISTS(SELECT 1 FROM snapshot_blobs WHERE digest=?1)",
            "OR EXISTS(SELECT 1 FROM cloud_upload_leases WHERE digest=?1)",
            "if bytes > super::super::MAX_BLOB_BYTES as u64",
            "if collected_bytes + stored_bytes > GC_BATCH_BYTES",
            "store.remove_unreferenced(&digest, bytes)?",
            'DELETE FROM cloud_gc_pending WHERE digest=?1',
            "TransactionBehavior::Immediate",
        ):
            self.assertIn(anchor, self.gc, anchor)
        self.assertIn("collect(&transaction, store, None)?", self.gc)

    def test_routes_and_router_expose_no_store_retention_boundaries(self) -> None:
        for anchor in (
            "pub(super) async fn retention(",
            "pub(super) async fn update_retention(",
            "pub(super) async fn maintenance(",
            "cloud::retention(connection, &principal, &organization, &project)",
            "cloud::update_retention(connection, &principal, &organization, &project, request)",
            "cloud::maintain(",
            'header::CACHE_CONTROL, "no-store"',
        ):
            self.assertIn(anchor, self.routes, anchor)
        self.assertIn('"/v1/organizations/{organization}/projects/{project}/cloud/retention"', self.http)
        self.assertIn("get(cloud_routes::retention).post(cloud_routes::update_retention)", self.http)
        self.assertIn('"/v1/organizations/{organization}/projects/{project}/cloud/maintenance"', self.http)
        self.assertIn("post(cloud_routes::maintenance)", self.http)

    def test_recovery_keeps_database_store_binding_and_unpublished_bytes(self) -> None:
        for anchor in (
            'const STORE_ID_FILE: &str = ".store-id";',
            "acquire_database_owner(connection)?",
            "TransactionBehavior::Immediate",
            'SELECT store_id,key_fingerprint FROM cloud_store_binding WHERE id=1',
            "read_store_id(&self.root)?",
            "self.key_fingerprint.as_slice()",
            "reject_link(&entry.path())?",
            "manifest::digest(&name)",
            "self.read(&name)?",
            "self.read(&digest)?",
            "publish_store_id(&self.root)?",
            "super::retention::gc::recover(connection, self)",
        ):
            self.assertIn(anchor, self.recovery, anchor)
        self.assertIn("populated_backup_recovers_at_new_database_root_and_key_paths", self.storage_binding_tests)
        self.assertIn("copied_roots_have_one_database_owner", self.storage_binding_tests)
        self.assertIn("store_identity_must_be_a_canonical_uuid_in_a_regular_file", self.storage_binding_tests)

    def test_schema_and_regressions_cover_restart_lease_and_failure_recovery(self) -> None:
        for table in (
            "CREATE TABLE cloud_retention_policies",
            "CREATE TABLE cloud_snapshot_trash",
            "CREATE TABLE cloud_upload_leases",
            "CREATE TABLE cloud_gc_pending",
            "CREATE TABLE cloud_maintenance_runs",
            "CREATE TRIGGER maintenance_operation_reserved",
            "PRAGMA user_version = 6;",
        ):
            self.assertIn(table, self.schema, table)
        for name in (
            "policy_has_current_admin_authority_revision_and_terminal_operation_receipts",
            "rolling_policy_sustains_more_than_128_commits_with_bounded_cas",
            "shared_tenant_blob_and_upload_lease_survive_unreferenced_retirement",
            "failed_policy_audit_rolls_back_pruning_and_tombstones",
            "failed_file_delete_preserves_tombstone_and_retry_collects_once",
            "recovery_replays_file_delete_when_database_ack_rolled_back",
            "same_base_writers_and_maintenance_never_collect_winner_or_live_uploads",
            "restart_preserves_an_upload_lease_until_its_snapshot_commit",
        ):
            self.assertIn(f"async fn {name}", self.retention_tests, name)
        for name in (
            "prepared_maintenance_survives_delete_ack_fault_and_restart",
            "concurrent_maintenance_retries_commit_one_terminal_result",
            "demotion_after_prepare_reports_unknown_and_retry_preserves_pruned_count",
            "completion_audit_and_receipt_failures_are_unknown_and_replayable",
            "recovery_waiting_for_writer_does_not_hold_physical_lock",
        ):
            self.assertIn(f"async fn {name}", self.maintenance_tests, name)
        self.assertIn("http_cloud_routes_require_verified_identity_and_return_replayable_conflict", self.route_tests)


if __name__ == "__main__":
    unittest.main()
