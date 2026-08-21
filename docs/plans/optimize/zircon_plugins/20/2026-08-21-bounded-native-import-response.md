# Plugins20 Bounded Native Import Response

- Date: 2026-08-21
- Owner: `optimize-plugins20-bounded-native-import-r2-01a00797-20260821`
- Source plan: `docs/plans/optimize/zircon_plugins/20-plugin-sdk-example-native-editor-fixture-test-carrier-artifact-isolation-product-truth-review.md`, `P1-27`, `P1-28`, `P1-29`
- Status: implementation complete; grouped managed regression and release measurements pending

## Problem

The native fixture importer converted its wire `u64` metadata length to `usize` and added it to the
envelope offset without checked admission. A hostile length could therefore truncate on narrow
targets or overflow before the malformed request became a typed protocol error.

For accepted requests, the response path also built an owned `serde_json::Value` for the complete
response, copied that tree into an intermediate metadata vector, copied the vector into the final
envelope, and only then asked the host output sink to enforce its one MiB budget. Large valid JSON
therefore paid an avoidable source-text clone and a second full response buffer before admission.

## Change

- Request metadata is limited to 64 KiB and source JSON to 256 KiB. The wire conversion and end
  offset use checked conversion/addition, so malformed lengths return an error without panic.
- Borrowed response DTOs retain the same importer ID, locator, Data payload, migration report, and
  diagnostics while avoiding an owned response tree and source-text clone.
- `BoundedResponseWriter` writes JSON directly into the final envelope, refuses growth beyond the
  host output budget, and backfills the checked metadata length after serialization.
- Protocol, budget, source-structure, and legacy-semantic regressions moved to `src/tests.rs`,
  keeping the production root below the repository's large-file threshold.

## Deterministic Delta

For the release workload of 131,101 source bytes and eight encodes per sample:

| Metric | Legacy encoder | Bounded encoder | Delta |
|---|---:|---:|---:|
| full response-sized buffers per encode | 2 | 1 | 50% fewer |
| source-text clone bytes per sample | 1,048,808 | 0 | 100% fewer |
| intermediate metadata buffers per sample | 8 | 0 | 100% fewer |
| admitted metadata bytes | unbounded by fixture policy | 65,536 | explicit bound |
| admitted source bytes | unbounded by fixture policy | 262,144 | explicit bound |

The canonical JSON value remains owned because the response protocol carries both original text and
parsed JSON. This milestone does not claim allocation-free import or remove that protocol-level
duplication.

## Acceptance

- Seven non-ignored unit tests cover manifest projection, legacy response semantics, overflowing
  metadata length, source budget, host output budget, and the no-owned-response-tree source guard.
- The ignored release benchmark runs 21 alternating legacy/bounded pairs with eight encodes per
  sample, emits both raw timing arrays, and uses nearest-rank P95.
- Bounded response P95 may not exceed 110% of legacy while the deterministic full-buffer,
  source-clone, and intermediate-buffer reductions must remain exact.
- Rust 1.94.1 formatting and scoped diff checks pass.
- Cargo regression counts, actual intermediate metadata bytes, and measured P95 remain pending the
  post-Main plugin aggregate batch; no timing result is claimed by this record yet.

## Remaining Scope

Plugins20 still requires a product-level fixture role and shipping exclusion, real source/native
carrier parity, and host-owned streaming output. This slice only makes the current importer parser
and response allocation path bounded and non-panicking.
