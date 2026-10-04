---
title: Runtime Render Graph Current Source Protected Plan Routing
date: 2026-08-23
status: routing_only
source_report:
  - docs/plans/performance/01/2026-08-23-runtime-render-graph-current-source-and-p0-revalidation.md
---

# Runtime Render Graph Current Source Protected Plan Routing

This file records required owner-plan updates without editing protected ledgers or canonical plans from this pass.

| Existing owner | Required adoption |
|---|---|
| `docs/plans/optimize/zircon_runtime/89-runtime-render-graph-builder-compiler-resource-lifetime-pass-culling-transient-aliasing-barrier-queue-scheduling-execution-product-integration-current-source-review.md` | Mark `RG89-P0-001` statically implemented by final duplicate-name validation and pass-ID execution, but keep dynamic acceptance open. Adopt the current 18-file fingerprint and preserve `RG89-P0-002/003` as open compile-to-execute P0s. Route M1-M8 in dependency order. |
| `docs/plans/optimize/zircon_runtime/09a-rhi-render-graph-gpu-lifetime-review.md` | Own native subresource barrier consumption, queue/fence support, completion-qualified reuse, device capability validation and sparse residency provider/fallback contracts. Consume the compiled packet; do not rediscover graph semantics. |
| `docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md` | Make typed resource schema, subresource access/version, compiler-owned hazard/culling, async fork/join, physical lifetime and immutable execution packet the RDG alignment path. Unreal RDG remains the primary source standard. |
| Render feature and plugin owners | Replace name-derived resource formats with an explicit `RenderResourceSchema`; reject unknown schemas before cache/graph compilation. Plugin features must declare storage-capable format/usage and fallback policy. |
| Graphics pipeline/cache owner | Normalize the final resource/pass set before cache insertion, qualify against device capabilities, remove exact-size churn where a measured size-class policy is valid, and publish cache/compile/eviction receipts. |
| Editor performance/diagnostics owner | Add a read-only view over compiled packet generations, cache results, allocation/alias/barrier/queue summaries and current acceptance receipts. Viewing diagnostics must not rebuild or mutate the graph. |

Protected `docs/plans/performance/review.md` and `docs/plans/performance/pending.md` remain untouched. This module is eligible for a concise `render_graph` folder entry only after the protected-ledger owner adopts this currentness report; dynamic acceptance still requires a current-source executable plus WPR/ETW and RenderDoc evidence.
