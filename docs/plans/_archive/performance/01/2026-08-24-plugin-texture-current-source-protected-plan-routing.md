---
title: Plugin Texture Current Source Protected Plan Routing
date: 2026-08-24
scope:
  - zircon_plugins/texture
status: routing_pending_owner_absorption
source_review:
  - docs/plans/performance/01/2026-08-24-plugin-texture-current-source-review.md
---

# Plugin Texture Current Source Protected Plan Routing

The 12/12 Rust-file static review is complete, but product and dynamic acceptance are not. This note routes the findings without editing protected ledgers or independently owned plans.

| Owner plan | Required absorption |
|---|---|
| `docs/plans/optimize/zircon_plugins/18-first-party-texture-source-importer-runtime-editor-dist-catalog-image-cubemap-array-volume-compression-streaming-product-integration-review.md` | Own the package hard cutover: withdraw false `complete` status, replace the summary-only provider, close source/runtime/editor/dist behavior and resource parity, and qualify install/uninstall. |
| `docs/plans/optimize/zircon_editor/35-texture-image-cubemap-render-target-sampler-compression-streaming-preview-authoring-review.md` | Remain the cross-layer product authority for source/recipe/build/artifact/install and the single transactional Texture toolkit. Reject a second plugin-owned document or preview pipeline. |
| `docs/plans/optimize/zircon_runtime/92-runtime-texture-image-cubemap-array-volume-format-sampler-mip-compression-upload-streaming-residency-budget-eviction-virtual-texture-product-integration-current-source-review.md` | Own typed Texture identities, artifact layout, upload, residency, streaming, eviction, budgets and runtime telemetry. Do not consume `TextureImportSummary` as storage/work truth. |
| `docs/plans/optimize/zircon_runtime/85-runtime-asset-import-source-discovery-importer-recipe-subasset-derived-data-artifact-cook-package-incremental-build-worker-determinism-product-integration-current-source-review.md` | Own deterministic recipe/build keys, bounded jobs, cancellation, duplicate coalescing, generation publication and immutable artifacts. |
| `docs/plans/optimize/zircon_runtime/09d-render-asset-streaming-residency-review.md` and `docs/plans/optimize/zircon_runtime/90-runtime-rhi-wgpu-adapter-device-capability-resource-command-queue-submission-completion-readback-surface-device-loss-product-integration-current-source-review.md` | Own asynchronous I/O/upload handoff, mip-tail startup, device-qualified format support, copy/fence lifetime, residency budgets and main/render-thread bounds. |
| `docs/plans/optimize/zircon_tooling/08-shared-derived-data-cache-build-cache-remote-execution-artifact-reuse-review.md` | Own content-addressed Texture artifact storage, cache integrity, cold/warm metrics, dedupe and local/remote policy. |
| `docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md` and `06-first-party-plugin-source-editor-runtime-dist-catalog-profile-capability-closure-review.md` | Gate capability readiness on concrete providers, resolved resources, selected-profile reachability and source/dist equivalence. |
| `docs/plans/optimize/zircon_plugins/08-first-party-editor-authoring-extension-document-operation-toolkit-runtime-contract-product-integration-review.md` | Require the Texture Editor contribution to decorate the canonical toolkit with real resources and executable operations, or withdraw it atomically. |

Protected `docs/plans/performance/review.md` and `pending.md` remain unchanged. Promotion requires managed Windows Cargo/tests, a launchable current-source executable, WPR/ETW and RenderDoc evidence, workload and energy measurements, and owner-plan absorption. Until then this is reviewed static evidence, not an accepted milestone.
