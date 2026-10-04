---
title: Hybrid GI Product Ownership Current-Source Algorithm Performance Review
date: 2026-08-24
scope:
  - zircon_plugins/hybrid_gi/editor/src
  - zircon_plugins/hybrid_gi/runtime/src
  - zircon_plugins/hybrid_gi/dist/src
  - zircon_runtime/src/graphics/visibility/context/from_extract_with_history
  - zircon_runtime/src/graphics/runtime/render_framework
status: static_complete_dynamic_pending
canonical_owners:
  - docs/plans/optimize/zircon_plugins/19-first-party-hybrid-gi-source-runtime-editor-dist-catalog-scene-representation-surface-cache-global-sdf-radiance-cache-probe-trace-denoise-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/09f3-hybrid-global-illumination-review.md
  - docs/plans/optimize/zircon_runtime/98-runtime-hybrid-global-illumination-scene-representation-surface-cache-global-sdf-screen-probe-radiance-cache-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/89-runtime-render-graph-builder-compiler-resource-lifetime-pass-culling-transient-aliasing-barrier-queue-scheduling-execution-product-integration-current-source-review.md
references:
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Lumen/LumenSceneData.h
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Lumen/LumenScene.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Lumen/LumenSurfaceCacheFeedback.cpp
  - dev/bevy/crates/bevy_render/src/render_resource/pipeline_cache.rs
  - dev/bevy/crates/bevy_render/src/render_resource/bind_group.rs
---

# Hybrid GI Product Ownership Current-Source Algorithm Performance Review

## 1. Coverage and product truth

At repository revision `79f64878f3b9526517644c055ad3bf5cadfccd0f`, the complete Hybrid GI plugin contains **250 Rust files / 37,945 physical lines / 35,078 non-empty lines / 1,419,343 bytes / 240 tests / 21 ignored tests**. Its ordered `workspace-relative path + NUL + raw bytes + NUL` SHA-256 is `69adb0c4b76810600a3cb466e6a63d18e9bebbfbbecdd9f251cc142146385a63`.

The non-validation set is **210 files / 25,668 physical lines / 23,519 non-empty lines / 949,623 bytes / 101 inline tests / 1 ignored test**, fingerprint `1f52074f574732fe0dfa89d6fd27a275c6caaad9e057017c0768f3db67a1199a`. The validation set is **40 files / 12,277 physical lines / 11,559 non-empty lines / 469,720 bytes / 139 tests / 20 ignored tests**, fingerprint `2a0b32136d27bc24bdeb23254badb43e082ad983d756f75355edcc72d8c2cd4d`.

`plugin.toml` still describes the feature as experimental/partial and ordinary runtime profiles do not require it. The Editor viewport path, however, enables Hybrid GI by default and derives only three fixed budgets from `ZIRCON_EDITOR_HYBRID_GI_PROFILE`. The dist crate is a stateless schema surface and explicitly delegates execution to runtime. The editor registers `plugins://hybrid_gi/editor/authoring.zui`, but no complete authoring product owns assets, compilation, diagnostics and live generations.

The worktree has concurrent Hybrid GI edits in material capture and Global SDF synchronization. They were included in the captured fingerprints and preserved. This review made no source edit.

## 2. Per-module review ledger

Every Rust file below was read in its owning folder rather than inferred from the crate root. Validation files are accounted separately so source inventory is not confused with executable coverage.

