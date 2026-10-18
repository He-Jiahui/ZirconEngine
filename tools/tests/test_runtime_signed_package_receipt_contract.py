"""Static contract for signed native-package receipt admission."""

from __future__ import annotations

import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
LOADER = ROOT / "zircon_runtime/src/plugin/native_plugin_loader"
RECEIPT = LOADER / "package_receipt.rs"
TRUST = LOADER / "package_receipt/trust.rs"
WIRE = LOADER / "package_receipt/wire.rs"
AUTHORITY = LOADER / "native_artifact_trust.rs"
RECEIPT_TESTS = LOADER / "package_receipt/tests/cases.rs"


def production(source: str) -> str:
    source = source.replace("#[cfg(test)]\nmod tests;", "")
    marker = "\n#[cfg(test)]\nmod tests {"
    if marker in source:
        source = source.rsplit(marker, 1)[0]
    return source


class RuntimeSignedPackageReceiptContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.receipt = production(RECEIPT.read_text(encoding="utf-8"))
        cls.trust = production(TRUST.read_text(encoding="utf-8"))
        cls.wire = production(WIRE.read_text(encoding="utf-8"))
        cls.authority = production(AUTHORITY.read_text(encoding="utf-8"))
        cls.tests = RECEIPT_TESTS.read_text(encoding="utf-8")

    def test_receipt_verification_is_bounded_and_fail_closed_before_load(self) -> None:
        for anchor in (
            "pub fn verify_native_package_receipts(",
            "policy.modules.is_empty()",
            "receipt_documents.len() > policy.max_receipt_count",
            "manifest_bytes.len() > policy.max_manifest_bytes",
            "wire::validate_path(&policy.manifest_relative_path)?",
            "receipt.verify_integrity()?",
            "trust.verify(&receipt, policy.now, &policy.package_id)?",
            "created > policy.now",
            "duplicate receipt",
            "target triple",
            "find_artifact(",
            "manifest digest",
            "PluginPackageManifest",
            "plugin/package identity",
            "missing, ambiguous or wrong-target manifest module",
            "capability exceeds host grant",
            "required capability is absent",
            "VerifiedNativePackageProof",
        ):
            self.assertIn(anchor, self.receipt, anchor)

    def test_receipt_policy_authenticates_exact_modules_dependencies_and_deadline(self) -> None:
        for anchor in (
            "seen_kinds.contains(&binding.module_kind)",
            "seen_artifacts.insert((&binding.logical_name, &binding.relative_path))",
            "PluginModuleKind::Runtime | PluginModuleKind::Editor | PluginModuleKind::Native",
            "modules.len() != 1",
            'artifact.kind == "dynamic_library"',
            "dependency_names.insert(file_name.to_ascii_lowercase())",
            "ambiguous dependency DLL basename",
            "dependency_matches.len() != 1",
            "NativePluginArtifactDependency",
            "valid_until = valid_until.min(key_valid_until).min(receipt_deadline)",
        ):
            self.assertIn(anchor, self.receipt, anchor)

    def test_trust_registry_requires_host_policy_and_ed25519_time_package_admission(self) -> None:
        for anchor in (
            "pub fn from_registry_json(",
            "registry.schema_version != 1",
            'registry.trust_registry_kind != "zircon_product_receipt_trust_registry"',
            'issuer.algorithm != "ed25519-v1"',
            "wire::decode_hex::<32>",
            "policy.not_after <= policy.not_before",
            "allowed_package_ids.is_empty()",
            "duplicate trust issuer",
            "host policy names unknown signer",
            "pub(super) fn verify(",
            "now >= self.valid_until",
            "issuer.disabled",
            "issuer.policy.revoked",
            "now < issuer.policy.not_before",
            "now >= issuer.policy.not_after",
            "allowed_package_ids",
            "UnparsedPublicKey::new(&ED25519",
        ):
            self.assertIn(anchor, self.trust, anchor)

    def test_wire_identity_rejects_ambiguous_paths_and_recomputes_receipt_id(self) -> None:
        for anchor in (
            "verify_integrity(&mut self)",
            'self.schema_version != 1 || self.receipt_kind != "zircon_product_receipt"',
            "Windows receipt lacks linker fingerprint",
            "duplicate build feature",
            "validate_artifact(artifact, allowed",
            "ambiguous artifact name/path or invalid artifact kind",
            "self.receipt_id != self.canonical_id()?",
            "pub(super) fn validate_path(value: &str)",
            "value.contains(['\\\\', ':'])",
            "unsafe artifact path",
            "pub(super) fn decode_hex<const N: usize>",
        ):
            self.assertIn(anchor, self.wire, anchor)

    def test_verified_proof_only_flows_into_native_authority_and_rechecks_dll_bytes(self) -> None:
        for anchor in (
            "pub fn from_verified_package(",
            "proof.into_parts()",
            "SignedPackage { signer_id, .. }",
            "authenticated signer",
            "signed_deadline",
            "AuthorityExpired",
            "verify_manifest(candidate, &expectation.manifest_digest)",
            "NativePluginArtifactStaging::prepare(",
            "open_library_for_admission(&staged_path)",
            "digest_file(&library_guard, &staged_path)",
            "DigestMismatch",
            "SignatureVerificationUnavailable",
        ):
            self.assertIn(anchor, self.authority, anchor)
        self.assertIn("/// Authenticated digests only; this does not grant native execution.", self.receipt)

    def test_receipt_regressions_cover_rotation_expiry_dependency_and_wire_fixtures(self) -> None:
        for name in (
            "signed_product_receipt_authenticates_manifest_and_native_library_expectation",
            "signed_receipt_rejects_altered_signature_manifest_and_dll_digest",
            "authenticated_proof_becomes_authority_and_expiry_is_rechecked_before_file_access",
            "signed_receipt_denies_identity_target_module_and_capability_escalation",
            "signed_real_dll_closure_admits_normal_and_rejects_tamper_before_entry",
            "signed_receipt_rotation_and_revocation_use_host_key_policy",
            "signed_receipt_rejects_disabled_expired_future_and_wrong_package_keys",
            "signed_receipt_accepts_wire_normalization_without_changing_identity",
            "signed_receipt_authenticates_multiple_modules_and_dependency_dlls",
            "signed_receipt_proof_deadline_keeps_key_and_receipt_expiry",
            "runtime_ring_verifies_independent_product_receipt_wire_fixture",
        ):
            self.assertIn(name, self.tests, name)


if __name__ == "__main__":
    unittest.main()
