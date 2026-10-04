---
title: Runtime214 external-access lifetime binary lookup
category: zircon_runtime
report_id: Runtime214-external-access-lifetime-binary-lookup-2026-09-29
date: 2026-09-29
session_id: astra-optimize-20260926-batch-a
plan_source: docs/plans/optimize/zircon_runtime/214-runtime-render-graph-builder-compiler-resource-lifetime-pass-culling-transient-aliasing-barrier-queue-scheduling-execution-current-working-tree-review.md
implementation_status: source_candidate_pending_validation
validation_status: focused_static_checks_passed_managed_validation_pending
performance_status: lookup_complexity_improved_product_gate_pending
---

# Runtime214: binary-search external access lifetimes

## Change

`build_external_access_packet` previously scanned every resource lifetime for each live external access. `RenderGraphBuilder::resource_lifetimes` already sorts lifetimes by name, and compilation rejects duplicate resource names. The packet builder now binary-searches that sorted slice using the access name, then verifies the matched lifetime's resource ID before copying its binding and descriptor. This keeps the resource ID authoritative and adds no lookup index allocation.

For `R` lifetimes and `E` live external accesses, lookup work changes from O(E × R) to O(E log R). This is a source-level complexity result; no latency or frame-time improvement has been measured.

## Evidence and remaining gates

- Current whole-file SHA-256: `zircon_runtime/src/render_graph/graph/external_access_packet.rs` = `ae7b4bd70ced4326b509d74601ae710911e07b94b20cc982a9289a060549de18`.
- Added `external_access_packet_looks_up_multiple_resources_by_sorted_name`. It imports three buffers in non-lexical name order and checks exact access identity, resource identity, binding, and typed descriptor for every packet entry.
- `rustfmt --edition 2021 --check` and scoped `git diff --check` passed. No Cargo command ran for this slice; the focused regression has not run in managed validation.
- Managed Runtime correctness validation remains pending. Runtime214 G14 Release p50/p95/p99, allocation/RSS, fixed-scene comparison, hardware profile, and explicit performance ceiling remain open.
- The neighboring RG166-P1-026 executor fallback work remains partial; this packet lookup does not close it.
