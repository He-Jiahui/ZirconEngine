---
title: Runtime457 Direct Cache Fingerprint Hex
category: zircon_runtime
report_id: Runtime457-direct-cache-fingerprint-hex-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime457 Direct Cache Fingerprint Hex

Artifact library cache keys now write their 16 lowercase hexadecimal fingerprint nibbles directly
into one exact-capacity string. The former cache lookup path invoked generic formatting after
hashing the source digest, importer version, and configuration digest.

Hasher input, fixed width, leading zeroes, lowercase alphabet, and resulting fingerprint bytes
remain unchanged. Regression coverage compares integer boundaries and a complete cache key with
the former formatter.

The ignored Windows Release benchmark emits `RUNTIME457_DIRECT_CACHE_FINGERPRINT_HEX_BENCH_V1`
over 17 alternating paired samples, each encoding 262,144 mixed `u64` values. Both paths allocate
one output string; the optimized path performs 16 direct nibble writes. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.65` (at least 35% lower P95).

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Runtime457 is prepared with Editor387 under request
`runtime457-editor387-performance-batch-20260831ey-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
