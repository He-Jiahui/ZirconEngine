---
title: Editor301 cached control-id lower-test wiring repair
category: zircon_editor
report_id: Editor823-cached-control-id-test-wiring-repair-2026-09-19
date: 2026-09-19
related_to:
  - docs/plans/optimize/zircon_editor/301/2026-08-30-cache-alert-control-id.md
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: lower_regression_wired
---

# Editor823 · cached control-id lower-test wiring repair

## Finding

The Editor301 lower regression existed at
`template_chips/identity/cached_control_id_tests.rs`, but
`template_chips/identity.rs` did not declare the child test module. The source
cache optimization was therefore present in production while its behavior
regression and ignored Release benchmark were detached from the normal Rust
test tree.

## Repair

`identity.rs` now declares the existing test-only child module through its
folder-backed path. Production chip classification is unchanged; the existing
result-parity regression and `EDITOR301_CACHED_CONTROL_ID_BENCH_V1` marker are
now reachable by the Editor test harness.

## TDD and local evidence

- RED: the new source contract failed because the path declaration was absent.
- GREEN: the focused contract passes `3/3` after wiring.
- Exact-file Rustfmt passes for the production owner and lower test source.
- The one-process current Runtime/Editor performance-contract loader now
  includes this contract and passes `2118/2118` tests across `592` modules
  with zero failures, errors, or skips.
- The broad current Runtime/Editor performance-contract batch remains the
  batched validation mechanism; no standalone Cargo process was started.

## Source fingerprints

| File | SHA-256 |
|---|---|
| `zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_chips/identity.rs` | `7EC8179B88C112CE5724F27EFAB05C2E30B79D885D6D4F739622C34518098817` |
| `zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_chips/identity/cached_control_id_tests.rs` | `CB4493B5BE6A375166A5EF850FA9AEB426B1CCE2BB1520049F5156BEE9F46E46` |
| `tools/tests/test_editor_cached_control_id_performance_contract.py` | `261712A6263401264E10F32993DAA6C464EF7FE2436E5D3BC16D9220091470C4` |

## Acceptance boundary

Managed Windows Cargo/Release execution, the now-wired lower regression and
ignored marker, allocator observations, and Editor chip-classification
p50/p95/p99 evidence remain pending behind the external dirty-worktree gate.
Tooling production remains deferred for the later Rust migration.
