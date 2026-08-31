---
title: Runtime586 Image Resource Group Extend
category: zircon_runtime
report_id: Runtime586-image-resource-group-extend-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime586 Image Resource Group Extend

The RHI UI image resource table now extends another table by moving each resource key's complete
generation map. The former path flattened every generation into an entry, cloned the same resource
key for every generation, hashed that key for every insertion, and rebuilt the destination ordered
generation map one item at a time.

Incoming resources continue to replace the same key/generation in the retained table, retained
generations remain present, and the public entry iterator remains available to existing conversion
callers. Regression coverage verifies those merge semantics and requires the group-append source
shape.

The ignored Windows Release benchmark emits
`RUNTIME586_IMAGE_RESOURCE_GROUP_EXTEND_BENCH_V1` over 21 alternating sample pairs, 256 resource
keys, 16 retained generations per key, and 64 incoming generations per key. The legacy model
performs 16,384 resource-key clones and hashes per sample; the optimized model moves 256 keys,
performs 256 outer-map hashes, and appends the ordered generation groups. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.50`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push,
and one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-08-31)

Runtime586 is prepared with Editor586 under request
`runtime586-editor586-image-panel-performance-20260831hj-v1`. Receipt, validation ticket, measured
P95, pushed SHA, and notification result are recorded only after coordinator completion.
