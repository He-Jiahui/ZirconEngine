---
title: Editor25 dirty-domain impact summary single buffer
category: zircon_editor
report_id: Editor824-dirty-domain-summary-single-buffer-2026-09-19
date: 2026-09-19
session_id: root-runtime-editor-async-optimization-20260919
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor824 · dirty-domain impact summary single buffer

## Scope

`dirty_domain_impact_summary` projected every active or node-bearing dirty
domain through `format!`, collected those owned strings into a temporary
`Vec`, then joined that vector. The Debug Reflector is a refresh surface, so a
dense `UiEcsDirtyDomainImpact` list paid one temporary owned string per
retained domain plus one temporary vector allocation on every summary.

## Implementation

- Iterate the source impacts once in their existing order.
- Keep the existing `active || node_count > 0` predicate and comma delimiter.
- Append the existing `Debug` domain and count representation directly to one
  returned `String` through `fmt::Write`.
- Add lower byte-parity/filtering and source-shape regressions, plus the
  ignored `EDITOR824_SINGLE_BUFFER_DIRTY_DOMAIN_SUMMARY_BENCH_V1` Release
  marker.

No Runtime authority, ECS dirty-domain classification, source order, Debug
spelling, or empty-summary behavior changes.

## TDD and deterministic model

The focused source/model contract was intentionally RED against the previous
`format! → collect::<Vec<_>>() → join` chain and GREEN after the direct output
buffer path was introduced. For a dense 4,096-impact summary, the old shape
creates 4,096 intermediate strings and one temporary vector; the new shape
creates neither intermediate object. The returned output string remains
necessary. This is allocation-shape evidence only, not allocator, CPU, UI
paint, or product p50/p95/p99 evidence.

## Local evidence

- Focused source/model contract:
  `tools/tests/test_editor_dirty_domain_impact_summary_performance_contract.py`
  (`4/4`).
- Lower Rust regression compares the retired and direct bytes for active,
  node-driven, filtered, and empty inputs. The ignored Release marker carries
  an alternating P95 gate of `optimized <= 80% of retired`.
- Exact-file Rustfmt, Python compilation, and scoped diff checks pass.
- The one-process Runtime/Editor non-tooling performance-contract batch loads
  `594` modules and passes `2126/2126` tests with zero failures, errors, or
  skips in `5.505s`.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/workbench/debug_reflector/schedule_sections.rs` | `16D7C5FC58F5A17075F44E997EEEC016C5C181DF8939F4516FF4A37DCE47DE3E` |
| `tools/tests/test_editor_dirty_domain_impact_summary_performance_contract.py` | `56CEC45A46395242B35A8AD65484C1BE545CED545F7611EEDCC614D6778FD3C6` |

## Acceptance boundary

Keep this record `implementation_complete` / `managed_validation_pending`
until the owner-attributed Windows Release batch compiles the current Editor
tree, runs the lower regression and ignored marker, and supplies allocator
plus Debug Reflector/product p50/p95/p99 evidence. Tooling production remains
deferred for the later Rust migration; this session does not poll the
coordinator.
