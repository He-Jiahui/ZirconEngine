---
title: Plugin glTF Importer Current Source Protected Plan Routing
date: 2026-08-24
scope:
  - zircon_plugins/gltf_importer
status: routing_pending_owner_absorption_and_dynamic_evidence
source_review:
  - docs/plans/performance/01/2026-08-24-plugin-gltf-importer-current-source-performance-review.md
---

# Plugin glTF Importer Current Source Protected Plan Routing

The 9/9 package Rust-file review and selected Runtime/catalog/reference-engine trace are statically complete. Single-provider authority, translated graph/payload factories, source dependency receipts, linear scene construction, unique geometry ownership, typed skin/animation products, scheduling and dynamic acceptance remain open. This note routes the findings without editing protected ledgers or independently owned plans.

| Owner plan | Required absorption |
|---|---|
| `docs/plans/optimize/zircon_plugins/06-first-party-plugin-source-editor-runtime-dist-catalog-profile-capability-closure-review.md` | Hard-cut Runtime builtin, aggregate diagnostic and split provider to one executable glTF authority; Stable/native claims require real import behavior and source/native artifact parity. |
| `docs/plans/optimize/zircon_plugins/07-first-party-asset-importer-source-dependency-subasset-artifact-determinism-sandbox-product-integration-review.md` | Own parse-once translated graph, sandboxed sidecar reads/hashes, stable typed payload keys, subasset determinism, one-payload ownership and provider cutover. |
| `docs/plans/optimize/zircon_plugins/13-first-party-animation-source-runtime-editor-dist-catalog-skeleton-clip-pose-graph-state-machine-ik-skinning-product-integration-review.md` | Replace glTF animation placeholders and first-node-wins skin binding with typed skeleton/clip/channel products and instance-correct skeletal payload identity. |
| `docs/plans/optimize/zircon_runtime/59-runtime-task-execution-job-scheduler-handle-dependency-cancellation-thread-budget-timer-shutdown-diagnostics-product-integration-review.md` | Schedule bounded parse/decode/payload/cook jobs with priority, byte budgets, progress, cancellation, generation receipts and no heavy editor/frame-thread work. |
| `docs/plans/optimize/zircon_runtime/61-runtime-scene-world-level-registry-lifecycle-project-io-snapshot-clone-serialization-schema-transaction-product-integration-review.md` | Consume compact imported scene descriptors and preserve source/artifact transaction generations without cloning complete node subtrees. |
| `docs/plans/optimize/zircon_runtime/62-runtime-scene-hierarchy-transform-propagation-reparent-activation-mobility-visibility-bounds-render-product-integration-review.md` | Define dense node/child adjacency and iterative hierarchy validation so imported scenes/nodes do not recursively rescan every descendant subtree. |
| `docs/plans/optimize/zircon_runtime/64-runtime-resource-authority-asset-handle-load-request-state-machine-version-lease-cache-dependency-reload-cancellation-product-integration-review.md` | Provide immutable buffer/image/mesh payload leases, dependency generations, last-good swaps and bounded reload/cancellation for glTF products. |
| `docs/plans/optimize/zircon_runtime/85-runtime-asset-import-source-discovery-importer-recipe-subasset-derived-data-artifact-cook-package-incremental-build-worker-determinism-product-integration-current-source-review.md` | Own immutable main-source snapshot, sidecar dependency resolver, translated graph/payload/factory pipeline, recipe workers, receipts and incremental currentness. |
| `docs/plans/optimize/zircon_runtime/86-runtime-asset-type-schema-imported-payload-project-document-validation-dependency-serialization-versioning-product-integration-current-source-review.md` | Define schema-v2 typed mesh/scene/material/texture/skeleton/clip payloads and remove duplicate JSON/text or inline geometry authorities. |
| `docs/plans/optimize/zircon_runtime/87-runtime-asset-reference-identity-locator-guid-subasset-redirector-rename-move-resolution-repair-migration-product-integration-current-source-review.md` | Stabilize typed scene/node/mesh/primitive/material/texture/skin/animation subasset identities across reorder and reimport. |
| `docs/plans/optimize/zircon_runtime/92-runtime-texture-image-cubemap-array-volume-format-sampler-mip-compression-upload-streaming-residency-budget-eviction-virtual-texture-product-integration-current-source-review.md` | Separate shared decoded image content from sampler/view identity, eliminate avoidable RGBA clones and route compression/mips through versioned recipes. |
| `docs/plans/optimize/zircon_runtime/93-runtime-mesh-geometry-section-lod-instancing-skinning-morph-deformation-bounds-collision-streaming-product-integration-current-source-review.md` | Publish geometry once, preserve tangents/colors/morphs, bind skins at instance/factory identity and route VG/SDF/LOD/collision through explicit recipes. |
| `docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md` | Expose sidecar dependency/currentness, coalesced reimport, stable subasset identities and generation-safe imported-product publication in editor workflows. |
| `docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md` | Keep glTF parse/decode/cook off the UI thread and surface bounded progress, cancellation, latest-wins rejection and failure/last-good state. |
| `docs/plans/optimize/zircon_editor/32-model-mesh-skeleton-geometry-import-lod-collision-retarget-preview-authoring-review.md` | Make model import/preview consume the canonical translated graph and mesh/skeleton products; expose explicit recipe choices instead of hidden forced VG cook. |
| `docs/plans/optimize/zircon_editor/35-texture-image-cubemap-render-target-sampler-compression-streaming-preview-authoring-review.md` | Preview imported texture/sampler products from shared immutable image payloads and exact target/profile compression generations. |
| `docs/plans/optimize/zircon_editor/77-editor-animation-sequence-clip-channel-binding-interpolation-compression-event-root-motion-sync-preview-compiler-product-integration-current-source-review.md` | Author and preview the same typed glTF clip/channel/skeleton artifacts used by Runtime, including stable target binding and interpolation truth. |
| `docs/plans/optimize/zircon_tooling/07-performance-benchmark-profile-capture-symbol-crash-evidence-baseline-review.md` | Define fixed glTF corpora and WPR/ETW, memory, RenderDoc and power receipts with p50/p95/p99, source-open, copy-byte, queue and main-thread metrics. |
| `docs/plans/optimize/zircon_tooling/08-shared-derived-data-cache-build-cache-remote-execution-artifact-reuse-review.md` | Cache translated/payload/derived products by source+sidecar content, schema/translator/factory/recipe, settings, target/profile/backend under non-C roots. |
| `docs/plans/optimize/zircon_tooling/25-memory-allocation-domain-budget-oom-pressure-fragmentation-pooling-cache-residency-observability-review.md` | Budget source/parser/decode/conversion/cook scratch and enforce one persistent geometry/image payload with peak RSS and retained/copy-byte telemetry. |
| `docs/plans/optimize/zircon_tooling/32-hot-path-catalog-algorithmic-complexity-data-movement-batching-cache-locality-performance-governance-review.md` | Track double parse/reopen, unconditional VG, three geometry copies, recursive O(N^2)/comparison O(N^3) scene dependencies, image clones and synchronous full-import stacks. |

## Protected ledger disposition

Do not mark `docs/plans/performance/review.md` complete yet. Keep the module pending until owner plans absorb the findings and a managed current-source binary passes single-provider truth, parse-once/source-dependency, linear graph, unique-payload, recipe, scheduling, correctness, WPR/ETW, RenderDoc and power gates. The eventual protected-ledger entry should remain module/folder-only per the root performance-plan convention.

No milestone commit or WeCom completion notice is warranted by static review alone.
