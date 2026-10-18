from __future__ import annotations

import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
HUB = ROOT / "zircon_hub"
CARGO = HUB / "Cargo.toml"
SERVICE_BIN = HUB / "src/bin/zircon_hub_service.rs"
CONFIG = HUB / "src/service/config.rs"
LIFECYCLE = HUB / "src/service/lifecycle.rs"
HTTP = HUB / "src/service/http/mod.rs"
CLOUD_ROUTES = HUB / "src/service/http/cloud_routes.rs"
AUTHORIZATION = HUB / "src/service/cloud/authorization.rs"
SNAPSHOT = HUB / "src/service/cloud/snapshot.rs"
STORE = HUB / "src/service/cloud/store.rs"
MIGRATION = HUB / "src/service/storage/migration.rs"
DEPLOYMENT = HUB / "deploy/local/deployment.json"
REALM = HUB / "deploy/local/zircon-local-realm.json"
DEPLOY_SCRIPT = HUB / "deploy/local/Manage-LocalDeployment.ps1"
LIFECYCLE_TESTS = HUB / "src/service/lifecycle/tests/cases.rs"
CLOUD_ROUTE_TESTS = HUB / "src/service/http/cloud_routes/tests/cases.rs"
CLOUD_SNAPSHOT_TESTS = HUB / "src/service/cloud/tests/snapshots.rs"
STORAGE_BINDING_TESTS = HUB / "src/service/cloud/tests/storage_binding.rs"
ORG_TESTS = HUB / "src/service/organization/tests/cases.rs"


def production(source: str) -> str:
    """Remove Rust test modules so the guard only checks production ownership."""

    return source.split("#[cfg(test)]", 1)[0]


