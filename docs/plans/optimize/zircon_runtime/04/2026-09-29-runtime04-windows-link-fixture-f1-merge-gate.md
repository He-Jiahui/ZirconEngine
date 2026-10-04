---
title: Runtime04 Windows link-fixture F1 gate
category: zircon_runtime
date: 2026-09-29
session_id: astra-optimize-20260926-batch-a
status: source_candidate_validation_pending
implementation_status: applied_source_candidate
validation_status: managed_tests_pending
product_status: f1_link_host_gate_open
plan_sources:
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/mvp/02-f1-project-and-assets.md
---

# Runtime04 Windows link-fixture F1 gate

Status: source candidate applied; managed validation and F1 acceptance pending.

## Scope

The migration source-boundary group has eight symlink/reparse tests. Each now keeps strict fixture creation and useful failure diagnostics. On Windows only, those eight tests carry an ignore reason requiring a symlink-capable host and explicit execution for F1. Unix runs them by default. The package_assets.rs privilege fixture and its open failure record are separate and unchanged.

## Required validation

- A normal Windows zircon_runtime --lib run may report these eight tests ignored. That run earns no F1 boundary credit.
- The dedicated managed F1 source-boundary job must filter the full asset::tests::migration::project_commandlet::source_boundary group and pass --ignored --test-threads=1.
- Acceptance requires the harness summary to show all eight named tests executed, 8 passed, 0 failed, 0 ignored. Missing output, an ignored result, or a failed link-creation diagnostic leaves the gate open.
- F1 M2.1/M2.2 product creation, registry/import, and product reopen gates remain independently required.
- No performance improvement is claimed and no performance samples are added by this test-only repair.

The current source preimage is SHA-256 bb2033691865cca123e32a0fdc21a53245c03c2b16b6bc7db331f985d720d04c. The revised candidate is SHA-256 4b2dd562b230c557f160f2f8d54fa1e526a77d9659c5533ff09201afae4fc3d1; exact preimage, unified patch, and static evidence are beside this draft. Current Windows token lacked SeCreateSymbolicLinkPrivilege and the Developer Mode registry value was 0x0; no fixture probe or privilege change was made.

Historical predecessor: the strict non-gated offline candidate remains unchanged at SHA-256 dce567a85d4eb01688a2c9d41966ed5040dbc27ade0007abc1c6537592b70c5b; its patch remains SHA-256 5ba26c3cf89f9af66da2d9d4d336b606feae9da9a7671b5ec314e05c491df4cd. The revised gated candidate was applied; the historical strict candidate remains unapplied.

Shared applied file SHA-256: eb792674e5808fd0e76439d97b0c0749512a7e4bbab28152c3f54ec3d8610ff2. Its LF-normalized content matches reviewed candidate 4b2dd562b230c557f160f2f8d54fa1e526a77d9659c5533ff09201afae4fc3d1. Patch applicability and scratch formatting checks passed; no Cargo result is recorded.
