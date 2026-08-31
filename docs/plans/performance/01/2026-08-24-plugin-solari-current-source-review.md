---
title: Plugin Solari Current Source Review
date: 2026-08-24
scope:
  - zircon_plugins/solari
status: static_complete_product_and_dynamic_pending
canonical_owners:
  - docs/plans/optimize/zircon_plugins/04-rendering-umbrella-feature-bundles-solari-native-provider-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/98-runtime-hybrid-global-illumination-scene-representation-surface-cache-global-sdf-screen-probe-radiance-cache-product-integration-current-source-review.md
references:
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Lumen/LumenSceneRendering.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Lumen/LumenSurfaceCache.cpp
---

# Plugin Solari Current Source Review

## 1. Coverage

The current Rust surface is **4/4 files**, **398 physical / 357 non-empty lines**, **14,797 bytes**, and **6 test markers**. Its workspace-relative `path + LF + decoded text + LF` SHA-256 is `983be0becd09ec419967792db9306114b954211948e5903afdfc0f6fb6e1665a`. The package directory is clean.

Per-file coverage is complete: Runtime `capability.rs`, `lib.rs` and `plugin.rs`, plus Dist `lib.rs`. The generated manifest, both Cargo manifests, first-party Runtime catalog, App render-profile selection, Runtime Solari capability/status/provider/assembly/frame-report chain, native registration parser/replayer and Plugins04/Runtime98 owner reports were also checked. There is no Solari Editor crate or physical shader/resource asset in this package.

## 2. Primary finding

Solari contains no lighting algorithm, render pass, shader, acceleration-structure builder, scene representation, surface/radiance cache, denoiser, temporal history or GPU resource. The source plugin registers one `PluginSolariRuntimeProvider` whose status is always `Unavailable` with the explicit message that the realtime raytraced-lighting executor is not implemented. The manifest correctly marks both capabilities `experimental/partial`.

The default and Advanced render profiles do not request Solari. App links the plugin only when the explicit `SolariExperimental` feature is selected; Runtime then requires six backend capabilities, an explicit experimental flag and a ready provider. The current source provider fails that last gate, so it cannot execute GPU work. The normal MVP frame path therefore has no Solari algorithmic cost. When explicitly requested, Runtime currently builds a small fixed-size status report per frame; optimizing its six checks or short vectors would be premature while no executor exists.

The actual defect is source/dist product divergence. Source registration inserts the unavailable provider into `RuntimeExtensionRegistry`. The native registration manifest declares `runtime.render.solari_provider`, but native replay returns immediately when `systems` is empty and otherwise replays only systems. Parsed modules/resources/events/extensions are not materialized. Solari dist consequently produces no provider: the same package resolves to `Unavailable` from source and `ProviderMissing` from native dist.

This cannot be repaired inside Solari by adding metadata or a no-op pass. The shared native ABI needs a typed provider contribution and lifecycle bridge, or `NativeDynamic` must be declared unsupported. No production edit is made here because a package-local shim would preserve the false source/dist contract.

## 3. Unreal source constraints

Unreal's Lumen source demonstrates the minimum structure behind a real dynamic-GI capability. `LumenSceneRendering.cpp` maintains scene/card state, derives camera-dependent update demand, limits card/tile captures per frame, stops request processing at the budget, and optionally schedules eligible Nanite capture work on async compute. `LumenSurfaceCache.cpp` owns typed surface-cache atlases and RDG passes that update selected pages rather than advertising GI through a provider status alone.

The transferable constraints are a concrete scene representation, persistent cache/residency state, bounded incremental updates, render-graph resource lifetimes, explicit capability/fallback selection, temporal validity and measurable GPU passes. Zircon should reuse Runtime98's canonical HGI/Solari boundaries and not copy Unreal class topology. A provider may become `Ready` only after it owns or binds those executable responsibilities.

## 4. Dependency-ordered plan

### M0: preserve the MVP fail-closed boundary

Keep Solari absent from default and Advanced profiles. Add product tests proving that default MVP startup and stable frames link/execute zero Solari provider, pass, shader, resource and worker task. An explicitly selected unavailable package must report one stable structured reason without retrying initialization or allocating repeated diagnostic state every frame.

Make source/dist truth identical. Either extend the shared native registration ABI to materialize a typed Solari provider with provider generation, status, diagnostics and lifecycle, or remove `NativeDynamic` support for this package. Parser acceptance without replay support is a fatal admission error, not success.

