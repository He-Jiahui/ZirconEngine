---
title: Plugin Audio Importer Current Source Protected Plan Routing
date: 2026-08-24
status: routing_only_protected_ledgers_unchanged
source_review: docs/plans/performance/01/2026-08-24-plugin-audio-importer-current-source-performance-review.md
---

# Plugin Audio Importer Current Source Protected Plan Routing

The protected `docs/plans/performance/review.md`, `pending.md` and numbered/main plans are intentionally unchanged. Route the reviewed findings to the following canonical owners when their maintainers next update implementation scope:

| Owner | Routed responsibility |
|---|---|
| `docs/plans/optimize/zircon_plugins/06-first-party-plugin-source-editor-runtime-dist-catalog-profile-capability-closure-review.md` | Add the missing AudioImporter catalog dependency/provider branch and enforce source/library/native capability truth. |
| `docs/plans/optimize/zircon_plugins/07-first-party-asset-importer-source-dependency-subasset-artifact-determinism-sandbox-product-integration-review.md` | Own source lease, settings, decode admission, deterministic diagnostics and audio recipe publication. |
| `docs/plans/optimize/zircon_plugins/11-first-party-sound-source-runtime-editor-dist-catalog-mixer-spatial-reverb-timeline-product-integration-review.md` | Own short/streaming playback products, Kira/backend ownership, channel policy, first-play preparation and in-flight load coalescing. |
| `docs/plans/optimize/zircon_runtime/08b-audio-runtime-review.md` | Define Runtime audio asset/loading/decompression contracts and RT-safe bounded-buffer behavior. |
| `docs/plans/optimize/zircon_runtime/59-runtime-task-execution-job-scheduler-handle-dependency-cancellation-thread-budget-timer-shutdown-diagnostics-product-integration-review.md` | Admit probe/decode/cook/waveform/artifact jobs with byte budgets, priorities, cancellation and shutdown fences. |
| `docs/plans/optimize/zircon_runtime/64-runtime-resource-authority-asset-handle-load-request-state-machine-version-lease-cache-dependency-reload-cancellation-product-integration-review.md` | Remove typed-load deep copies, expose immutable leases, and coalesce generation-bound same-key loads. |
| `docs/plans/optimize/zircon_runtime/85-runtime-asset-import-source-discovery-importer-recipe-subasset-derived-data-artifact-cook-package-incremental-build-worker-determinism-product-integration-current-source-review.md` | Define audio source/cook/stream/seek/waveform recipe keys and last-good publication. |
| `docs/plans/optimize/zircon_runtime/86-runtime-asset-type-schema-imported-payload-project-document-validation-dependency-serialization-versioning-product-integration-current-source-review.md` | Replace universal resident f32 schema with versioned resident/streamed payloads and target layout/codec metadata. |
| `docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md` | Surface audio import/analysis queue, progress, cancellation and close-project cleanup without editor-thread decode. |
| `docs/plans/optimize/zircon_editor/17-sound-audio-clip-mixer-routing-effect-spatial-acoustic-timeline-authoring-review.md` | Own import settings, waveform/loudness artifacts, audition readiness, long-clip UX and reimport diagnostics. |
| `docs/plans/optimize/zircon_tooling/07-performance-benchmark-profile-capture-symbol-crash-evidence-baseline-review.md` | Capture cold/warm import, first/repeat play, WPR/ETW CPU/I/O/scheduling/power and backend underrun receipts. |
| `docs/plans/optimize/zircon_tooling/08-shared-derived-data-cache-build-cache-remote-execution-artifact-reuse-review.md` | Store content-addressed platform chunks, seek/index and analysis products without full duplicate PCM staging. |
| `docs/plans/optimize/zircon_tooling/25-memory-allocation-domain-budget-oom-pressure-fragmentation-pooling-cache-residency-observability-review.md` | Enforce source/decoded/scratch/resident/ring-buffer budgets and expose peak/lease/residency accounting. |
| `docs/plans/optimize/zircon_tooling/32-hot-path-catalog-algorithmic-complexity-data-movement-batching-cache-locality-performance-governance-review.md` | Track full-payload clone counts, cache-hit operation counts, concurrent duplicate work and duration-independent streaming memory. |

The immediate implementation order is Plugins06/07 provider truth, Runtime85/86 product schema, Runtime59/64 scheduling and ownership, then Plugins11/Runtime08B playback convergence. Editor17 and Tooling07 validate the complete authoring/import/playback path; they must not manufacture a second audio authority.

Dynamic acceptance remains pending. Do not mark the package accepted, create a milestone commit or send quantified WeCom results until the managed current-source binary completes the gates in the source review.
