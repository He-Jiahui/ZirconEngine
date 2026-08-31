# Plugins20 Bounded Native Import Response

- Date: 2026-08-21
- Owner: `optimize-plugins20-bounded-native-import-r2-01a00797-20260821`
- Source plan: `docs/plans/optimize/zircon_plugins/20-plugin-sdk-example-native-editor-fixture-test-carrier-artifact-isolation-product-truth-review.md`, `P1-27`, `P1-28`, `P1-29`
- Status: implementation complete; grouped managed regression and release measurements pending

## Current-source convergence receipt

- Ownership transfer preview request: `15521ce76e4146a899d86c06cec2ed9f`.
- Ownership transfer apply request: `9d0c5c3e088b43b28035c0572c4b51a5`.
- Applied fingerprint: `098667d0ba88a5e07592f6132e85ff0c17721441fecc4b46aa40575fa5eb81b4`.
- Current session: `root-runtime-interface03-activate-link-failure-20260831`.
- Shared static/model ticket: `d49fbe45b4534105bc4be8fc36273fec` (queued, 7 Python tests).
- Release performance ticket: `5ad3550bb264437b9e10aea407c6f3ba` (queued; exact ignored bounded-response benchmark).
- Shared model: `tools/plugins_texture_native_pressure.py`, source manifest `DEF4AD94090A71DB775902D1190AA327F94910C7B0AF1E9FB38BB2C1553C5B37`.
- Current source hashes: `lib.rs` `3B1CE1E31A1F8B1B97AE12A3AF7DC2A0B18CB35111F0A1A9822145EAB0BC698C`; `tests.rs` `BC50857F300570DB9F465EAF8D729076D253266D0F779838A91535D96F5DA52C`; shared model `5FFA4EF8EA38EC84143374E4471E14B7AEF4F437B6179E7C94412516844B6981`.

The current-source model is structural evidence, not wall-clock timing. Across eight 131,101-byte encodes it changes full response-sized buffers `16 -> 8`, source-text clone bytes `1,048,808 -> 0`, and intermediate metadata buffers `8 -> 0`, while retaining the explicit 64 KiB metadata, 256 KiB source, and 1 MiB host-output limits. The exact ignored 21-pair release benchmark remains authoritative for timing and must satisfy bounded P95 `<= 110%` of legacy before integration or WeCom publication.

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
