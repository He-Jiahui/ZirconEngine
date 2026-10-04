---
title: Runtime628 Preallocated glTF Skin Dependency Index
category: zircon_runtime
report_id: Runtime628-preallocated-gltf-skin-dependency-index-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime628 Preallocated glTF Skin Dependency Index

glTF skin subasset import now reserves its dependency membership index from the joint iterator
bound plus the actually present skeleton-root, skeleton-asset, and inverse-bind-matrix dependencies.
The previous index grew from zero for every imported skin.

Joint traversal order, first dependency retention, generated URI identity, skeleton publication,
and inverse-bind-matrix handling remain unchanged. The capacity calculation uses iterator metadata
and option presence instead of a raw fixed allowance.

The ignored Windows Release benchmark emits `RUNTIME628_PREALLOCATED_GLTF_SKIN_DEPENDENCY_BENCH_V1`
over 17 alternating sample pairs with 32,768 long dependency URIs. The gate requires preallocated
membership P95 to be at most 85% of the unreserved path P95.

No direct Cargo validation was run. The coordinator owns combined Runtime628/Editor628 Windows
Release compilation, prefixed regression tests, ignored benchmarks, measured record finalization,
manifest-only commit/push, and one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Runtime628 is prepared with Editor628 under request
`runtime628-editor628-gltf-watch-membership-performance-20260901ir-v1`. Receipt, validation ticket,
measured P95, pushed SHA, and notification result are recorded only after coordinator completion.
