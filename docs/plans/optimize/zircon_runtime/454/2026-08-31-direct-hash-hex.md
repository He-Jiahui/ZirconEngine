---
title: Runtime454 Direct Hash Hex
category: zircon_runtime
report_id: Runtime454-direct-hash-hex-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime454 Direct Hash Hex

Project asset content hashing now writes the 16 lowercase hexadecimal nibbles directly into one
exact-capacity `String`. The former scan/import path sent every completed `u64` hash through the
generic formatting machinery before storing it in asset metadata.

The hash algorithm, fixed width, lowercase alphabet, leading zeroes, and digest bytes remain
unchanged. Regression coverage compares boundary integers and representative content buffers with
the former `format!("{value:016x}")` output.

The ignored Windows Release benchmark emits `RUNTIME454_DIRECT_HASH_HEX_BENCH_V1` over 17
alternating paired samples, each encoding 262,144 mixed `u64` values. Both paths allocate one
result string; the optimized path replaces generic integer formatting with 16 direct nibble
writes. The gate requires `optimized_p95_ns <= legacy_p95_ns * 0.65` (at least 35% lower P95).

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Runtime454 is prepared with Editor384 under request
`runtime454-editor384-performance-batch-20260831ev-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
