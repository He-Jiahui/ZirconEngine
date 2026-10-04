---
title: Runtime596 UI V2 Component Document Reuse
category: zircon_runtime
report_id: Runtime596-ui-v2-component-document-reuse-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime596 UI V2 Component Document Reuse

UI v2 component instancing now has an owned internal entry point. The borrowed public contract
clones its source once, while component-root validation directly consumes the temporary document it
already owns. During expansion the source remains shared through `Arc`; after traversal releases all
task owners, the instancer recovers that document and replaces only its root and node table instead
of deep-cloning the complete document again and immediately discarding the cloned nodes and
components.

Asset metadata, imports, tokens, stylesheets, root expansion order, generated node identifiers,
component-cycle validation, slot routing, and the public borrowed API retain their previous
contracts. Focused tests require borrowed/owned result equality and reject restoration of the
second whole-document clone.

The ignored Windows Release benchmark emits `RUNTIME596_COMPONENT_DOCUMENT_REUSE_BENCH_V1` over
21 alternating sample pairs, 1,024 UI nodes, and 24 scaffold conversions per sample. The legacy
model performs the two whole-document clones used by the former traversal/output setup; the
optimized model transfers ownership and recovers the source allocation. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.65`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and
one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Runtime596 is prepared with Editor596 under request
`runtime596-editor596-component-projection-performance-20260901ho-v1`. Receipt, validation ticket,
measured P95, pushed SHA, and notification result are recorded only after coordinator completion.
