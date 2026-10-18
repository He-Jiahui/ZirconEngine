"""Static contract for Hub signed catalog and artifact authority.

The catalog is a local-service marketplace boundary: a signed publisher
release must be admitted before it can create a catalog head or expose bytes,
and an organization entitlement must remain the authority for downloads.
"""

from __future__ import annotations

import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
HUB = ROOT / "zircon_hub"
CATALOG = HUB / "src/service/catalog/mod.rs"
ARTIFACTS = HUB / "src/service/catalog/artifacts.rs"
CATALOG_ROUTES = HUB / "src/service/http/catalog_routes.rs"
ARTIFACT_ROUTES = HUB / "src/service/http/catalog_routes/artifacts.rs"
HTTP = HUB / "src/service/http/mod.rs"
CATALOG_TESTS = HUB / "src/service/catalog/tests/cases.rs"
ARTIFACT_TESTS = HUB / "src/service/catalog/artifacts/tests/cases.rs"


def production(source: str) -> str:
    return source.split("#[cfg(test)]", 1)[0]


class HubCatalogArtifactAuthorityContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.catalog = production(CATALOG.read_text(encoding="utf-8"))
        cls.artifacts = production(ARTIFACTS.read_text(encoding="utf-8"))
        cls.catalog_routes = production(CATALOG_ROUTES.read_text(encoding="utf-8"))
        cls.artifact_routes = production(ARTIFACT_ROUTES.read_text(encoding="utf-8"))
        cls.http = production(HTTP.read_text(encoding="utf-8"))
        cls.catalog_tests = CATALOG_TESTS.read_text(encoding="utf-8")
        cls.artifact_tests = ARTIFACT_TESTS.read_text(encoding="utf-8")

    def test_policy_verifies_signed_publisher_release_before_catalog_admission(self) -> None:
        for anchor in (
            "pub struct CatalogPolicy",
            "JwkSet",
            "Algorithm::RS256",
            "validation.set_issuer",
            "validation.set_audience",
            "validation.leeway = 0",
            'set_required_spec_claims(&["iss", "aud", "exp", "sub"])',
            "receipt::validate_id(&release.package_id)?",
            "release.artifact_digest.len() != 64",
            "release.artifact_size > 1073741824",
            'matches!(release.kind.as_str(), "plugin" | "asset")',
            "self.publishers[&kid].subject != release.sub",
        ):
            self.assertIn(anchor, self.catalog, anchor)
        self.assertIn("keys.keys.len() > 32", self.catalog)
        self.assertIn("publishers.len() > 32", self.catalog)
        self.assertIn("envelope.len() > 32768", self.catalog)

    def test_publish_license_and_listing_use_transactional_receipts_and_tenant_fences(self) -> None:
        for anchor in (
            "pub fn publish(",
            "TransactionBehavior::Immediate",
            "receipt::replay(&transaction, principal, &request.operation_id, &fingerprint)?",
            "revision >= release.revision",
            "INSERT INTO catalog_releases",
            "INSERT INTO catalog_heads",
            "catalog_audit",
            "receipt::commit(",
            "transaction.commit()?;",
            "pub fn entitlements(",
            "memberships WHERE organization_id=?1",
            "LIMIT 51",
            "pub fn authorized_artifact(",
            "e.organization_id=?1",
            "m.active=1",
            "pub fn accept_license(",
            "expected_policy_revision",
            "license.accept",
        ):
            self.assertIn(anchor, self.catalog, anchor)
        self.assertIn("current.to_string() != request.expected_policy_revision", self.catalog)
        self.assertIn("release.license_id != request.license_id", self.catalog)
        self.assertIn("ON CONFLICT(organization_id,package_id,revision) DO NOTHING", self.catalog)

    def test_artifact_bytes_are_bounded_digest_checked_and_entitlement_authorized(self) -> None:
        self.assertIn("pub const MAX_ARTIFACT_BYTES: usize = 16 * 1024 * 1024;", self.artifacts)
        self.assertIn("const MAX_CATALOG_BYTES: u64 = 256 * 1024 * 1024;", self.artifacts)
        for anchor in (
            "fn publisher_release(",
            "policy.verify(&envelope.ok_or(ServiceError::Forbidden)?)",
            "publisher.issuer != principal.issuer",
            "fn validate_capacity(stored: u64, added: u64)",
            "fn matches(bytes: &[u8], release: &Release)",
            "Sha256::digest(bytes)",
            "pub fn upload(",
            "release.artifact_size > MAX_ARTIFACT_BYTES as u64",
            "if !matches(bytes, &release)",
            "INSERT INTO catalog_artifacts(digest,payload)",
            "pub fn download(",
            "super::authorized_artifact(",
            "length(payload)<=?2",
            "if !matches(&bytes, &release)",
            "pub fn envelope(",
        ):
            self.assertIn(anchor, self.artifacts, anchor)
        self.assertIn("existing != bytes", self.artifacts)
        self.assertIn("ServiceError::Storage", self.artifacts)

    def test_catalog_routes_keep_authentication_limits_and_response_integrity(self) -> None:
        for anchor in (
            "pub(super) async fn catalog_entitlements(",
            "pub(super) async fn catalog_artifact(",
            "pub(super) async fn catalog_list(",
            "pub(super) async fn catalog_publish(",
            "pub(super) async fn catalog_license(",
            "CatalogPolicy::load(state.catalog_policy_file.as_deref())?",
            "catalog::authorized_artifact(",
            "catalog::publish(",
            "catalog::accept_license(",
        ):
            self.assertIn(anchor, self.catalog_routes, anchor)
        for anchor in (
            "catalog_requests",
            "MAX_ARTIFACT_BYTES",
            'Some("application/octet-stream")',
            "catalog::artifacts::upload(",
            "catalog::artifacts::download(",
            "catalog::artifacts::envelope(",
            'header::CACHE_CONTROL, "no-store"',
            "header::X_CONTENT_TYPE_OPTIONS",
        ):
            self.assertIn(anchor, self.artifact_routes, anchor)

    def test_router_exposes_publish_license_and_artifact_paths_with_global_guards(self) -> None:
        for route in (
            '"/v1/catalog"',
            '"/v1/catalog/{package}/{revision}/artifact"',
            '"/v1/organizations/{organization}/catalog/{package}/{revision}"',
            '"/v1/organizations/{organization}/catalog/{package}/{revision}/artifact"',
            '"/v1/organizations/{organization}/catalog/{package}/{revision}/manifest"',
            '"/v1/organizations/{organization}/licenses"',
        ):
            self.assertIn(route, self.http, route)
        self.assertIn("get(catalog_list).post(catalog_publish)", self.http)
        self.assertIn("get(catalog_entitlements).post(catalog_license)", self.http)
        self.assertIn("DefaultBodyLimit::max(65536)", self.http)
        self.assertIn("request.headers().contains_key(header::ORIGIN)", self.http)
        self.assertIn("Duration::from_secs(15)", self.http)

    def test_rust_regressions_cover_signature_license_cursor_and_artifact_faults(self) -> None:
        for name in (
            "catalog_requires_live_signature_authority_and_lossless_revision",
            "signed_rollback_and_publisher_impersonation_make_no_catalog_write",
            "free_license_acceptance_is_qualified_idempotent_and_audited",
            "license_acceptance_rejects_noncanonical_revision_alias",
            "listing_has_a_cursor_beyond_one_hundred_and_revocation_hides_releases",
        ):
            self.assertIn(name, self.catalog_tests, name)
        for name in (
            "artifact_upload_requires_publisher_exact_signed_bytes_and_is_idempotent",
            "artifact_download_requires_live_membership_license_and_intact_persisted_bytes",
            "artifact_capacity_is_explicit_and_missing_bytes_are_not_metadata_success",
        ):
            self.assertIn(name, self.artifact_tests, name)


if __name__ == "__main__":
    unittest.main()
