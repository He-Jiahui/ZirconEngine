---
title: Editor06 Reused Native Status Diagnostics
category: zircon_editor
report_id: Editor06-reused-native-status-diagnostics-2026-08-26
date: 2026-08-26
session_id: root-editor57-name-line-split-20260825
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor06 Reused Native Status Diagnostics

## Scope

This slice removes a duplicate owned diagnostic projection from every native-plugin status row. It
does not change the parent plan's string-based diagnostic classification contract or claim the
larger typed lifecycle-authority migration.

## Implementation

`native_plugin_status_report_from_load_report` already materializes each package's diagnostics for
the final owned status DTO. That same vector is now borrowed by `native_load_state`; the classifier
no longer calls `NativePluginLoadProjection::diagnostics_for_plugin` and therefore no longer clones
the vector and every diagnostic string a second time.

The final `EditorPluginStatus` still owns its diagnostics and load-state string. This keeps the
published report contract unchanged while removing only redundant intermediate ownership.

## Performance Evidence

| Evidence | Before | After / target | Structural change |
| --- | ---: | ---: | ---: |
| Diagnostic vector materializations per native row | 2 | 1 | 50% reduction |
| Diagnostic string clones for a 256-diagnostic row | 512 | 256 | 50% reduction |
| Windows-native release p95 | dynamic evidence pending | <= 75% of legacy p95 | coordinator gate |

The ignored release benchmark alternates 17 legacy/optimized samples over 512 row projections with
256 diagnostics of at least 128 payload bytes. It prints
`EDITOR06_REUSED_NATIVE_STATUS_DIAGNOSTICS_BENCH_V1` with both p95 timings, sample/iteration counts,
diagnostic count and width, vector materializations, and string-clone counts. Exact elapsed-time
evidence is accepted only from the coordinator terminal receipt.

## Validation

- Functional and structural regressions cover diagnostic priority and the single
  `diagnostics_for_plugin` call in the row projection.
- Exact Rustfmt and scoped diff checks are required before submission.
- One Windows-native release Cargo invocation batches this task with the adjacent single-pass
  classifier task; no per-task Cargo lane is launched.
- Commit integration, terminal benchmark values, record finalization, and the automatic WeCom
  notification remain coordinator-owned and pending.

## Remaining Parent-plan Work

Editor06 still requires the typed diagnostic envelope, unified plugin-management authority,
durable lifecycle transactions, reload generation publication, scalable detail projection, and
the product qualification gates listed by its parent plan.
