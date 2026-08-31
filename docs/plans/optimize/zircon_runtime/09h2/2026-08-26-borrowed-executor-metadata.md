---
title: Runtime09H2 Borrowed Executor Metadata
category: zircon_runtime
report_id: Runtime09H2-borrowed-executor-metadata-2026-08-26
date: 2026-08-26
session_id: root-runtime09h2-borrowed-executor-metadata-20260826
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime09H2 borrowed executor metadata

## Scope

- Parent scope: the Runtime09H2 graph execution CPU path, specifically builtin post-process, velocity, HZB, exposure, and clustered-lighting GPU dispatch entrypoints.
- Baseline: `8e56165c4c789416c328898d3d8937d934b52efa`, epoch `443`.
- Owners: `builtin_postprocess_executors.rs`, its focused source and Rust contracts, the standalone allocation/timing model, and this record.
- This slice removes executor metadata string copies only. It does not change graph ordering, resources, attachment operations, GPU commands, effect predicates, shaders, or the remaining Runtime09H2 acceptance gates.

## Change

- The 29 builtin GPU entrypoints no longer clone `context.pass_name` before borrowing the GPU context.
- The six entrypoints that also need an executor id no longer materialize `executor_id.as_str().to_string()`.
- A single `with_borrowed_gpu_metadata` helper temporarily moves the two owned strings out of the execution context, lends their `str` views to the GPU call, and restores both fields after either `Ok` or `Err`.
- The missing-GPU branch preserves the existing `require_gpu` error text using the original executor and pass names.
- A direct Rust contract verifies the missing-GPU error and both restored context fields. The source contract fixes the complete 29-call distribution at six executor-id consumers and 23 pass-name-only consumers.

The helper does not claim panic recovery: GPU recording methods use the existing `Result` error contract, and all normal success and error exits restore the context before returning.

## TDD and local evidence

- RED: `python -m unittest tools.tests.test_runtime09h2_borrowed_executor_metadata_performance_contract -v` initially reported four failing/erroring contracts because the helper, borrowed calls, zero-copy path, and direct Rust restoration test were absent.
- GREEN: the same source contract passes 4/4 after all 29 callsites are migrated.
- Two intermediate false negatives assumed `rustfmt` would keep `mem::replace` and error-format arguments on one line; those guards now normalize whitespace without weakening the required operations or argument order.
- `rustfmt +1.94.1 --edition 2021 --config skip_children=true` and scoped `git diff --check` pass.
- The standalone model is compiled with `rustc 1.94.1 -O`; it does not use Cargo or the shared build lane.

The deterministic Rust model measures 31 alternating legacy/borrowed sample pairs, with 32,768 complete 29-executor metadata projections per sample. Six entrypoints consume both pass name and executor id. Every pair must produce rolling checksum `5782480548167811072`; a separate missing-GPU probe must preserve the exact diagnostic and restore both strings. Four local runs passed the acceptance thresholds; the table records the final run.

| Metric | Clone/to-string metadata | Borrowed moved metadata | Change |
|---|---:|---:|---:|
| P50 | 314.1769 ms | 109.1561 ms | -65.256% |
| P95 | 491.6024 ms | 225.4585 ms | -54.138% |
| allocations / 29-executor chain | 35 | 0 | -100.000% |

The other three runs produced P50 reductions of 55.022%, 51.482%, and 57.694%, P95 reductions of 50.143%, 46.421%, and 52.421%, and the same 100% allocation reduction. These timings isolate CPU metadata handling around GPU recording calls; they do not claim shader, GPU, or complete frame-time improvement.

## Async validation

One coordinator batch must run the four Python source contracts, the direct borrowed-metadata Rust test, all 16 existing postprocess context guards, Rust formatting checks, scoped diff checks, exact model parity/error restoration, and the same performance workload.

Acceptance requires 4/4 source contracts and all 17 owned/relevant Rust tests to pass, exact model checksum and error restoration, zero metadata allocations for the 29-executor chain, and P50/P95 reductions of at least 40% and 35% respectively. The Cargo validation remains required even while the foreign tracked deletion of `zircon_runtime/src/core/framework/render/environment/skybox.rs` prevents workspace compile-time input closure planning. Integration and automatic WeCom publication remain coordinator-owned after managed validation succeeds. The WeCom message must include managed P50/P95 and allocation reductions and label them as CPU executor-metadata evidence for one complete 29-entrypoint chain.
