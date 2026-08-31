---
title: Plugins11 Borrowed Expanded-Route Cache
category: zircon_plugins
report_id: Plugins11-cached-expanded-routes-2026-08-26
date: 2026-08-26
session_id: root-runtime-interface03-activate-link-failure-20260831
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Plugins11 borrowed expanded-route cache

## Scope

- Parent scope: Plugins11 mixer graph compilation, specifically recursive expansion of post-effect track sends.
- Baseline: `b41b0c0b9da31eb4d19e3f086d6027f745f11a38`, epoch `446`.
- Owned paths: `graph_compile/routes.rs`, its focused source contract, the shared RPC/routes pressure model and contract, and this record.
- This slice preserves cycle and unknown-track errors, local target gain, duplicate-target accumulation, stable target ordering, direct-send rows, expanded downstream rows, graph diff consumers, and public sound contracts. It does not claim to close the plan's effect/send execution parity, audio callback, device, streaming, or product-provider gates.

## Change

Recursive route expansion now reports only whether a track's immutable cache entry has been populated. A cache hit returns without cloning the cached `Vec<SoundTrackSend>`. After recursively populating a target, the parent borrows that target's cached routes while accumulating gains; the completed route vector then moves directly into the cache.

The borrowed child slice also exposes an exact upper bound for the next aggregation step. The gain map reserves space for the direct send and cached downstream rows before insertion, eliminating repeated hash-table growth for broad shared route graphs. Duplicate targets continue to collapse through the same additive `HashMap` entry logic.

## TDD and local evidence

- Initial RED: `python -m unittest tools.tests.test_plugins11_cached_expanded_routes_performance_contract -v` failed 5/5 because recursion returned cloned vectors, cache insertion cloned again, and no shared-downstream Rust contract existed.
- Capacity RED: after the cache-borrow change, a sixth contract failed 1/6 until the gain map reserved for the direct and downstream rows.
- GREEN: the focused source contract now passes 6/6.
- `rustfmt +1.94.1 --edition 2021 --check --config skip_children=true` passes for `routes.rs`.
- Scoped `git diff --check` passes.
- The standalone model compiles with `rustc 1.94.1 -O`; it does not use Cargo or a shared build target.

The deterministic model measures 31 alternating legacy/cached sample pairs on a graph with 2,048 source tracks sharing one hub with 256 downstream routes. Input graph construction is outside the timed and allocation-counted region. All four runs produced checksum `13349105238628374174` for both algorithms.

| Metric | Legacy cloned expansion | Borrowed cached expansion | Change |
|---|---:|---:|---:|
| P50 | 48.6129 ms | 28.6061 ms | -41.16% |
| P95 | 97.2023 ms | 61.5532 ms | -36.67% |
| allocations / expansion | 22,550 | 4,117 | -81.742794% |

The other three runs produced P50 reductions of 42.49%, 40.77%, and 39.35%; P95 reductions of 18.09%, 65.39%, and 33.04%; and the same allocation counts. These numbers cover CPU mixer graph expanded-route compilation only and do not claim audio callback or playback latency.

## Async validation

### Current-source convergence receipt

- Ownership transfer preview request: `8cbea769baba4c82ae6debf6c4338d46`.
- Ownership transfer apply request: `dfcd3de2cfd9454dbba46d0615255185`.
- Applied fingerprint: `04fce303e447edc840b141df5e769942aff944994377cd143f3b134d0a5fe2a0`.
- Current session: `root-runtime-interface03-activate-link-failure-20260831`.
- Shared static/model ticket: `dd4881f740a74ea1997f4e20faedb233` (queued, 17 Python tests).
- Plugins11 Cargo behavior ticket: `644f86170262498bb03a6fe9853caa37` (queued; exact shared-cache behavior test).
- Shared model: `tools/plugins_rpc_routes_pressure.py`, source manifest `C7976604E82B67CA2BC572A3AF78A8DF73B86DFA54D641C591F1D89B86EEF335`.
- Current source hashes: `routes.rs` `2BC52F097A2B403237090205625A696EDE4EFC92DCCBC25C7EF6FB9AAFB8C33C`; shared model `2C63D72FE8BAFCE0C6885A639A27B1E2E66C1BC2AFBCECAF55FBBC40AF0603D8`.

The current-source model is deterministic structural evidence, not wall-clock timing. For 2,048 source tracks sharing 256 downstream routes, it preserves 2,305 cache inserts and models route-row copies `1,050,880 -> 0`, shared-cache route clones `2,048 -> 0`, and gain-map reserve planning for 2,304 direct-send edges. Historical alternating release evidence remains the performance claim: P50 `48.6129 -> 28.6061 ms` (`-41.16%`), P95 `97.2023 -> 61.5532 ms` (`-36.67%`), allocations `22,550 -> 4,117` (`-81.742794%`), checksum `13349105238628374174`.

The coordinator must run the six focused source contracts, the exact Rust test `kira_bridge::graph_compile::routes::tests::shared_downstream_routes_are_reused_from_cache`, Rust formatting, scoped diff checks, checksum parity, and the enlarged standalone model. Acceptance requires 6/6 source contracts, exactly one filtered Rust test, checksum `13349105238628374174`, P50 reduction of at least 25%, no P95 regression, and allocation reduction of at least 75%.

The ticket joins the outstanding optimization validation batch and the business Session does not wait for it before subsequent work. Cargo remains coordinator-owned. At the time of this record, foreign unmanaged wave85-wave87 build directories can stop managed copies at `artifact_governance`, and the foreign tracked deletion of `zircon_runtime/src/core/framework/render/environment/skybox.rs` can stop Cargo input-closure planning. Neither path is part of this candidate.

Integration and automatic WeCom publication remain coordinator-owned after managed validation and independent review succeed. The WeCom message must include the managed P50/P95 and allocation reductions and label them as CPU mixer graph expanded-route compilation evidence for the 2,048-root/256-downstream workload.
