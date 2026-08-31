---
title: Runtime458 Literal Endpoint Parse
category: zircon_runtime
report_id: Runtime458-literal-endpoint-parse-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime458 Literal Endpoint Parse

`NetEndpoint::to_socket_addr` now parses the host directly as an `IpAddr` and constructs the
`SocketAddr` with the stored port. This removes the temporary `host:port` allocation and the
second combined-address parse from every literal endpoint conversion.

The change also closes the IPv6 correctness gap identified by network review P1-7: bare IPv6 and
bracketed IPv6 hosts now both resolve to the intended address and port, while endpoint display
adds brackets around bare IPv6 hosts. IPv4, wildcard address, zero port, and invalid-host
diagnostics retain their existing contracts.

The ignored Windows Release benchmark emits `RUNTIME458_LITERAL_ENDPOINT_PARSE_BENCH_V1` over 17
alternating paired samples, each parsing 131,072 IPv4 endpoints. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.70` (at least 30% lower P95).

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Runtime458 is prepared with Editor388 under request
`runtime458-editor388-performance-batch-20260831ez-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
