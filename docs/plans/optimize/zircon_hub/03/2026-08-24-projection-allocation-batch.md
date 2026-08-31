---
title: Hub03 Projection Allocation Batch
category: zircon_hub
report_id: Hub03-projection-allocation-batch-2026-08-24
date: 2026-08-24
session_id: root-hub03-two-task-performance-batch-r2-20260831
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Hub03 Projection Allocation Batch

## Scope

This batch reduces allocation pressure in the current local catalog and disabled remote-service
read model without enabling Marketplace, account, organization, or Cloud capabilities. Those
surfaces remain fail-closed as required by Hub03.

## Task 1: Borrow Static Coming-soon Copy

The 15-entry coming-soon projection previously allocated owned strings for six fields whose values
always come from static localized copy. `HubComingSoonEntry` now stores those fields as
`Cow<'static, str>` and retains ownership only for the composed `meta` field. Serde continues to
emit the same camel-case string payload.

The deterministic string-allocation count per projection is `105 -> 15`, an 85.7% reduction. The
release gate uses 21 alternating sample pairs, 400 projections per sample, nearest-rank
percentiles, and requires optimized P95 to be at most 80% of the legacy owned-copy path.

## Task 2: Zero-allocation Catalog Classifiers

Plugin maturity and learn-resource category classification previously created a lowercase `String`
for every call. Both now use byte-window ASCII case-insensitive matching and preserve the existing
Chinese display-copy branches. Temporary heap allocations per classifier call are `1 -> 0`, a
100% reduction.

The release gate runs 10,000 mixed classifier calls per sample over 21 alternating sample pairs and
requires optimized P95 to be at most 80% of the legacy lowercase-allocation path.

## Validation

- TDD red state: the new source contract failed 3/3 before implementation.
- Source contract after implementation: 3/3 passed; the combined Hub03 projection/plugin contract
  batch passes 8/8.
- `rustfmt`, Python bytecode compilation, and scoped whitespace validation: passed.
- Existing inline behavior tests are named with the shared `hub03_` filter. Both projection
  benchmarks emit alternating raw samples, P50/P95, and deterministic allocation deltas.
- Focused behavior tests plus all three ignored release performance gates are pending one managed
  Windows-native coordinator ticket.
- No local Cargo lane or cargo dry-run was started, polled, or terminated.

## Async Validation

This record shares one immutable-source coordinator ticket with borrowed plugin scope matching.
The managed Windows Rust 1.94.1 command is `cargo test --manifest-path zircon_hub/Cargo.toml --bin
zircon_hub --locked --release --jobs 1 -- hub03_ --include-ignored --nocapture --test-threads=1`.
The filter selects 17 behavior/performance tests across the coming-soon, catalog view-model, and
plugin catalog modules in one Cargo invocation. Commit and automatic WeCom publication remain
pending until the managed ticket returns all three structured performance rows.

## Remaining Parent-plan Work

This batch does not complete signed Marketplace snapshots, provider registries, tenant isolation,
entitlements, package trust, or Cloud snapshot/CAS synchronization. Hub03 P0 and the wider product
qualification matrix remain open.
