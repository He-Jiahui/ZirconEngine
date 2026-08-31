---
title: Plugin Rendering Current Source Protected Plan Routing
date: 2026-08-24
scope:
  - zircon_plugins/rendering
status: routing_pending_owner_absorption
source_review:
  - docs/plans/performance/01/2026-08-24-plugin-rendering-current-source-performance-review.md
---

# Plugin Rendering Current Source Protected Plan Routing

The 114/114 Rust-file static review is complete, but product and dynamic acceptance are not. This note routes the structural findings without editing protected ledgers or independently owned plans.

| Owner plan | Required absorption |
|---|---|
| `docs/plans/optimize/zircon_plugins/04-rendering-umbrella-feature-bundles-solari-native-provider-product-integration-review.md` | Own the umbrella hard cut: no stable/complete or default feature may resolve through no-op/descriptor-only behavior; bind all 15 bundles through one executable source/dist provider contract. |
| `docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md` | Define typed render-feature provider contributions, generation/lifecycle and native replay. Reject native packaging that exports only umbrella metadata while dropping child executors/resources. |
| `docs/plans/optimize/zircon_plugins/06-first-party-plugin-source-editor-runtime-dist-catalog-profile-capability-closure-review.md` | Make profile selection, linked/native artifact, feature bundle, executor/resource readiness and Editor capability close from one resolved product graph. |
| `docs/plans/optimize/zircon_plugins/09-first-party-particle-vfx-source-runtime-editor-dist-catalog-simulation-render-product-integration-review.md` | Absorb VFX Graph into the canonical particle product; remove fixed placeholder workloads and no-op simulation/render executors. |
| `docs/plans/optimize/zircon_runtime/89-runtime-render-graph-builder-compiler-resource-lifetime-pass-culling-transient-aliasing-barrier-queue-scheduling-execution-product-integration-current-source-review.md` | Own pass culling, resource aliasing, barriers, queue eligibility, execution receipts and per-feature CPU/GPU/VRAM telemetry; no-op success is forbidden. |
| `docs/plans/optimize/zircon_runtime/90-runtime-rhi-wgpu-adapter-device-capability-resource-command-queue-submission-completion-readback-surface-device-loss-product-integration-current-source-review.md` | Own device-generation pipeline/resource caches, queue/fence lifetime, device loss and proof of async-compute overlap. |
| `docs/plans/optimize/zircon_runtime/91-runtime-material-shader-module-graph-permutation-compiler-reflection-layout-pipeline-pso-cache-prewarm-hot-reload-product-integration-current-source-review.md` | Replace Shader Graph string concatenation with typed IR, topology/type/stage validation, reflection, diagnostics and deterministic shader/PSO artifacts. |
| `docs/plans/optimize/zircon_runtime/95-runtime-direct-lighting-photometry-light-grid-clustered-forward-plus-shadow-atlas-cascade-point-spot-rect-cookie-ies-submission-product-integration-current-source-review.md` | Own the per-light Contact Shadow algorithm, light/scissor/quality inputs, temporal validity and Light Cookie integration; reject directionless AO-like substitution. |
| `docs/plans/optimize/zircon_runtime/96-runtime-environment-sky-atmosphere-cloud-ibl-reflection-probe-capture-convolution-sh-pmrem-cache-residency-submission-product-integration-current-source-review.md` | Replace synchronous six-face Reflection Probe capture with a prioritized, cancellable, time-sliced capture/filter/persist/publish job and remove clone/write-read duplication. |
| `docs/plans/optimize/zircon_runtime/97-runtime-baked-lighting-lightmap-probe-volume-bake-job-artifact-residency-sampling-product-integration-current-source-review.md` | Replace the default Baked Lighting no-op with a real bake/artifact/residency contract or keep the feature partial and out of MVP default assembly. |
| `docs/plans/optimize/zircon_runtime/99d-runtime-particle-vfx-system-emitter-cpu-gpu-simulation-rendering-scalability-determinism-product-integration-current-source-review.md` | Own alive-count-driven simulation/render workloads, GPU/CPU fallback, determinism and frame budgets used by VFX Graph. |
| `docs/plans/optimize/zircon_editor/15-material-shader-graph-instance-vfx-particle-preview-compiler-diagnostics-authoring-review.md` | Own real graph editing/preview/compile diagnostics and job lifecycle against the same Runtime provider/artifact generation. |
| `docs/plans/optimize/zircon_editor/22-render-pipeline-frame-capture-lighting-bake-reflection-probe-post-process-debug-authoring-review.md` | Own capture/bake/post-process settings, progress/cancellation, stale-generation diagnostics, viewport receipts and current-source RenderDoc entry points. |
| `docs/plans/optimize/zircon_tooling/08-shared-derived-data-cache-build-cache-remote-execution-artifact-reuse-review.md` | Isolate all shader/pipeline/capture artifacts to explicit non-C DDC roots, complete keys/integrity, cold/warm evidence and zero source-tree writes; then remove tracked cache artifacts. |

Protected `docs/plans/performance/review.md` and `pending.md` remain unchanged. Promotion requires owner absorption, managed Windows tests, one resolved source/dist product graph, removal of no-op success paths, isolated caches, a current-source executable, matched visual gates, WPR/ETW scheduling and power evidence, and RenderDoc pass/resource/GPU evidence.
