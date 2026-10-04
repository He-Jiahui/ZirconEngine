---
title: Runtime629 Preallocated glTF Mesh Dependency Index
category: zircon_runtime
report_id: Runtime629-preallocated-gltf-mesh-dependency-index-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime629 Preallocated glTF Mesh Dependency Index

glTF mesh subasset import now reserves its dependency membership index from the primitive count.
Each primitive can add at most its generated mesh-primitive URI and one material URI, so the exact
safe upper bound is twice the primitive count. The previous index grew from zero for every mesh.

Primitive traversal order, first dependency retention, generated URI identity, material fallback,
and imported entry publication remain unchanged. Saturating multiplication keeps the capacity
calculation defined even for an unrealistically large malformed input.

The ignored Windows Release harness emits `RUNTIME629_PREALLOCATED_GLTF_MESH_DEPENDENCY_BENCH_V2`.
It is a helper microbenchmark: it measures a synthetic `HashSet` insertion loop rather than
`add_gltf_mesh_subassets` with an admitted glTF source and import publication. The 32,768 synthetic
dependency identities are not a project-manifest root workload and must not be read as an approved
manifest limit. The harness collects 31 alternating sample pairs and reports nearest-rank
p50/p95/p99 for both paths.

A managed Windows Release measurement through the real caller (`add_gltf_mesh_subassets`) remains pending. It must
exercise an admitted source snapshot, primitive traversal, dependency publication, and the caller's
allocation behavior; this microbenchmark alone is not product acceptance.

No direct Cargo validation was run. The coordinator owns combined Runtime629/Editor629 Windows
Release compilation, prefixed regression tests, ignored benchmarks, measured record finalization,
manifest-only commit/push, and one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Runtime629 is prepared with Editor629 under request
`runtime629-editor629-gltf-export-diagnostic-capacity-performance-20260901is-v1`. Receipt,
validation ticket, measured P95, pushed SHA, and notification result are recorded only after
coordinator completion.
