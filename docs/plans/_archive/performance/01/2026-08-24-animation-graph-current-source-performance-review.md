---
title: Animation Graph Current Source Performance Review
date: 2026-08-24
scope:
  - zircon_plugins/animation_graph
status: static_complete_shared_source_preserved_and_dynamic_pending
canonical_owners:
  - docs/plans/optimize/zircon_plugins/13-first-party-animation-source-runtime-editor-dist-catalog-skeleton-clip-pose-graph-state-machine-ik-skinning-product-integration-review.md
  - docs/plans/optimize/zircon_editor/76-editor-animation-graph-state-machine-node-edge-parameter-condition-compiler-runtime-transition-blend-preview-transaction-persistence-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/08c-animation-runtime-review.md
references:
  - dev/UnrealEngine/Engine/Source/Editor/AnimGraph/Private/AnimBlueprintCompiler.cpp
---

# Animation Graph Current Source Performance Review

## 1. Coverage

The current package surface is **6/6 Rust files**, **1,135 physical / 1,050 non-empty lines**, **42,668 bytes**, **13 tests** and **0 ignored tests**. Its package-relative `path + LF + raw bytes` SHA-256 is `2fd439516e3d845b453659c9a0125afbda5a08709d1158db43997e05c44d8856`.

Every Dist and Editor Rust file was indexed and parsed. Capability/extension declarations, plugin registration, node palette, graph/state-machine validation and compile reports, native registration and all tests were reviewed. Runtime graph/state-machine compilers and their revision caches were then traced in `zircon_plugins/animation/runtime` to establish the real execution contract. Four package files already had shared changes at review start; those changes were treated as current source and were not edited or formatted by this review.

| Area | Rust files | Physical lines | Current execution truth |
|---|---:|---:|---|
| Dist | 1 | 102 | Publishes a registration manifest, but exposes no bridge method or command invocation path. |
| Editor | 5 | 1,033 | Registers authoring metadata and returns validation/summary reports; it does not produce or install the Runtime compiled artifact. |

## 2. Structural performance findings

### P0: the advertised authoring product has no executable command path

The descriptor registers `animation_graph.authoring.validate` and `animation_graph.authoring.compile`, but neither operation has a handler, background job, artifact route or preview installation path. A workspace-wide caller search finds the compile identifier only in the descriptor and tests, and the validate identifier only in the descriptor. Native Dist reports the authoring capability while `invoke_command` is absent and no bridge method materializes it.

This is a product-boundary failure before it is a hot-loop problem. The resolved plugin profile must bind each advertised operation to one typed implementation and fail admission when the selected source/native package cannot execute it. A registration manifest cannot count as equivalent native behavior.

### P0: Editor “compile” is a summary while Runtime owns the real compiler

`compile_animation_graph` validates the source and returns only its source ID. `compile_animation_state_machine` returns entry-state and state/transition counts. By contrast, Runtime compiles graph nodes and parameters into dense slots, resolves skeleton masks, validates links and stores a `CompiledAnimationGraph`; it separately compiles state machines into dense state/parameter slots, transition condition expressions, consumed triggers and precompiled blend spaces.

Runtime caches these artifacts by asset revision (the state-machine cache is bounded at 64 entries), so the Editor result is neither the artifact Runtime will execute nor proof that Runtime compilation will succeed. Any future attempt to make the Editor report richer by copying more Runtime logic would increase semantic drift and duplicate compile cost. One shared compiler service must own validation, lowering, diagnostics, artifact identity and installability.

### P0: validation semantics have already diverged

The current shared Editor implementation builds a borrowed index and uses non-recursive Kahn traversal over all nodes. Runtime graph compilation uses recursive DFS from the output-reachable graph. Therefore an unreachable cycle is rejected by Editor but ignored by Runtime, while a sufficiently deep reachable graph retains recursion/stack risk only in Runtime.

Playback-speed validation also differs: Editor rejects values `<= 0.0`, which allows `NaN`; Runtime normalizes non-finite speed to `1.0` and permits finite negative speed. The Editor palette exposes `blend_space_1d` and `blend_space_2d` graph nodes even though the current graph asset/compiler supports only Clip, Blend, Additive, Mask and Output. Runtime supports blend spaces as state kinds, not graph nodes. These are observable product semantics, not independent validation preferences.

