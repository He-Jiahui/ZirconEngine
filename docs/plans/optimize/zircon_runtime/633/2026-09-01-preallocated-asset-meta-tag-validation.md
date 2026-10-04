---
title: Runtime633 Preallocated Asset Meta Tag Validation
category: zircon_runtime
report_id: Runtime633-preallocated-asset-meta-tag-validation-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime633 Preallocated Asset Meta Tag Validation

Asset metadata tag validation now reserves duplicate-detection membership from the serialized TOML
array length and typed tag-slice length. The previous local hash sets grew from zero for every root
or labeled-entry tag list.

Filtering of non-string TOML values, per-tag validation, first-duplicate error identity, and stable
serialized tag ownership remain unchanged. Serialized arrays may reserve a conservative upper
bound when they contain non-string values.

The ignored Windows Release benchmark emits `RUNTIME633_PREALLOCATED_ASSET_META_TAG_BENCH_V1` over
17 alternating sample pairs with 65,536 unique tags. The gate requires preallocated P95 to be at
most 80% of the unreserved path P95.

No direct Cargo validation was run. The coordinator owns combined Runtime633/Editor633 Windows
Release compilation, prefixed regression tests, ignored benchmarks, measured record finalization,
manifest-only commit/push, and one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Runtime633 is prepared with Editor633 under request
`runtime633-editor633-meta-widget-capacity-performance-20260901iw-v1`. Receipt, validation ticket,
measured P95, pushed SHA, and notification result are recorded only after coordinator completion.
