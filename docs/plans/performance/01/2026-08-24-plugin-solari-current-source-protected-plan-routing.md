---
title: Plugin Solari Current Source Protected Plan Routing
date: 2026-08-24
scope:
  - zircon_plugins/solari
status: routing_pending_owner_absorption
source_review:
  - docs/plans/performance/01/2026-08-24-plugin-solari-current-source-review.md
---

# Plugin Solari Current Source Protected Plan Routing

The 4/4 Rust-file static review is complete, but product and dynamic acceptance are not. This note routes findings without editing protected ledgers or independently owned plans.

| Owner plan | Required absorption |
|---|---|
| `docs/plans/optimize/zircon_plugins/04-rendering-umbrella-feature-bundles-solari-native-provider-product-integration-review.md` | Own the Solari package hard cut: preserve default-off behavior, make source/dist provider materialization identical, and prohibit `Ready` for metadata-only/no-op providers. |
| `docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md` | Add a typed native provider contribution/lifecycle bridge, or fail admission for unsupported extensions. Parsing an extension and replaying zero behavior is not compatibility. |
| `docs/plans/optimize/zircon_plugins/06-first-party-plugin-source-editor-runtime-dist-catalog-profile-capability-closure-review.md` | Require selected profile, linked/native artifact, concrete provider, provider generation and health to close before capability publication. |
| `docs/plans/optimize/zircon_runtime/98-runtime-hybrid-global-illumination-scene-representation-surface-cache-global-sdf-screen-probe-radiance-cache-product-integration-current-source-review.md` | Remain the canonical owner for GI scene representation, caches, tracing, denoise/history, composition, budgets and quality/fallback; reject a parallel Solari data model. |
| `docs/plans/optimize/zircon_runtime/89-runtime-render-graph-builder-compiler-resource-lifetime-pass-culling-transient-aliasing-barrier-queue-scheduling-execution-product-integration-current-source-review.md` | Own pass/resource/barrier lifetime, async-compute eligibility and execution receipts for any future Solari provider. |
| `docs/plans/optimize/zircon_runtime/90-runtime-rhi-wgpu-adapter-device-capability-resource-command-queue-submission-completion-readback-surface-device-loss-product-integration-current-source-review.md` | Own ray-query/acceleration-structure/device qualification, queue/fence lifetime, device loss and GPU telemetry. |
| `docs/plans/optimize/zircon_editor/22-render-pipeline-frame-capture-lighting-bake-reflection-probe-post-process-debug-authoring-review.md` | Own profile/settings/debug/capture UX and show exact unavailable/degraded/provider-generation reasons; do not create a descriptor-only Solari editor shell. |
| `docs/plans/optimize/zircon_tooling/08-shared-derived-data-cache-build-cache-remote-execution-artifact-reuse-review.md` | Own shader/pipeline/cache artifact identity, isolated roots, integrity and cold/warm evidence once an executable provider exists. |

Protected `docs/plans/performance/review.md` and `pending.md` remain unchanged. Promotion requires owner absorption, managed Windows tests, source/dist lifecycle equivalence, a current-source executable, WPR/ETW and RenderDoc evidence, matched visual correctness, GPU/VRAM/power measurements and removal of all no-op readiness paths.