class HubLocalServiceAuthorityContract(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.cargo = CARGO.read_text(encoding="utf-8")
        cls.service_bin = SERVICE_BIN.read_text(encoding="utf-8")
        cls.config = production(CONFIG.read_text(encoding="utf-8"))
        cls.lifecycle = production(LIFECYCLE.read_text(encoding="utf-8"))
        cls.http = production(HTTP.read_text(encoding="utf-8"))
        cls.cloud_routes = production(CLOUD_ROUTES.read_text(encoding="utf-8"))
        cls.authorization = production(AUTHORIZATION.read_text(encoding="utf-8"))
        cls.snapshot = production(SNAPSHOT.read_text(encoding="utf-8"))
        cls.store = production(STORE.read_text(encoding="utf-8"))
        cls.migration = production(MIGRATION.read_text(encoding="utf-8"))
        cls.deployment = json.loads(DEPLOYMENT.read_text(encoding="utf-8"))
        cls.realm = json.loads(REALM.read_text(encoding="utf-8"))
        cls.deploy_script = DEPLOY_SCRIPT.read_text(encoding="utf-8")
        cls.lifecycle_tests = LIFECYCLE_TESTS.read_text(encoding="utf-8")
        cls.cloud_route_tests = CLOUD_ROUTE_TESTS.read_text(encoding="utf-8")
        cls.cloud_snapshot_tests = CLOUD_SNAPSHOT_TESTS.read_text(encoding="utf-8")
        cls.storage_binding_tests = STORAGE_BINDING_TESTS.read_text(encoding="utf-8")
        cls.org_tests = ORG_TESTS.read_text(encoding="utf-8")

    def test_service_binary_and_config_are_explicitly_bounded(self) -> None:
        self.assertIn('name = "zircon_hub_service"', self.cargo)
        self.assertIn('required-features = ["local-service"]', self.cargo)
        self.assertRegex(self.cargo, r"(?m)^local-service\s*=")
        self.assertIn("RUNTIME_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(2)", self.service_bin)
        self.assertIn("runtime.shutdown_timeout(RUNTIME_SHUTDOWN_TIMEOUT)", self.service_bin)
        self.assertIn("!self.bind.ip().is_loopback()", self.config)
        self.assertIn("!self.database.is_absolute()", self.config)
        self.assertIn("!self.introspection_secret_file.is_absolute()", self.config)
        self.assertIn("self.allow_loopback_http && loopback && url.scheme() == \"http\"", self.config)
        self.assertIn('url.scheme() == "https"', self.config)
        self.assertIn("url.username().is_empty()", self.config)
        self.assertIn("url.password().is_some()", self.config)
        self.assertIn("url.fragment().is_some()", self.config)

    def test_http_boundary_applies_origin_body_concurrency_and_timeout_gates(self) -> None:
        self.assertIn("DefaultBodyLimit::max(65536)", self.http)
        self.assertIn("requests: Arc::new(Semaphore::new(32))", self.http)
        self.assertIn("cloud_requests: Arc::new(Semaphore::new(4))", self.http)
        self.assertIn("middleware::from_fn_with_state", self.http)
        self.assertIn("request.headers().contains_key(header::ORIGIN)", self.http)
        self.assertIn("return Err(ServiceError::Forbidden)", self.http)
        self.assertIn("try_acquire()", self.http)
        self.assertIn("Duration::from_secs(15)", self.http)
        self.assertIn("/v1/organizations/{organization}/projects/{project}/cloud/commit", self.http)
        self.assertIn("/v1/organizations/{organization}/projects/{project}/cloud/blobs/{digest}", self.http)
        self.assertIn("to_bytes(request.into_body(), cloud::MAX_BLOB_BYTES)", self.cloud_routes)
        self.assertIn("to_bytes(request.into_body(), cloud::MAX_MANIFEST_BYTES + 65536)", self.cloud_routes)

    def test_cloud_authority_uses_tenant_membership_and_transactional_cas(self) -> None:
        self.assertIn("m.organization_id=?1 AND p.id=?2", self.authorization)
        self.assertIn("m.issuer=?3 AND m.subject=?4 AND m.active=1", self.authorization)
        self.assertIn('matches!(role.as_str(), "owner" | "admin" | "member")', self.authorization)
        self.assertIn('(!write && role == "viewer")', self.authorization)
        self.assertIn("TransactionBehavior::Immediate", self.snapshot)
        self.assertIn("authorize(&transaction, principal, organization, project, true)", self.snapshot)
        self.assertIn("receipt::replay(&transaction", self.snapshot)
        self.assertIn("if current != base", self.snapshot)
        self.assertIn("CommitOutcome::Conflict", self.snapshot)
        self.assertIn("transaction.commit()?", self.snapshot)
        self.assertIn("authorize(connection, principal, organization, project, false)?", self.snapshot)
        self.assertIn("manifest::digest(digest)?", self.store)
        self.assertIn("Sha256::digest(&plaintext)", self.store)
        self.assertIn("fs::rename(&temporary, &target)", self.store)
        self.assertIn("MAX_GLOBAL_BYTES", self.store)
        self.assertIn("MAX_BLOB_BYTES", self.store)

    def test_lifecycle_and_schema_migration_share_failure_safe_boundaries(self) -> None:
        self.assertIn("let deadline = supervisor.deadline()", self.lifecycle)
        self.assertIn("storage.stop_admission()", self.lifecycle)
        self.assertIn("storage.shutdown(deadline)", self.lifecycle)
        self.assertIn("tokio::time::timeout_at(deadline, &mut server)", self.lifecycle)
        self.assertIn("let census = storage.job_census()", self.lifecycle)
        self.assertIn("transaction_with_behavior(TransactionBehavior::Immediate)", self.migration)
        self.assertIn("if !(0..=7).contains(&version)", self.migration)
        for version in range(2, 8):
            self.assertIn(f"if version < {version}", self.migration)
            self.assertIn(f"schema_v{version}.sql", self.migration)
        self.assertIn("transaction.commit()?", self.migration)

    def test_deployment_manifest_keeps_identity_and_state_on_loopback(self) -> None:
        self.assertEqual(self.deployment["schemaVersion"], 1)
        keycloak = self.deployment["keycloak"]
        self.assertEqual(keycloak["version"], "26.7.3")
        self.assertEqual(keycloak["httpHost"], "127.0.0.1")
        self.assertTrue(keycloak["issuer"].startswith("http://127.0.0.1:"))
        service = self.deployment["hubService"]
        self.assertTrue(service["bind"].startswith("127.0.0.1:"))
        self.assertTrue(service["healthUrl"].endswith("/health"))
        self.assertIn("ValidateArtifacts", self.deploy_script)
        self.assertIn("Readiness", self.deploy_script)
        self.assertIn("Backup", self.deploy_script)
        self.assertIn("Restore", self.deploy_script)
        self.assertIn("IntrospectionSecretFile must be outside StateRoot", self.deploy_script)
        self.assertIn("CloudKeyFile must be outside StateRoot", self.deploy_script)
        self.assertEqual(self.realm["realm"], keycloak["realm"])
        self.assertEqual(self.realm["defaultSignatureAlgorithm"], "RS256")
        desktop = next(client for client in self.realm["clients"] if client["clientId"] == self.deployment["clients"]["desktop"])
        self.assertIn("basic", desktop["defaultClientScopes"])
        self.assertIn("pkce.code.challenge.method", desktop["attributes"])
        self.assertNotIn("password", self.deploy_script.lower())

    def test_rust_regressions_cover_tenant_conflict_recovery_and_binding(self) -> None:
        for name in (
            "startup_signal_waits_for_blocking_owner_until_it_finishes",
            "blocked_startup_returns_unknown_at_shutdown_deadline",
            "shutdown_before_storage_registration_closes_later_database_admission",
        ):
            self.assertIn(f"fn {name}", self.lifecycle_tests)
        for name in (
            "http_cloud_routes_require_verified_identity_and_return_replayable_conflict",
            "response_json",
        ):
            self.assertIn(name, self.cloud_route_tests)
        for name in (
            "concurrent_same_base_has_one_commit_and_one_durable_conflict",
            "missing_blob_and_failed_audit_leave_no_head_or_reference",
            "durable_commit_and_conflict_replay_ignore_unrelated_collection_failure",
        ):
            self.assertIn(f"fn {name}", self.cloud_snapshot_tests)
        for name in (
            "binding_rejects_another_root_and_another_key_independently",
            "concurrent_copied_roots_have_one_database_owner",
        ):
            self.assertIn(f"fn {name}", self.storage_binding_tests)
        for name in (
            "tenant_isolation_stale_revision_and_revocation_are_transactional",
            "failed_audit_rolls_back_the_entire_mutation",
            "cancelled_response_can_be_reconciled_and_retried_without_a_second_commit",
        ):
            self.assertIn(f"fn {name}", self.org_tests)


if __name__ == "__main__":
    unittest.main()
