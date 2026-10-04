---
status: in_progress
review_date: 2026-09-05
owner_session: astra-signed-receipt-20260905
parent_plan: docs/plans/astra/features/plugins/02-native-trust-admission.md
---

# Runtime Signed Package Receipts

## Contract Before Implementation

The runtime verifier consumes the existing cargo-zircon ProductReceipt v1 and trust-registry v1 wire contracts. It reproduces fixed-order compact serde JSON receipt identity and attestation payloads, then verifies Ed25519 through ring 0.17.14. Runtime never imports tooling or treats candidate-provided trust labels as authority.

Host-owned policy supplies expected plugin/package identity, target triple and runtime/platform, required module kinds, allowed/required capabilities, evaluation time, maximum receipt age, and per-key validity/revocation policy. The existing registry's disabled issuers remain denied. Rotation uses distinct signer IDs; no implicit key fallback.

Signed resource artifacts authenticate the exact manifest bytes. The host policy maps requested manifest module kinds to exact signed artifact logical names and relative paths: product receipt names are caller-defined and runtime DLLs are runtime_dependencies of an executable build, so they cannot be guessed from the Cargo action or module crate name. Each requested native module is covered exactly once; separate Runtime and Editor receipts may contribute to one opaque VerifiedNativePackageProof. The proof contains private expectations and can only be consumed by the native-artifact authority owner. That owner's existing opened-file DLL digest check remains the final load-time byte admission.

## Ownership

- Candidate-only source: zircon_runtime/src/plugin/native_plugin_loader/package_receipt.rs and its focused child modules/tests.
- Candidate-only plan: this file.
- Dependency proposal: audit_plugin_closure owns zircon_runtime/Cargo.toml and adds ring = "0.17.14" beside its Windows-loader dependency edits; validator owns the shared lockfile update.
- audit_plugin_closure owns native_artifact_trust.rs, loader module wiring and all lifecycle changes.
- No tooling changes or Cargo runs in this implementation session. The isolated E:/Git/astra-signed-receipt-20260905 candidate is delivered as a delayed patch; shared leases stay exclusive.

## Acceptance

1. Exact positive ProductReceipt v1 fixture, trusted key and authenticated manifest/DLL digest produce opaque expectations consumable by authority.
2. Unknown/disabled/revoked/expired/not-yet-valid key, future/expired receipt, tampered receipt ID or signature fail before proof construction.
3. Changed manifest bytes/name/package/capabilities, target triple/runtime/platform mismatch, missing/duplicate/wrong native artifact/module and unsigned manifest/DLL substitution fail.
4. Key rotation accepts a currently valid new key while refusing the revoked old key. Multiple native modules remain distinguishable.
   Explicit host dependency bindings become authenticated dependency DLL filenames/digests; unlisted dependency substitutions and basename conflicts are denied before native staging.
   Proof consumption preserves a deadline equal to the earliest used key expiry, trust-policy freshness deadline and receipt age limit; native authority must reject every later load after that deadline.
5. Source format and scoped diff checks precede one managed runtime compilation/regression batch. Runtime validation and source-independent product fixture interoperability stay pending until that batch provides evidence.

## Status

Plan recorded before source implementation. Independent source review found no P0/P1 blocker. Static fixture was independently signed/verified with Node/OpenSSL; Rust ring fixture tests and existing cargo-zircon producer issue/verify are staged for the managed validation batch. Source formatting and SHA256 manifest checks pass; runtime and producer interoperability validation remain pending.
