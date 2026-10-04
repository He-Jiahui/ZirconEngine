---
title: RG-P1-9 compiled store-lint and attachment-ledger artifacts
finding: RG-P1-9 / RG89-P1-040
plan_id: astra-rg-p1-lint-20260911
owner: zircon_runtime::render_graph
session_id: astra-rg-p1-lint-20260911
status: implemented_pending_validation
---

## Scope and result

`CompiledRenderGraph::new` now materializes the store-lint report and attachment
bandwidth ledger once, after the graph metadata is complete. Public report/ledger
accessors return the immutable compiled artifacts, and frame statistics use the
zero-scan `store_lint_count` accessor. The existing report and ledger semantics
are unchanged. A pre-existing `depth_or_array_layers()` correction in the shared
baseline supplies texture array-layer byte accounting; this slice does not claim
that correction. This closes the
steady-frame `O(P²*A)` diagnostic rescan identified by RG-P1-9 while leaving
backend byte calibration and observer-only detailed materialization for the
parent performance plan.

The regression `render_perf_store_lint_and_bandwidth_reads_use_compiled_artifacts`
locks the artifact fields, no-rescan accessors, and frame-stat caller; the existing
dead-store regression also checks count parity. No RG-A3 resource replay or RG-A4
interval/accounting files were changed.

## Changed paths and fingerprints

The following paths were leased exclusively for this slice. Hashes are SHA-256
after the edit; graph/store files retain unrelated pre-existing changes (the
resource-state-plan wiring and `depth_or_array_layers()` correction).

| Path | Post-edit SHA-256 |
|---|---|
| `zircon_runtime/src/render_graph/store_lint.rs` | `fdd56ed408ee4a1c74e74834c88e36899ca1f2ad1ff8406b0f42feb36d56e5d0` |
| `zircon_runtime/src/render_graph/graph.rs` | `3e65727bd5fbfababd31ddb5f87b2c3110c7d7ab87dd3417bf8a58d169d2531d` |
| `zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/update.rs` | `a084a9f8219af6bbcf174968a7d6eeffc34935f68aa0c07d1c8c676af9d6b1b` |

Registration base was `c37155ba304740b3762b20585f77fb53a6da47fb` (baseline epoch
608). Lease-time content hashes were `graph.rs=8f8e72384ae9ef5418f2d8b31cfda7152e0c90527fd014121e7a75935c8976c4`,
`store_lint.rs=19ccb4dd9b914b682659d45e0b34ed4d1a15f205d9c52d668079e1094ae9051d`,
and `update.rs=c83d8286521311027c046070dc25858c5b175e18a17e4de4465363793f65ef14`.

## Evidence and remaining gate

- `rustfmt --edition 2021 --check` passed for all three Rust paths.
- Scoped `git diff --check` passed.
- Static source guards passed: both artifact fields are built in `CompiledRenderGraph::new`, both accessors clone the cached values, `store_lint_count` is used by `update_stats`, and the old `.store_lint_report().count()` frame call is absent.
- Cargo, native/WGPU, focused tests, and release performance sampling were not run. The managed Runtime/Editor gate remains pending while the external `E:\Git\zr_vm` checkout is dirty. Dynamic acceptance must confirm report/ledger parity, compile-time work placement, and paired steady-frame allocation/latency behavior before this status can become accepted.
