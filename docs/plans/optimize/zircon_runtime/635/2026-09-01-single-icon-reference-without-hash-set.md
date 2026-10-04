---
title: Runtime635 Single Icon Reference Without Hash Set
category: zircon_runtime
report_id: Runtime635-single-icon-reference-without-hash-set-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime635 Single Icon Reference Without Hash Set

External UI icons can contribute at most one direct resource reference. Their projection now
normalizes the URI through the shared locator helper and returns zero or one reference directly,
instead of allocating both a vector and a hash set for an impossible duplicate.

Multi-reference UI document projection continues to use the same hash membership set and shared
fragment-stripping locator normalization. Inline SVG and missing external URI paths still return an
empty reference list.

The ignored Windows Release benchmark emits `RUNTIME635_SINGLE_ICON_REFERENCE_WITHOUT_HASH_BENCH_V1`
over 17 alternating sample pairs and 65,536 projections per sample. The gate requires the direct
path P95 to be at most 85% of the vector-plus-hash path P95.

No direct Cargo validation was run. The coordinator owns the combined Runtime/Editor regression and
performance validation. Measured P95, commit, push, and WeCom outcome are recorded only after
coordinator completion.
