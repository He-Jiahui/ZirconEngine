---
title: Runtime24 lazy host export call table
category: zircon_runtime
report_id: Runtime24
date: 2026-08-31
baseline_head: 14c89f9776bed828cc85e05e4b9914b3f8d1e784
baseline_epoch: 575
status: release_validation_submitted
session_id: root-runtime24-lazy-call-table-release-r2-20260831
implementation_files:
  - zircon_runtime/src/script/vm/host/host_export_registry.rs
tests:
  - tools/tests/test_runtime24_lazy_host_export_call_table_performance_contract.py
  - zircon_runtime/src/script/vm/host/host_export_registry.rs
---

# Runtime24 lazy host export call table

## Problem

`HostExportRegistry::register_module` rebuilt the complete immutable `ScriptCallTable` after every module registration. Registering N modules therefore recopied and reindexed all existing function descriptors and callback Arcs N times. Startup work grew with the cumulative function count even when no caller requested a table between registrations.

## Change

Module registration now updates only the authoritative module map and generation. `call_with_capabilities` and `script_call_table` share a generation-aware helper that returns the existing immutable table when current and rebuilds exactly once when the registry generation has advanced. Previously returned tables remain immutable snapshots of their original generation.

The Rust behavior test registers two modules, proves the internal table remains generation zero throughout the batch, then proves the first read publishes generation two with both functions resolvable. The source performance contract prevents table construction from returning to the registration path and requires both read paths to use the shared generation guard.

The release acceptance test exercises the real `HostExportRegistry`, descriptors, callbacks, capability registry, and immutable call table. Its legacy oracle forces a table read after every registration, reproducing the eager rebuild cost without retaining a second production implementation. The optimized path registers the same batch and reads once. Four paired warmups precede 21 alternating-order sample pairs; raw nanosecond arrays, nearest-rank P50/P95, structural copy counts, and a final table checksum are emitted in one machine-readable line.

## Performance evidence

Acceptance thresholds are at least 99% fewer copied call sites and at least 90% P50/P95 registration-batch latency reduction. The real acceptance fixture registers 256 modules with eight functions each. Its deterministic structural count is 263,168 legacy call-site copies versus 2,048 optimized copies, a 99.222% reduction. Release latency remains pending the managed validation receipt.

### Historical standalone model

The earlier standalone Rust model used 1,024 modules with 16 functions each. These measurements motivated the optimization but are retained only as historical evidence; they are not the terminal acceptance result.

| Measurement | Legacy | Optimized | Reduction |
|---|---:|---:|---:|
| Call-site copies/index insertions | 8,396,800 | 16,384 | 99.805% |
| Nine-round P50 | 4,070,417,300 ns | 7,923,300 ns | 99.805% |
| Nine-round P95 | 4,468,352,000 ns | 13,820,300 ns | 99.691% |

Legacy and optimized tables were exactly equal before timing; the final ordered entry checksum was `360448`. Rounds alternated legacy/optimized execution order.

## Validation

- Original red phase: all three source contracts failed against the eager rebuild implementation.
- Release-acceptance red phase: the new actual-benchmark contract failed alone while the original three contracts passed.
- Release-acceptance green phase: all four source contracts passed.
- Rustfmt 1.94.1 and owned-path `git diff --check` passed.
- The historical standalone Rust model compiled with rustc 1.94.1 and passed its copy and latency thresholds.
- Managed release batch: `cargo +1.94.1 test -p zircon_runtime --locked --release --jobs 1 -- runtime24_lazy_host_export_call_table_ --include-ignored --nocapture --test-threads=1` (two tests, behavior plus performance acceptance).
- Validation submission request: `b1b451d7425248ee9bec3de9bbaaf00d`; terminal ticket and measured P50/P95 remain asynchronous.