The shared compiler must define reachability/pruning, cycles, finite/range policy and supported-node capability once. Editor affordances must be generated from that same capability set.

### P1: compilation has no generation, cancellation or artifact-currentness contract

There is no edit generation, debounce/coalescing policy, cancellable compile job, derived-data key, stale-result rejection, diagnostics generation or preview/runtime artifact receipt. Once a real handler is added, synchronous whole-graph compilation on the UI thread would make edit bursts scale with repeated O(V+E) validation/lowering and asset/skeleton resolution.

The Editor should snapshot an immutable authoring generation, schedule one latest-wins compile job, publish a typed result only if source/skeleton/compiler generations still match, and install the exact compiled artifact into preview. Cold and warm compilation must use the shared non-C derived-data cache; steady preview frames must perform zero graph compilation.

### P1: the new Editor algorithm is appropriate locally, but cannot remain a second authority

For command/save-time work, the current borrowed `AnimationGraphIndex` plus Kahn traversal is O((V+E) log V) because of ordered maps/sets and is non-recursive. With current feature scale this is not a demonstrated frame bottleneck. Registration and palette construction are startup operations, not frame work. Micro-optimizing these paths before product wiring would optimize the wrong layer.

The reusable part is the non-recursive reachability/topology design. It should move behind the shared Runtime/compiler boundary, use stable dense indices after authoring-name resolution, prune according to one explicit policy, and feed both diagnostics and lowering. The Editor copy should then disappear.

### P1: existing Runtime caches do not close authoring or telemetry

Runtime recompiles on resource revision changes and bounds compiled caches, which is a sound runtime baseline. It does not make the Editor's output equivalent, provide DDC persistence, expose cold/warm compile phases, or prove that graph/state-machine preview uses the same generation. There are no benchmarks for large/deep graphs, edit bursts, cache hit/miss, artifact size or source/preview/runtime result parity.

Instrumentation must separate index/reachability, semantic validation, skeleton/mask binding, state-machine condition lowering, blend-space compilation, serialization/cache lookup and preview installation. Record input scale, compiled slots/bytes, cold/warm latency, queue delay, cancellation/stale drops and cache outcomes.

## 3. Reference-engine constraints

Unreal's `AnimBlueprintCompiler.cpp` is the primary structural reference:

- `ProcessAnimationNode` validates and bakes each source node into a Runtime node property/handler, assigns compiled node indices and preserves source-to-generated debug mappings. Compilation produces executable Runtime structure, not a source ID or count summary.
- The compiler first identifies pose roots and prunes nodes unreachable through pose links, then validates/processes the retained graph and compiler extensions. Reachability and cycle semantics are compiler policy rather than a separate Editor-only approximation.
- Pose links are patched using compiled indices after allocation. Runtime evaluation therefore uses dense compiled references rather than repeated authoring-name/map traversal.
- Compilation carries worker-thread update eligibility and validates thread-safety constraints while producing the artifact. Performance capability is part of compile output.
- Compiler phase cycle statistics distinguish graph processing, extension work, constant folding and finalization, enabling scale and regression evidence.

ZirconEngine should adopt those boundaries, not Unreal's object model verbatim: one authoritative compiler, explicit pruning, dense runtime artifacts, stable source diagnostics, extension phases and per-phase evidence.

## 4. Dependency-ordered optimization plan

### M0: close the authoring product

Bind Validate and Compile descriptors to typed handlers or remove them from resolved profiles until available. Native Dist must execute equivalent commands through a versioned bridge or fail capability admission. Publish typed unsupported/error reasons; never treat manifest-only registration as product completion.

Define the supported graph/state-machine node capability from the real compiler. Remove or disable palette entries that cannot be lowered, and make source/native/editor/runtime capability resolution identical.

### M1: establish one shared animation compiler service

Move graph and state-machine validation/lowering behind a Runtime-owned compiler API that accepts source, dependency/skeleton snapshots and compiler options. Return a typed `CompiledAnimationProduct` containing dense graph/state-machine artifacts, dependency revisions, compiler/schema version, source-to-compiled node maps, diagnostics and artifact key.