| Module/folder | Files | Physical lines | Non-empty lines | Bytes | Tests | Result |
|---|---:|---:|---:|---:|---:|---|
| `dist` | 1 | 98 | 86 | 3,676 | 2 | Stateless descriptor only. |
| `editor` | 5 | 139 | 123 | 4,571 | 1 | Default-on viewport profile; authoring asset unresolved. |
| build/resolve | 4 | 1,329 | 1,228 | 51,929 | 0 | Fixed graph dispatch and label-level queue validation. |
| declarations | 14 | 739 | 623 | 23,544 | 2 | DTO-heavy public surface; no unique product authority. |
| GPU readback | 26 | 1,169 | 1,069 | 45,031 | 1 | Broad full-result readback and CPU decode. |
| GPU resources | 61 | 9,017 | 8,385 | 337,385 | 45 | Persistent pipelines mixed with warm-frame allocation. |
| Hybrid GI root | 17 | 897 | 791 | 31,221 | 0 | Registration and neutral bridge wiring. |
| package root | 5 | 1,390 | 1,284 | 52,717 | 8 | Capability/profile assembly. |
| pending completion | 6 | 166 | 148 | 5,755 | 0 | Front-only completion semantics. |
| prepare frame | 5 | 398 | 363 | 14,830 | 2 | Full DTO projection and generation preparation. |
| render-pass executors | 5 | 1,725 | 1,633 | 66,322 | 7 | Per-execute pipeline/bind-group creation. |
| renderer other | 1 | 14 | 13 | 654 | 0 | Module shell. |
| root output | 13 | 2,209 | 2,050 | 85,615 | 21 | Global mutex spans CPU and GPU preparation. |
| scene representation | 29 | 5,551 | 4,997 | 197,607 | 13 | Full-frame sorting/cloning and heuristic cards/voxels. |
| test support | 2 | 93 | 82 | 3,465 | 1 | Optional adapter acquisition. |
| types | 19 | 863 | 760 | 29,860 | 0 | Fixed-capacity packet formats. |
| dedicated validation | 40 | 12,277 | 11,559 | 469,720 | 139 | All sources wired, but GPU skip/ignored coverage remains. |

## 3. Structural performance findings

### P0: two owners compute and composite Hybrid GI

The plugin builds its own scene representation, tracing products, reconstruction output and history. Independently, `zircon_runtime/src/graphics/visibility/context/from_extract_with_history/construct.rs:133-136` initializes core Hybrid GI probe/update/feedback/request state, while the core post-process shader loops the runtime probe array and trace regions before also blending plugin Hybrid GI/history and baked ambient.

With the current hard limits of 16 probes and 16 trace regions, that shader path has a static upper bound of **33,177,600 probe iterations and 530,841,600 probe-region combinations at 1920x1080**, or **132,710,400 and 2,123,366,400 at 3840x2160**, per frame. These are operation-count bounds, not measured timings. The more serious issue is duplicated semantic ownership: visibility, GI, history and composition cannot be scheduled, invalidated or budgeted once.

### P0: one global mutex serializes representation, scheduling and readback setup

The runtime prepare collector clones the frame extract and three light arrays, then holds one global `Mutex` while it selects/evicts per-instance state, performs CPU scene projection and residency updates, encodes Global SDF work, encodes Hybrid GI work and enqueues readback. This turns nominal render-graph queues into a serialized host critical section. A shared in-flight count combines Hybrid GI and Global SDF readbacks, so one workload can back-pressure the other.

Both completion queues examine only the front item. A slow earlier map therefore blocks later ready generations. The implementation also scans a `BTreeMap` to evict among at most 32 instance states and clears legacy registration maps on enabled frames. These are symptoms of a product boundary that rebuilds neutral projections under a lock instead of publishing immutable generations to independently scheduled owners.

### P0: warm frames allocate resources and pipelines that should be device-generation state

Hybrid GI prepare creates 13 base buffers per frame, including cache, residency, pending, update/consume, trace, descriptor, voxel lookup, completion and diagnostic buffers. Five output buffers are zero-filled through temporary host vectors. Scene resources add per-slot texture/upload/readback buffers.

The scene-depth, trace-schedule and resolve executors recreate bind-group layouts, shader modules, pipeline layouts, pipelines and bind groups on every execute; resolve also recreates its parameter buffer. Radiance-cache update creates six bind groups per update frame. This prevents pipeline warmup, obscures lifetime accounting and adds driver synchronization/CPU allocation to the main render path.

