---
title: Runtime25 Single-Pass Asset URI Projection
category: zircon_runtime
report_id: Runtime25-single-pass-asset-uri-projection-2026-08-26
date: 2026-08-26
session_id: root-runtime25-single-pass-asset-uri-20260826
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime25 single-pass asset URI projection

## Scope

- Parent gaps: the allocation portion of `FILESYSTEM-P1-023`, `FILESYSTEM-P1-024`, and `FILESYSTEM-P1-032` in the watcher path-to-URI ingress.
- Baseline: `8e56165c4c789416c328898d3d8937d934b52efa`, epoch `443`.
- Owners: the watcher asset URI projection helper, its focused Rust tests, the source/performance contract, and this record.
- This slice removes redundant intermediate allocations only. Portable URI escaping, non-UTF-8 policy, structured watcher mapping outcomes, rename reconciliation, mount identity, and complete Runtime25 qualification remain open.

## Change

- `asset_uri_for_path` still rejects paths outside the admitted asset root and still uses the existing `to_string_lossy` component policy.
- The projection now preallocates one URI string, writes `res://`, and appends path components and separators in a single pass.
- The previous component `Vec`, joined path `String`, and formatted URI `String` are removed.
- Focused Rust tests pin nested path projection and the existing `EscapeAttempt` boundary without modifying the foreign-owned watcher integration test file.

For valid UTF-8 watcher paths, intermediate projection containers fall from three to one, a 66.667% structural reduction. The measured allocator calls fall further because collecting 128 components grows the legacy vector several times.

## TDD and local evidence

- RED: `python -m unittest tools.tests.test_runtime25_asset_uri_projection_performance_contract -v` initially passed 1/5, failed three guards, and errored on the absent focused Rust tests.
- GREEN: the same command passes 5/5 after the single-pass projection and tests are added.
- `rustfmt +1.94.1 --edition 2021 --config skip_children=true` completed for both owned Rust files.
- The standalone benchmark is compiled with `rustc +1.94.1 -O -C target-cpu=native` after importing the x64 MSVC environment; it does not use Cargo or the shared build lane.

The deterministic Rust model measures 31 alternating legacy/optimized sample pairs. Each sample projects a 128-component ASCII relative path 4,096 times, uses nearest-rank percentiles, and counts allocation/reallocation calls with a process-local global allocator.

| Metric | Collect + join + format | Single preallocated buffer | Change |
|---|---:|---:|---:|
| P50 | 26.1197 ms | 19.3280 ms | -26.002% |
| P95 | 31.4565 ms | 22.4202 ms | -28.726% |
| allocations / 4,096 projections | 36,864 | 4,096 | -88.889% |

These timings isolate relative-path string projection before `AssetUri::parse`. They do not claim filesystem, watcher ingress, notify delivery, full path parsing, reload publication, or end-to-end asset latency.

## Async validation

One coordinator batch must run the five Python contracts, two focused Rust tests in the real `zircon_runtime` crate, Rust formatting checks, scoped diff checks, and the same optimized Rust performance model.

Acceptance requires both Rust tests and all five source contracts to pass, projection parity to remain exact, allocation reduction to remain at least 80%, and P50/P95 reductions to remain at least 5%. Integration and automatic WeCom publication remain coordinator-owned after managed validation and independent review succeed. The WeCom message must include managed P50/P95 and allocation reductions and label them as watcher asset-URI-projection-only evidence.
