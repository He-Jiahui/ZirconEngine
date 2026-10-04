---
title: Runtime633 Bitset Export Packaging Strategies
category: zircon_runtime
report_id: Runtime633-bitset-export-packaging-strategies-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime633 Bitset Export Packaging Strategies

Export-profile duplicate diagnostics and sanitization now represent the three closed packaging
strategies with a stack `u8` bitset. The previous paths allocated a temporary vector and performed
linear membership scans for every strategy.

The exhaustive strategy-to-bit match makes a future enum extension a compile-time migration point.
Duplicate diagnostic order, first-occurrence retention, and the existing diagnostic output capacity
remain unchanged.

The ignored Windows Release benchmark emits
`RUNTIME633_BITSET_EXPORT_PACKAGING_STRATEGY_BENCH_V1` over 17 alternating sample pairs and 65,536
strategy inputs. The gate requires bitset dedupe P95 to be at most 85% of temporary-vector P95.

No direct Cargo validation was run. The coordinator owns the combined Runtime633/Editor633 Windows
Release regression and performance validation. Measured P95, commit, push, and WeCom outcome are
recorded only after coordinator completion.
