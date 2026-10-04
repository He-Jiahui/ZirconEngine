---
title: Runtime855 V2 file source capacity
category: zircon_runtime
report_id: Runtime855-v2-file-source-capacity-2026-09-20
date: 2026-09-20
session_id: root-runtime-editor-async-optimization-20260920
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Runtime855 · V2 file source capacity

## Scope

The V2 file-cache source loader seeds a breadth-first queue with the caller's
root paths, then publishes the loaded source list in discovery order. Both
vectors previously started at zero even though `paths.len()` is a known
root-input capacity bound. This slice reserves that bound for the queue and source output without
changing canonical-path de-duplication, import traversal, error precedence, or
source order.

The shared `file_cache.rs` owner also contains the adjacent Runtime589 source
move changes. This record attributes only the two root-bound reservations and
preserves those existing ownership changes.

## Optimization

- Initialize the BFS queue with `Vec::with_capacity(paths.len())` before
  canonical-path admission.
- Initialize the loaded-source output with the same root-input bound; duplicate
  paths may leave spare capacity and transitive imports can still grow it
  normally, while empty input retains zero capacity.
- Keep the existing queue index walk, `BTreeSet` duplicate suppression,
  resource/asset-ID import resolution, and discovery order unchanged.

## TDD and deterministic evidence

The new Python source/model contract was intentionally run RED before the
reservations and lower module existed (two structural failures), then GREEN at
`4/4`. The lower Rust regression checks both reservations and a representative
root/import/source order. The ignored Release probe emits
`RUNTIME855_V2_FILE_SOURCE_CAPACITY_BENCH_V1` with alternating legacy/optimized
P50/P95/P99 samples. For a 4,096-root no-import baseline, the modeled queue and
source growth events each change from `11` to `0`.

## Local validation

- `tools/tests/test_runtime_v2_file_source_capacity_performance_contract.py`:
  `4/4` after the RED run.
- Combined V2 file-source, selector-path, filter-retain, rule-capacity, and
  pseudo-state source/model batch: `18/18`, zero failures/errors/skips.
- The batched non-tooling performance-contract loader now covers `658` files
  and passes `2420/2420` tests in `33.490s`, with zero load errors, failures,
  errors, or skips.
- Exact-file `rustfmt --edition 2021 --check` passes for `file_cache.rs`, the
  lower owner, and the adjacent V2 style owners.
- The latest widened non-tooling contract loader after this slice passes
  `4042/4042` across `957` files in `125.129s`, with zero failures, errors,
  load errors, or skips.
- The current expanded source-contract loader covers `962` non-tooling files
  under the explicit performance-or-contract filename filter and passes
  `4072/4072` tests in `139.499s`, with zero load errors, failures, errors, or
  skips; the `957`/`4042` receipt remains pre-Editor856 historical context.
- No managed Windows Cargo/Release validation command was started locally.
  Allocator observations and product file-source p50/p95/p99 evidence remain
  pending behind the shared external worktree gate.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/ui/v2/file_cache.rs` | `B3D29685042A522E0C7D30374BD5708D834F895CC146DBC1F885D2A6E981A513` |
| `zircon_runtime/src/ui/v2/file_source_capacity_tests.rs` | `B689AB0238FF3C23B184C2E25B8650359165277C64D64A95506BD4EB90B6CEEE` |
| `tools/tests/test_runtime_v2_file_source_capacity_performance_contract.py` | `81B6CA902D5D785F614C255300ADEFCD36817929B9A029368737D184DF9E44EF` |

## Acceptance boundary

Keep this record `implementation_complete` / `managed_validation_pending` until
the owner-attributed Windows Release lane compiles the current Runtime tree,
executes the lower regression and ignored marker, and supplies allocator plus
product file-source p50/p95/p99 measurements. Tooling production remains
deferred for the later Rust migration.
