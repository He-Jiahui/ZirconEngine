---
title: Plugin OBJ Importer Current Source Protected Plan Routing
date: 2026-08-24
scope:
  - zircon_plugins/obj_importer
status: routing_pending_owner_absorption_and_dynamic_evidence
source_review:
  - docs/plans/performance/01/2026-08-24-plugin-obj-importer-current-source-performance-review.md
---

# Plugin OBJ Importer Current Source Protected Plan Routing

The 4/4 Rust-file static review is complete, but canonical-provider ownership, dependency/material translation, asynchronous payload/cook execution, single-copy artifacts and dynamic acceptance are not. This note routes findings without editing protected ledgers, independently owned plans or the two shared Runtime source files.

| Owner plan | Required absorption |
|---|---|
| `docs/plans/optimize/zircon_plugins/07-first-party-asset-importer-source-dependency-subasset-artifact-determinism-sandbox-product-integration-review.md` | Hard-cut three OBJ authorities to one split translator plus shared factories; consume source snapshots, resolve MTL/textures, publish stable nodes/subassets/dependencies, prove source/native parity and remove duplicate Runtime/diagnostic paths. |
| `docs/plans/optimize/zircon_plugins/06-first-party-plugin-source-editor-runtime-dist-catalog-profile-capability-closure-review.md` | Link `RuntimePluginId::ObjImporter` through the first-party provider catalog and make enabled-but-missing source/native providers fail closed instead of silently disappearing or advertising Stable metadata. |
| `docs/plans/optimize/zircon_plugins/17-first-party-virtual-geometry-source-runtime-editor-dist-catalog-asset-cook-cluster-page-streaming-culling-raster-product-integration-review.md` | Remove unconditional default VG cook from OBJ translation; own versioned recipe admission, canonical mesh input, cached artifacts and source/native/render closure. |
| `docs/plans/optimize/zircon_runtime/25-filesystem-path-uri-vfs-mount-watch-sandbox-atomic-io-review.md` | Provide immutable main-source snapshots and bounded resolver-owned MTL/texture reads with sandbox/path admission, hashes and no translator main-file reopen. |
| `docs/plans/optimize/zircon_runtime/59-runtime-task-execution-job-scheduler-handle-dependency-cancellation-thread-budget-timer-shutdown-diagnostics-product-integration-review.md` | Schedule translated mesh payloads and derived recipes with dependency edges, priorities, worker budgets, progress, cancellation, generation receipts and bounded shutdown. |
| `docs/plans/optimize/zircon_runtime/85-runtime-asset-import-source-discovery-importer-recipe-subasset-derived-data-artifact-cook-package-incremental-build-worker-determinism-product-integration-current-source-review.md` | Own metadata/payload/recipe phase separation, dependency currentness, incremental reimport, last-good publication, deterministic cache keys and non-C artifact roots. |
| `docs/plans/optimize/zircon_runtime/86-runtime-asset-type-schema-imported-payload-project-document-validation-dependency-serialization-versioning-product-integration-current-source-review.md` | Define versioned translated OBJ/material/section metadata and validate malformed/unsupported data before canonical artifact publication. |
| `docs/plans/optimize/zircon_runtime/87-runtime-asset-reference-identity-locator-guid-subasset-redirector-rename-move-resolution-repair-migration-product-integration-current-source-review.md` | Replace ordinal-only `MeshN/Primitive0` identity with stable normalized object/group/material identity and deterministic duplicate disambiguation/reimport migration. |
| `docs/plans/optimize/zircon_runtime/93-runtime-mesh-geometry-section-lod-instancing-skinning-morph-deformation-bounds-collision-streaming-product-integration-current-source-review.md` | Make one MeshAsset the canonical geometry owner; root models retain references/sections only, publication moves buffers, and VG/SDF/indices/vertex channels are not cloned into two retained payloads. |
| `docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md` | Expose OBJ import settings, dependencies, diagnostics, progress/currentness and generation-safe reimport without blocking the editor thread. |
| `docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md` | Own latest-wins import/cook job admission, edit-burst coalescing, cancellation, stale-result rejection, progress and shutdown behavior. |
| `docs/plans/optimize/zircon_editor/32-model-mesh-skeleton-geometry-import-lod-collision-retarget-preview-authoring-review.md` | Own material/section preview, explicit VG/SDF/tangent/LOD/collision recipes and exact-artifact preview/diagnostics for imported OBJ meshes. |
| `docs/plans/optimize/zircon_tooling/08-shared-derived-data-cache-build-cache-remote-execution-artifact-reuse-review.md` | Cache canonical mesh and enabled recipe outputs by source/dependency/settings/schema/cooker/target/profile identity under configured non-C roots. |
| `docs/plans/optimize/zircon_tooling/32-hot-path-catalog-algorithmic-complexity-data-movement-batching-cache-locality-performance-governance-review.md` | Govern double source reads, serial project import, `192 * V + 8 * I` retained duplicate geometry, VG duplication, transient SDF clone and cold microbench displacement. |
| `docs/plans/optimize/zircon_tooling/07-performance-benchmark-profile-capture-symbol-crash-evidence-baseline-review.md` | Define fixed OBJ corpora and WPR/ETW/import/render receipts with p50/p95/p99, throughput, RSS, bytes/copies, queue/main-thread time, cache outcomes and energy/import. |

## Protected ledger disposition

Do not mark `docs/plans/performance/review.md` complete yet. Keep the module pending until owner plans absorb the findings and a managed current-source binary passes canonical-provider, dependency/material, async import/cook, single-copy artifact, reimport, WPR/ETW, rendered-output and power gates. The concise protected-ledger row should eventually use only the module/folder name, per the root performance-plan convention.

No milestone commit or WeCom completion notice is warranted by static review alone.