### M1: bind one canonical GI architecture

Use Runtime98 as the architecture owner for scene representation, surface/global-distance-field data, radiance/screen-probe caches, tracing, denoise/history and composition. Solari contributes an implementation/provider bundle to those stable contracts; it must not create a second set of scene, cache, history or quality identities.

Provider admission includes backend/adapter/device features, shader/pipeline artifact versions, target/profile/quality tier, memory budgets and fallback. `Ready` requires an initialized executable generation and a health receipt. Device loss, shader failure, hot update and unload revoke readiness before resources are retired.

### M2: bounded frame scheduling

Build scene changes incrementally by stable object/generation identity. Coalesce duplicate invalidations and prioritize view-visible work. Bound acceleration-structure updates, surface-cache captures, radiance updates, trace rays, denoise/history work, GPU memory and uploads per frame. Use Render Graph dependencies and async compute only when dependency and overlap evidence permit it; do not move unbounded work to a worker label.

Stable camera/scene frames must perform no scene rebuild, shader compile, pipeline creation, full-cache clear or host readback/reupload loop. Over-budget demand carries forward with age/fairness telemetry and explicit quality degradation.

### M3: source/dist lifecycle and diagnostics

One canonical provider bundle must materialize from linked source, generated export and every supported native artifact. Registration is transactional. Mount publishes a new generation only after preflight and initialization; unmount stops new frame demand, cancels/quiesces jobs, waits for GPU fences and revokes the generation. Reports distinguish not requested, capability missing, provider missing, experimental disabled, unavailable, initializing, ready, degraded and failed.

Cache and shader artifacts live in explicit project/user DDC roots, never the source tree. Their keys include algorithm/shader/compiler/backend/device/profile/schema inputs, and corrupt or incompatible entries fail closed.

### M4: visual and performance qualification

Use matched indoor/outdoor, static/dynamic light, emissive, occlusion, reflection, camera-cut and large-world scenes. Record disabled baseline and enabled deltas at fixed resolution, quality, warmup, hardware, driver and power mode: CPU/GPU p50/p95/p99, pass timings, queue overlap, updated objects/pages/probes/rays, cache hit ratio, allocations, RSS, VRAM, power and energy/frame.

WPR/ETW proves main/render/worker scheduling and stable-frame behavior. RenderDoc verifies current-source pass/resource/barrier/copy/draw/dispatch evidence and final pixels. Unreal comparison is valid only for matched scene, output quality, tracing mode, cache warm state and hardware; source architecture is not a performance measurement.

## 5. Acceptance

1. Default MVP and Advanced profiles execute zero Solari code beyond bounded profile/status lookup, allocate no Solari GPU resource and publish `NotRequested`.
2. Source and every supported dist form materialize the same provider ID, generation, status, diagnostic and lifecycle. Unsupported contribution kinds reject admission; no manifest field is silently ignored.
3. `Ready` implies concrete initialized passes, shader/pipeline artifacts, scene/cache state and execution receipts. An unavailable/no-op provider can never satisfy readiness.
4. Scene/cache work is incremental and budgeted. Stable frames perform no rebuild, compile, full-cache clear or GPU-host-GPU authority loop; backlog has bounded memory and fair progress.
5. Device loss, hot update and unload revoke readiness, cancel/quiesce work and release resources after fences without stale-generation publication.
6. Current-source WPR and RenderDoc runs publish reproducible CPU/GPU/VRAM/power data for disabled and enabled modes, with visual correctness gates preceding comparison.

## 6. Validation status

- Static per-Rust-file review: **4/4 complete**.
- Source provider truth: **explicitly Unavailable**, default profile execution **disabled by construction**.
- Native source/dist provider equivalence: **failed statically** because extensions are parsed but not replayed.
- `rustfmt --check`: **pass** for all 4 Rust files.
- Cargo/test execution: **pending** because the managed Windows validation session is not executable; no raw Cargo lane was substituted.
- Current-source executable, WPR/ETW, RenderDoc, visual, GPU and energy qualification: **pending**.
- No production source was changed; the required fix is owned by the shared native provider ABI and canonical GI architecture.
- This module is not eligible for protected-ledger acceptance, milestone commit or WeCom completion notification.
