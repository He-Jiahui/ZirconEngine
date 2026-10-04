---
doc_type: completion-list
candidate_id: runtime1061-runtime04-link-fixture-gate
status: source_candidate_validation_pending
implementation_status: applied_source_candidate
validation_status: managed_tests_pending
product_status: f1_link_host_gate_open
plan_sources:
  - docs/plans/optimize/zircon_runtime/04/2026-09-29-runtime04-windows-link-fixture-f1-merge-gate.md
  - docs/plans/mvp/02-f1-project-and-assets.md
implementation_files:
  - zircon_runtime/src/asset/tests/migration/project_commandlet/source_boundary.rs
tests:
  - zircon_runtime/src/asset/tests/migration/project_commandlet/source_boundary.rs
---

# Runtime1061 Runtime04 migration link-fixture F1 gate

This Astra completion list records the applied Runtime04 fixture repair. It does not change the package_assets.rs fixture or its existing failure record.

The eight source-boundary link/reparse cases are ignored by default on Windows with an explicit symlink-capable-host/F1 reason. Unix keeps them enabled by default. Both platforms retain strict helper failure with target, link, fixture type, and OS-error diagnostics. Existing boundary assertions remain unchanged.

F1 acceptance requires a separate managed Windows run of the whole asset::tests::migration::project_commandlet::source_boundary group using --ignored --test-threads=1. All eight named cases must execute and pass (8 passed, 0 failed, 0 ignored). A default Windows suite that reports these cases ignored is not F1 acceptance. This gate is required alongside, and does not replace, the M2.1/M2.2 project creation, registry/import, and reopen gates.

Preimage SHA-256: bb2033691865cca123e32a0fdc21a53245c03c2b16b6bc7db331f985d720d04c
Candidate SHA-256: 4b2dd562b230c557f160f2f8d54fa1e526a77d9659c5533ff09201afae4fc3d1
Patch SHA-256: 950bcf2689437cfa945fee7e772ce959f1985cc8cf44438c2bcc5ac3a5250f69

Scratch rustfmt, source-structure, and patch-applicability checks passed. Cargo and managed tests were not run. The current Windows token did not list SeCreateSymbolicLinkPrivilege, and AppModelUnlock AllowDevelopmentWithoutDevLicense was 0x0, so a link-capable managed host is still required. This applied source record adds no performance result.

Historical predecessor: the strict non-gated offline candidate remains unchanged at SHA-256 dce567a85d4eb01688a2c9d41966ed5040dbc27ade0007abc1c6537592b70c5b; its patch remains SHA-256 5ba26c3cf89f9af66da2d9d4d336b606feae9da9a7671b5ec314e05c491df4cd. The revised gated candidate was applied; the historical strict candidate remains unapplied.

- [x] Apply strict link fixture setup and explicit Windows ignore reasons.
- [x] Preserve existing unsafe-root boundary assertions and separate package_assets fixture behavior.
- [ ] Execute the eight cases on a symlink-capable managed Windows host.
- [ ] Accept F1 M2.1/M2.2 with applicable product evidence.
