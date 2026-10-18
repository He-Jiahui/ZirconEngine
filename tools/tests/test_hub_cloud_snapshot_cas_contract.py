import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
CLOUD = ROOT / "zircon_hub/src/service/cloud"
MANIFEST = CLOUD / "manifest.rs"
SNAPSHOT = CLOUD / "snapshot.rs"
ROUTES = ROOT / "zircon_hub/src/service/http/cloud_routes.rs"
SNAPSHOT_TESTS = CLOUD / "tests/snapshots.rs"
MANIFEST_TESTS = CLOUD / "tests/manifest.rs"
BLOB_TESTS = CLOUD / "tests/blobs.rs"
ROUTE_TESTS = ROOT / "zircon_hub/src/service/http/cloud_routes/tests/cases.rs"


class HubCloudSnapshotCasContractTests(unittest.TestCase):
    def test_manifest_is_bounded_canonical_and_safe_for_windows_projects(self) -> None:
        source = MANIFEST.read_text(encoding="utf-8")

        for anchor in (
            "pub const MAX_BLOB_BYTES: usize = 16 * 1024 * 1024;",
            "pub const MAX_PROJECT_BYTES: u64 = 512 * 1024 * 1024;",
            "pub const MAX_MANIFEST_BYTES: usize = 8 * 1024 * 1024;",
            'pub const IGNORE_POLICY: &str = "zircon-project-v1";',
            "#[serde(rename_all = \"camelCase\", deny_unknown_fields)]",
            "self.files.len() > 10000",
            "self.ignore_policy != IGNORE_POLICY",
            "BTreeSet::new()",
            "entry.path.to_lowercase()",
            "total > MAX_PROJECT_BYTES",
            'matches!(component, "." | "..")',
            '".env"',
            '"secrets"',
            '"node_modules"',
            "self.files.sort_by",
            "serde_json::to_vec(self)",
            "bytes.len() > MAX_MANIFEST_BYTES",
            "use sha2::{Digest, Sha256};",
        ):
            self.assertIn(anchor, source)

    def test_snapshot_commit_has_typed_receipts_and_explicit_conflict_outcome(self) -> None:
        source = SNAPSHOT.read_text(encoding="utf-8")

        for anchor in (
            "pub struct CommitRequest",
            "pub operation_id: String,",
            "pub base_revision: String,",
            "pub struct SnapshotReceipt",
            "pub enum CommitOutcome",
            "Committed {",
            "Conflict {",
            "current_revision: String,",
            "pub fn authorize_receipt(",
            "authorize(connection, principal, organization, project, false)?",
        ):
            self.assertIn(anchor, source)

        self.assertNotIn("last_writer_wins", source.lower())
        self.assertNotIn("unwrap_or_else(|_|", source)

    def test_commit_rechecks_authority_and_replays_before_cas_mutation(self) -> None:
        source = SNAPSHOT.read_text(encoding="utf-8")

        for anchor in (
            "receipt::validate_id(&request.operation_id)?",
            ".parse::<i64>()",
            "base.to_string() != request.base_revision",
            "request.manifest.canonicalize()?",
            "TransactionBehavior::Immediate",
            "authorize(&transaction, principal, organization, project, true)?",
            "receipt::replay(&transaction, principal, &request.operation_id, &fingerprint)?",
            "if current != base",
            '"cloud.commit"',
            '"conflict"',
            "receipt::commit(",
            "transaction.commit()?",
        ):
            self.assertIn(anchor, source)

        transaction = source.index("let transaction = connection.transaction_with_behavior")
        replay = source.index("receipt::replay", transaction)
        cas_check = source.index("if current != base", replay)
        self.assertLess(transaction, replay)
        self.assertLess(replay, cas_check)

    def test_commit_verifies_blob_bytes_before_publishing_snapshot_head(self) -> None:
        source = SNAPSHOT.read_text(encoding="utf-8")

        for anchor in (
            "let mut verified = std::collections::BTreeSet::new();",
            "SELECT bytes FROM cloud_blobs",
            "size != Some(file.bytes)",
            "verified.insert(&file.digest)",
            "store.read(&file.digest)?.len() as u64 != file.bytes",
            "INSERT INTO project_snapshots",
            "INSERT OR IGNORE INTO snapshot_blobs",
            "INSERT INTO project_heads",
            "super::retention::apply(",
            "SELECT COUNT(*) FROM project_snapshots",
            "revisions > MAX_REVISIONS",
            "organization::audit(",
        ):
            self.assertIn(anchor, source)

        verify = source.index("let mut verified")
        snapshot_insert = source.index("INSERT INTO project_snapshots")
        head_insert = source.index("INSERT INTO project_heads")
        retention = source.index("super::retention::apply")
        self.assertLess(verify, snapshot_insert)
        self.assertLess(snapshot_insert, head_insert)
        self.assertLess(head_insert, retention)

    def test_cloud_routes_bound_body_concurrency_and_cache_authority(self) -> None:
        source = ROUTES.read_text(encoding="utf-8")

        for anchor in (
            "fn admit(state: &AppState)",
            "try_acquire_owned()",
            "cloud::MAX_MANIFEST_BYTES + 65536",
            "cloud::MAX_BLOB_BYTES",
            "cloud::commit(",
            "StatusCode::CONFLICT",
            "CACHE_CONTROL, \"no-store\"",
            "CONTENT_TYPE, \"application/json\"",
            "CONTENT_TYPE, \"application/octet-stream\"",
            "cloud::head(",
            "cloud::read_blob(",
        ):
            self.assertIn(anchor, source)

        self.assertNotIn("to_bytes(request.into_body(), usize::MAX)", source)
        self.assertNotIn("Cache-Control\", \"public", source)

    def test_snapshot_and_manifest_regressions_cover_conflict_replay_and_rejection(self) -> None:
        snapshot_tests = SNAPSHOT_TESTS.read_text(encoding="utf-8")
        manifest_tests = MANIFEST_TESTS.read_text(encoding="utf-8")
        blob_tests = BLOB_TESTS.read_text(encoding="utf-8")
        route_tests = ROUTE_TESTS.read_text(encoding="utf-8")

        for test_name in (
            "concurrent_same_base_has_one_commit_and_one_durable_conflict",
            "durable_commit_and_conflict_replay_ignore_unrelated_collection_failure",
            "missing_blob_and_failed_audit_leave_no_head_or_reference",
            "retention_limit_preserves_every_existing_revision_and_reference",
        ):
            self.assertIn(f"fn {test_name}", snapshot_tests)
        for test_name in (
            "snapshot_paths_reject_escape_collisions_and_secret_or_generated_content",
            "canonical_manifest_digest_ignores_input_entry_order_and_rejects_links",
        ):
            self.assertIn(f"fn {test_name}", manifest_tests)
        for test_name in (
            "encrypted_blobs_are_idempotent_and_require_current_tenant_snapshot_reference",
            "quotas_and_bad_digest_do_not_publish_or_admit_blob_rows",
            "restart_accounts_orphans_removes_partial_and_rejects_corruption",
        ):
            self.assertIn(f"async fn {test_name}", blob_tests)
        for anchor in (
            "http_cloud_routes_require_verified_identity_and_return_replayable_conflict",
            'assert_eq!(conflict["status"], "conflict")',
            'assert_eq!(response_json(replay).await, conflict)',
            'assert_eq!(operation["result"], conflict)',
        ):
            self.assertIn(anchor, route_tests)


if __name__ == "__main__":
    unittest.main()