Make reachability/pruning, cycle policy, playback-speed rules, node support, masks, conditions and blend spaces single-source. Replace Runtime recursive DFS with a bounded non-recursive topology pass. Delete Editor-only compile semantics once all consumers use the shared service.

### M2: add incremental Editor jobs and exact preview installation

Assign authoring generations to graph/state-machine edits. Debounce/coalesce edit bursts, cancel superseded work, compile immutable snapshots on the Runtime job scheduler and reject stale completions. Preview must install the same immutable artifact generation that packaging/Runtime consumes and report source/artifact/preview generations together.

Keep UI-thread work to snapshot publication and result application. Dependency changes such as skeleton, mask, sub-machine or referenced graph revisions must invalidate only affected artifacts through an explicit dependency index.

### M3: persist and bound compiler work

Key DDC artifacts by normalized source, dependency revisions/content, compiler/schema version, target/profile and feature set. Store them under the configured non-C cache root with integrity and size metadata. Keep bounded in-memory caches for interactive work and expose eviction/currentness receipts.

After name resolution, use stable dense slots and contiguous buffers. Establish explicit node/edge/state/condition/depth limits, diagnostic budgets and artifact byte limits so hostile or accidental graphs cannot monopolize a worker or the Editor event loop.

### M4: qualify the current-source product

Create deterministic graph/state-machine fixtures spanning shallow, deep, wide, cyclic, unreachable, mask-heavy, condition-heavy and nested-machine cases. Measure cold/warm compile p50/p95/p99, UI queue delay, cancellations, stale drops, cache hit/miss, allocations/bytes and preview install latency across fixed scales.

Once a managed Windows current-source executable exists, capture Editor edit bursts and preview/runtime evaluation with WPR/ETW. Prove no compile work or unbounded queue growth in stable preview frames and match output/artifact generations. RenderDoc is only required when a visible animation preview must be checked for draw/GPU parity; it is not a compiler CPU profiler.

## 5. Acceptance gates

1. Every advertised Validate/Compile operation resolves to one executable source/native product or fails admission with a typed reason.
2. Editor, preview, packaged Runtime and tests consume the same graph/state-machine compiler and immutable artifact schema.
3. Reachability, unreachable-cycle policy, playback-speed semantics and supported-node capabilities are identical across all consumers.
4. Deep valid graphs use a non-recursive topology path and respect explicit node/edge/depth/diagnostic/artifact budgets.
5. Superseded edit generations are cancelled or discarded; stale artifacts can never replace the current preview/runtime generation.
6. Warm cache hits avoid validation/lowering and cold artifacts are stored only in the configured non-C DDC with integrity/version receipts.
7. Stable preview frames perform zero compilation, zero authoring-name graph traversal and zero artifact rebuild.
8. Managed scale evidence reports cold/warm compile p50/p95/p99, queue delay, cancellations, cache outcomes, allocations/bytes and source/artifact/preview generation parity.
9. Current-source launchability, correctness parity and WPR evidence pass before protected-ledger promotion, milestone commit or WeCom completion notification.

## 6. Validation status

- Static per-Rust-file review: **6/6 complete** for the captured source fingerprint.
- Runtime contract trace: real graph/state-machine compilers and revision caches were read to verify the split product behavior.
- `rustfmt --check --config skip_children=true`: Dist plus Editor `capability.rs`, `extension_ids.rs` and `plugin.rs` pass; shared current changes in Editor `lib.rs` and `tests.rs` have formatting-only diffs and were preserved.
- Source changes by this review: **none**; the active four-file shared change was not modified.
- Product closure: **failed statically** because descriptor commands have no executable handler/artifact route and native Dist has no invocation path.
- Compiler parity: **failed statically** because Editor returns summaries while Runtime owns separate executable compilers with different validation semantics.
- Cargo/test execution: **pending** because the managed Windows validation session is not executable; no raw Cargo lane was substituted.
- Current-source executable, WPR/ETW timing/power and visual preview qualification: **pending**. No current-source executable exists, so WPR and RenderDoc were not run.
- This module is not eligible for protected-ledger acceptance, milestone commit or WeCom completion notification.
