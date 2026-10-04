---
title: Editor123 template-to-view direct lookup
category: zircon_editor
report_id: Editor826-template-view-binding-lookup-2026-09-19
date: 2026-09-19
session_id: root-runtime-editor-async-optimization-20260919
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor826 · template-to-view direct lookup

## Scope

Both Editor extension contribution owners bound plugin UI templates to views by
first cloning every matching template ID into a temporary `BTreeSet`, then
scanning the view map. The binding decision only needs the template keyed by
the current view ID, so the set construction added an avoidable allocation and
full template traversal to each contribution replacement.

## Implementation

- In `EditorExtensionRegistry` and `ContributionBatch`, probe
  `ui_templates.get(view.id())` while scanning mutable views.
- Bind only when the matching descriptor uses the `plugins://` document scheme;
  preserve explicit view template IDs, missing-template behavior, view order,
  and non-plugin template behavior.
- Add lower parity regressions and the ignored
  `EDITOR826_TEMPLATE_VIEW_BINDING_LOOKUP_BENCH_V1` marker for the managed
  Release lane in both owners.

## TDD and deterministic model

The Python source/model contract was intentionally RED against both temporary
`BTreeSet` paths and GREEN after the direct keyed probes were added. For 4,096
templates and 4,096 views, the legacy shape allocates one temporary template
set and scans all templates before the view pass; the optimized shape performs
only the required per-view map probes and allocates no template-ID set. The
model is allocation/complexity evidence only; it is not allocator, CPU, RSS,
or product p50/p95/p99 evidence.

## Local evidence

- Focused source/model contract:
  `tools/tests/test_editor_template_view_binding_lookup_performance_contract.py`
  (`4/4`).
- Lower parity regressions and ignored Release markers are wired in both
  `EditorExtensionRegistry` and `ContributionBatch` test modules.
- Exact-file Rustfmt and Python compilation pass. The current one-process
  non-tooling Runtime/Editor batch loads `621` modules and passes `2218/2218`
  tests with zero failures, errors, or skips; a later shared batch including
  Runtime824, Editor827, and Editor828 loads `624` modules and passes
  `2227/2227` with zero failures, errors, or skips. Managed Cargo, Windows
  Release, and product Editor percentile evidence remain pending.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/core/editor_extension.rs` | `A6639A8DED53746EC1AC4B318FC24A0B26D1EEB04EE113DFF4D4F9C3A6D62D2F` |
| `zircon_editor/src/core/extension/store/batch.rs` | `866C2D0F40CE08F8867DF4C4F0DD60AC08BB793FE6D8B648524C433AE611737F` |
| `tools/tests/test_editor_template_view_binding_lookup_performance_contract.py` | `A677828EAA7C5DD270BB71B9B85F11BFFFE633B9AF5975616309A51913A6522A` |

## Acceptance boundary

Keep this record `implementation_complete` /
`managed_validation_pending` until the owner-attributed Windows Release batch
compiles the current Runtime/Editor tree, executes both lower regressions and
ignored markers, and supplies Editor allocation plus product p50/p95/p99
evidence. Tooling production remains deferred for the later Rust migration.
