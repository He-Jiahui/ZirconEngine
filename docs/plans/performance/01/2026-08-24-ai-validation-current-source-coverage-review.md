---
title: AI Validation Current-Source Coverage Review
date: 2026-08-24
scope:
  - zircon_plugins/ai/editor/src/tests.rs
  - zircon_plugins/ai/editor/src/overlay/allocation_tests.rs
  - zircon_plugins/ai/editor/src/runtime_mirror/lookup_allocation_tests.rs
  - zircon_plugins/ai/runtime/src/**/*_tests.rs
  - zircon_plugins/ai/runtime/src/tests
status: static_complete_execution_pending
canonical_owners:
  - docs/plans/optimize/zircon_plugins/15-first-party-ai-source-runtime-editor-dist-catalog-behavior-tree-blackboard-perception-eqs-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/08f-ai-behavior-tree-blackboard-perception-runtime-review.md
  - docs/plans/optimize/zircon_editor/20-ai-behavior-tree-blackboard-perception-eqs-debug-authoring-review.md
---

# AI Validation Current-Source Coverage Review

## 1. Coverage

The validation scope is **43/43 Rust files**, **11,993 physical / 11,068 non-empty lines**, **432,736 bytes** and **188 tests**. Ordered fingerprint: `e3c9ed3bc3ca41e448802847cf0ac6014ed0bee10e8ebfb7b90193ae224284ad`.

Together with production, AI is **93/93 Rust files**, **23,987 physical / 22,082 non-empty lines**, **859,744 bytes** and **237 tests**. Composite ordered fingerprint: `ec16df0aaff7627cf0cbbce46b067fb391d631ad80a4f77274e981aa7830bd24`.

| Scope | Files | Tests | Static result |
|---|---:|---:|---|
| Production | 50 | 49 inline | Behavior/blackboard/manager, perception/product, Editor and Dist fully reviewed. |
| Validation | 43 | 188 | Semantics, aborts, registration, perception budgets, mirrors and allocation checks reviewed. |
| Total | 93 | 237 | Every current AI `.rs` file accounted for at the composite fingerprint. |

Across production and validation, **46** release-only benchmark tests are conditionally ignored and **50** `include_str!` uses assert source structure/text. Those counts describe coverage shape; they are not executed evidence.

## 2. Coverage gaps that block performance acceptance

1. No at-scale scenario covers multiple worlds, `100/1k/10k` agents, many registered trees/providers and mixed LOD. The O(agent * all implementation slots) admission path is therefore unguarded.
2. Node-semantic matrices can inject generic `result`/`service_result` values for SetBlackboard, EmitEvent and UpdateBlackboardDistance; they do not prove the named product side effects.
3. Pair-budget tests count admitted pair calls but do not bound World collection/JSON decoding, stimulus aging, receiver/source vector construction, physics time or oldest-query latency.
4. Tests accept sight as visible when the physics provider is unavailable; no profile test requires typed failure for missing occlusion capability.
5. No test proves debug-reader-off zero work, delta-sized publication, byte/backlog limits, stale generation rejection or bounded overlay geometry/upload.
6. Editor tests validate registered descriptors rather than running Import/Open/Validate/Compile/Toggle factories and installing the exact compiled generation into PIE.
7. No EQS implementation or validation suite exists for compiled templates, resumable steps, async tests, cancellation and time budgets.
8. Source-text `include_str!` assertions can remain green while behavior is non-compiling or non-executable; current legacy viewport-tool symbols are an example.
9. Forty-six release-only microbenchmarks are not part of the available execution evidence. Their thresholds, environments and raw samples must be reported when the validator returns.

## 3. Required validation matrix

Add structural counters before wall-clock gates: worlds/agents/programs/reachable slots, node evaluations/restarts/aborts, blackboard slots changed/validated/published, collected/decoded perception records, candidate/pair/trace counts, oldest deferred age, snapshots/deltas/bytes, debug readers, overlay primitives/uploads, job queue/wait/cancel/stale receipts and lock wait/hold.

Run release matrices for stable and changed frames. Report raw samples plus p50/p95/p99 and complexity relationships; avoid one machine-specific elapsed threshold. Include missing-provider, reload/unload, world shutdown, script/navigation/animation integration, debug off/on and source/Dist capability profiles.

## 4. Execution status

- Static validation-file review: **43/43 complete**.
- Static all-Rust-file review: **93/93 complete** for the captured fingerprint.
- Test execution: pending; the managed Windows Cargo validation session is unavailable and raw Cargo was not used.
- Current-source WPR/ETW/RenderDoc/power: pending; no launchable current-source executable exists.
- No performance bottleneck is declared removed and no Unreal power/time parity is claimed from static evidence.
- This record does not promote protected `review.md`/`pending.md`, create a milestone commit or send WeCom completion.
