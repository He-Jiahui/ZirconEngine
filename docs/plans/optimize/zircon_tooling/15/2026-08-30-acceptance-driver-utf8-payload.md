---
title: Tooling15 Acceptance Driver UTF-8 Payload Compatibility
category: zircon_tooling
report_id: Tooling15-acceptance-driver-utf8-payload-2026-08-30
date: 2026-08-30
implementation_status: implementation_complete
validation_status: local_targeted_green_full_acceptance_pending
promotion_status: pushed_not_promoted
---

# Tooling15 Acceptance Driver UTF-8 Payload Compatibility

## Scope

The Windows PowerShell 5.1 acceptance harness now computes fixture SHA-256 values through a
pure .NET helper instead of depending on the optional `Get-FileHash` command. The nested driver
transfers child output through an ASCII Base64 envelope containing strict UTF-8 bytes, so Unicode
project roots and manifest identities are not converted through the host code page. Existing
bounded child wait, process-tree termination, exit-code refresh, and structured error handling are
unchanged.

## Validation

- PowerShell 5/7 AST parse: passed for both changed scripts.
- Minimal child-driver failure-envelope round trip: `UTF8_DRIVER_PAYLOAD_OK`; Unicode path length
  13 characters, Base64 payload length 28 bytes.
- Full Python performance-contract batch: `1630/1630` passed in `14.163s` after the change.
- Full `mvp-acceptance.Tests.ps1` remains pending: the fixture exercises many nested acceptance
  processes and exceeded the available non-blocking validation window in the shared worktree.

## Performance Boundary

This is a test-harness compatibility and determinism fix. It changes no runtime/editor hot path
and therefore claims no product P50/P95 improvement. The currently published Runtime75 evidence
remains: category projection P50/P95 `3.150ms/5.6137ms -> 0.1073ms/0.1287ms`, Toast scan
P50/P95 `620.2us/755.0us -> 86.9us/99.1us`, and allocation counts `2 -> 0`, `3207 -> 3`.
ProductReceipt-bound Cargo and product timing qualification remain coordinator-owned and pending.
