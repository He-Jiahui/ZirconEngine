---
title: Runtime Prototype Resource Alias Single Buffer
category: zircon_runtime
report_id: Runtime872-prototype-resource-alias-single-buffer-2026-09-21
date: 2026-09-21
session_id: root-runtime-editor-async-optimization-20260921
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Runtime872 Prototype Resource Alias Single Buffer

## Finding

Prototype-file cache alias projection collected every valid UTF-8 relative path
component into a borrowed-reference vector, joined the vector with `/`, and
formatted the joined child string behind `res://`. Dense nested resource paths
therefore allocated temporary slots and one joined string before the required
alias.

## Optimization

- Reuse the relative component iterator directly and return `None` when it has
  no valid UTF-8 component.
- Reserve one alias output from the `res://` prefix plus the relative path byte
  upper bound, then append the first and remaining components positionally.
- Preserve nearest `assets` root selection, relative and absolute paths,
  separator normalization, component order, Unicode, and empty/no-root results.
- Keep cache lookup, freshness, source loading, and prototype parsing unchanged.

## TDD and deterministic evidence

The combined Editor891/Runtime872 source-model batch was observed RED at `2/10`
and GREEN at `10/10`. Lower regressions compare absolute, relative, Unicode,
nested-`assets`, equal-root, and missing-root cases against the retired
collect/join/format path.

Across 4,096 aliases with 64 relative components, the deterministic model
changes temporary borrowed-reference slots from `262144` to `0` and join
outputs from `4096` to `0`. Ignored marker
`RUNTIME872_PROTOTYPE_RESOURCE_ALIAS_SINGLE_BUFFER_BENCH_V1` emits 101
alternating p50/p95/p99 sample pairs and requires the single-buffer p95 to
remain within 10% of collect/join/format.

## Local validation boundary

- Exact-file Rustfmt, Python bytecode compilation, and scoped
  `git diff --check` pass.
- The final Runtime872/Editor891 contracts plus adjacent Workbench/ZUI
  contracts pass `31/31`.
- Runtime872 received no per-task Cargo run and was submitted with Editor891 in
  asynchronous v18 (PID `35604`) at `2026-09-21T22:02:33.0422146+08:00`.
- No v13-v18 receipt was read or monitored after submission.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/ui/template/asset/prototype_file_cache.rs` | `86E0B3F8F3C7F32EAB5437F8C5F76FAE8F4C2D732D1E0D84B8B67839FF17DFB5` |
| `zircon_runtime/src/ui/template/asset/prototype_file_cache/resource_alias_single_buffer_tests.rs` | `59D2AC49547BE357BD198B3F1F9FEC546055A0F85FD87794FFE3FDA289682540` |
| `tools/tests/test_runtime872_prototype_resource_alias_single_buffer_performance_contract.py` | `D30F486B3BD2272C7280D68C34570DC3B889FBD832DE47208B2E2D7C243F8FFC` |

## Acceptance boundary

Keep this record `implementation_complete` / `managed_validation_pending` until
a combined current-source Windows lane compiles Runtime, executes the lower
regression and ignored Release marker, and supplies allocator plus real
prototype-cache resource-resolution p50/p95/p99 evidence. The deterministic
allocation model is not product acceptance.
