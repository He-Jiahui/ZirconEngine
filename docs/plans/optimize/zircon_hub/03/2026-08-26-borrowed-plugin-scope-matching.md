---
title: Hub03 Borrowed Plugin Scope Matching
category: zircon_hub
report_id: Hub03-borrowed-plugin-scope-matching-2026-08-26
date: 2026-08-26
session_id: root-hub03-two-task-performance-batch-r2-20260831
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Hub03 Borrowed Plugin Scope Matching

## Scope

This slice removes transient lowercase strings and eager fallback strings from local plugin-manifest
discovery. It does not change plugin discovery roots, duplicate filtering, manifest parsing, scope
ordering, editor-target/capability semantics, module counting, public APIs, or remote Marketplace,
account, organization, cloud-repository, or provider behavior.

## Change

- Editor targets now use borrowed, case-insensitive comparisons after trimming instead of allocating
  a lowercase `String` for every classification.
- Editor capabilities now compare the borrowed ASCII prefix case-insensitively, preserving the
  existing `editor.` prefix contract without materializing a normalized string.
- Manifest `id` and `display_name` normalization now preserves an already canonical owned string and
  evaluates fallbacks only when the parsed field is absent or empty.
- A Rust regression proves canonical ownership is reused and fallback construction stays lazy. A
  Python source contract prevents eager normalization or lowercase allocations from returning.

## Deterministic Performance Evidence

The original independent release model scans 32,768 canonical manifests. Each manifest performs
six editor scope classifications and normalizes both `id` and `display_name`; each run contains 21
paired samples with alternating baseline/optimized order.

| Evidence | Eager/lowercase baseline | Borrowed/lazy path | Result |
| --- | ---: | ---: | ---: |
| Scan checksum | 895,284 | 895,284 | identical |
| Total allocations | 327,680 | 0 | 327,680 fewer; 100% reduction |
| Run 1 P50 | 35.0910 ms | 6.7669 ms | 80.716% faster |
| Run 1 P95 | 117.4644 ms | 8.1217 ms | 93.086% faster |
| Run 2 P50 | 36.6352 ms | 7.0924 ms | 80.640% faster |
| Run 2 P95 | 62.2540 ms | 11.3362 ms | 81.790% faster |
| Run 3 P50 | 37.3755 ms | 7.1394 ms | 80.898% faster |
| Run 3 P95 | 62.4131 ms | 9.6186 ms | 84.589% faster |

The managed gate requires the exact checksum, exact allocation counts, 100% modeled allocation
reduction, and at least 75% P50 and P95 improvement.

That model is now embedded in the actual plugin catalog test module. The optimized side directly
calls `is_editor_target`, `is_editor_capability`, and `non_empty_or_else`; a test-only counting
allocator resets after input cloning and measures only the timed scan. The executable gate requires
exactly `327,680 -> 0` allocations, output checksum parity, and emits
`HUB03_PLUGIN_SCOPE_MATCHING_BENCH_V1` with raw alternating samples, allocated bytes, P50, and P95.

## Acceptance

- TDD RED observed three failures: target and capability classifiers still allocated lowercase
  strings, and manifest normalization lacked a lazy ownership-preserving helper.
- `tools.tests.test_hub03_borrowed_plugin_scope_matching_performance_contract` passes 5/5 locally;
  the combined Hub03 projection/plugin contract batch passes 8/8.
- Exact production `rustfmt --check` and scoped diff checks pass locally; the embedded native
  benchmark remains pending the managed Cargo lane.
- Seventeen focused Rust behavior/performance tests are selected by `hub03_` across the three
  modules. The managed command is `cargo test --manifest-path zircon_hub/Cargo.toml --bin
  zircon_hub --locked --release --jobs 1 -- hub03_ --include-ignored --nocapture
  --test-threads=1`.
- Commit integration and automatic WeCom performance notification remain gated on managed
  validation and the repository's independent-review policy.

## Remaining Parent-plan Work

Hub03 still owns real Marketplace catalog/search/detail/install flows, account/authentication,
organization context, cloud repositories, provider connections, rollback, observability, and
product-scale qualification. This record only accepts the local plugin-manifest scan slice.