Bevy's `pipeline_cache.rs:213-432,668` centralizes queued pipeline creation and reuse; `bind_group.rs:98` explicitly documents creating a bind group once and reusing it. Zircon's equivalent owner belongs in Runtime89/09a, not in three plugin-local execute functions.

### P0: readback is the state transition mechanism, not bounded feedback

Every Hybrid GI frame requests seven fixed buffer readbacks, plus per-atlas/capture/depth slots and complete trace-tile/indirect-argument products. A future is ready only when every component maps. Results are decoded into neutral vectors, rebuilt into provider `BTreeMap`s, and uploaded again on later frames. Readback-ring capacity blocks new work rather than dropping superseded diagnostics or consuming the latest completed generation.

Unreal's `LumenSurfaceCacheFeedback.cpp:22-79,139-299` sizes bounded feedback from the view, hashes/compacts it on GPU and reads from a ring whose latest ready result can be consumed. It does not require a complete CPU mirror of all GPU-owned GI state each frame.

### P1: current timing fields are labels, not GPU measurements

The collector uses host `Instant` spans around command encoding calls, while relevant passes set `timestamp_writes: None`. Values described as GPU pass duration therefore measure host encoding latency. They cannot establish GPU bottlenecks, overlap or parity. GPU timestamp queries, calibrated submission generations and delayed resolve must precede any timing claim.

## 4. Dependency-ordered optimization plan

### M0: hard-cutover to one GI authority

Choose the plugin-backed services already named by the canonical plans as the only Hybrid GI owner. Remove core probe/trace GI loops and duplicate history/composition after the plugin publishes a typed lighting product. Until then, the capability must report experimental/incomplete rather than silently combining two models. Record deterministic counters before changing algorithms.

### M1: move resource lifetime to the render resource authority

Cache shader modules, layouts and pipelines by device generation and specialization key. Persist/grow buffers, textures, bind groups, staging rings and timestamp query sets. Warm-frame creation counters for unchanged views must be zero. This milestone is owned jointly by Runtime89/09a and Plugins19.

### M2: split CPU generations from GPU submission

Publish immutable scene deltas and budget snapshots outside the render lock. Give scene synchronization, Global SDF, surface cache, trace and readback independent queues/generations. Replace front-only completion with newest-ready generation consumption plus explicit supersession/cancellation. Bound readback to counters, sparse feedback and requested captures.

### M3: make the editor/profile truthful

Do not default-enable an incomplete GI path. Editor authoring must expose real asset/state generations, scalability tiers, invalidation reason, residency/overflow, capture controls and fallback status. Runtime, editor and packaged builds must select the same capability contract.

## 5. Acceptance gates

1. Exactly one service owns scene GI state, trace, reconstruction, history and final lighting composition.
2. Unchanged warm frames create zero layouts, pipelines, shader modules and fixed-capacity buffers.
3. No global mutex spans scene compilation, command encoding and readback enqueue; each queue has an observable bounded backlog.
4. CPU work consumes changed generations; GPU-to-CPU transfer contains bounded feedback/counters rather than complete GI state.
5. GPU timestamps identify each pass and frame latency reports p50/p95/p99 with CPU scheduling, allocations, RSS/VRAM, wakeups and power.
6. Dynamic comparison uses the same source, executable, device, scene, resolution and quality configuration. No Unreal timing or power target is claimed without workload-equivalent measurement.

## 6. Validation status

- Non-validation static review: **210/210 files complete** for the captured fingerprint.
- Validation static review: **40/40 files complete**; see the dedicated validation report.
- Cargo execution is pending because the managed Windows validation session is unavailable. Raw Cargo was not used.
- WPR and WPAExporter are installed, but no WPR/ETW or power trace was started because no launchable current-source product executable exists. RenderDoc CLI is unavailable.
- No bottleneck is declared removed, no performance parity is claimed, and no protected-ledger update, milestone commit or WeCom completion message is warranted.
