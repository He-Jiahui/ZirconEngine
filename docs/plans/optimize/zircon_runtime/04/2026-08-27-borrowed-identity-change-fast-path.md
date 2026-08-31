---
title: Runtime04 Borrowed Identity Change Fast Path
category: zircon_runtime
report_id: Runtime04-borrowed-identity-change-fast-path-2026-08-27
date: 2026-08-31
session_id: root-runtime04-borrowed-identity-release-r2-20260831
implementation_status: implementation_complete
validation_status: local_contract_passed_managed_validation_pending
---

# Runtime04 Borrowed Identity Change Fast Path

## Scope

This slice removes the unconditional deep clone of the projected metadata identity-change batch
before duplicate GUID normalization. Full project scans normally have no additional watch delta,
so the registry now reads the already-owned batch through a borrowed slice. A non-empty watch delta
still produces one ordered `base + watch` buffer.

## Change

- Return `Cow::Borrowed` when watch changes are absent or empty.
- Allocate the merged buffer once at the exact combined capacity when watch changes are present.
- Preserve base-before-watch ordering and the existing optional-empty batch behavior.
- Keep duplicate GUID ownership and document mutation inside `AssetRegistryIndex` unchanged.

## Deterministic Performance Evidence

The historical standalone Rust model uses 16,384 rename changes with two owned URI strings per change, eight
normalizations per sample, and 15 alternating legacy/optimized samples.

| No-watch identity-change merge | Before | After | Reduction |
|---|---:|---:|---:|
| Allocations per normalization | 32,769 | 0 | 100.000% |
| Allocated bytes per normalization | 3,768,320 | 0 | 100.000% |
| P50 for eight normalizations | 101,761,400 ns | 100 ns | 100.000% |
| P95 for eight normalizations | 215,824,100 ns | 100 ns | 100.000% |

The model verifies that the borrowed projection has the same length, order, and values as the
legacy cloned projection. Its checksum is `1,966,080`.

Release-r2 moves the acceptance gate into the crate and invokes the production
`merged_identity_changes` helper. The fixture retains 16,384 changes, four merges per sample, four
paired warmups, and 21 alternating sample pairs. It reports both raw arrays and nearest-rank
P50/P95. Acceptance requires checksum parity, `65,536 -> 0` cloned `AssetChange` rows per sample,
and at least 95% lower P50 and P95.

## Validation

- Current baseline is Git HEAD `14c89f9776bed828cc85e05e4b9914b3f8d1e784`, coordinator epoch
  `575`.
- Release-r2 TDD RED: the new release-gate guard failed while the existing three source contracts
  passed.
- Release-r2 GREEN: `python -m unittest
  tools.tests.test_runtime04_borrowed_identity_change_fast_path_performance_contract -v` passes
  4/4 contracts.
- Exact-file `rustfmt +1.94.1 --check` and scoped diff validation pass.
- The standalone optimized Rust model compiles with `rustc --edition 2021 -C opt-level=3` and
  enforces at least 99% allocation/byte reduction and at least 95% P50/P95 reduction.
- The in-source Rust tests cover borrowed no-watch/empty-watch behavior and ordered non-empty
  merging.
- Release-r2 validation request `81319061685c4437b56cc24affb9bab0` batches both behavior tests and
  the ignored release benchmark. The managed command is `cargo +1.94.1 test -p zircon_runtime
  --locked --release --jobs 1 -- identity_change_merge_ --include-ignored --nocapture
  --test-threads=1`, with three expected tests. Cargo validation, commit, push, and WeCom remain
  pending until the ticket returns the raw arrays and exact P50/P95.

## Remaining Parent-plan Work

Runtime04 still owns the larger typed artifact, asynchronous residency, semantic section streaming,
pack mounting, signing, and publication tasks. This slice only removes avoidable identity-change
copying before duplicate GUID normalization.
