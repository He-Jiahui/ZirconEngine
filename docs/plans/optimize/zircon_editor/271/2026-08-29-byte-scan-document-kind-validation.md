# Editor271 Byte-Scan DocumentKind Validation

- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime326-editor271-performance-batch-20260829ay-v1`

## Scope

Document-kind parsing previously split the string into segments and scanned each segment as Unicode
characters. Validation now uses one ASCII byte state scan that tracks segment boundaries without
allocating split iterators or character iterators. Accepted identifiers, rejected input, and owned
error values remain unchanged.

## Static Evidence

- Validation traversals: `segment split + per-segment char scan -> one byte scan`.
- Temporary segment materialization: `0` before and after; no collection is introduced.
- Original invalid input remains owned by `DocumentKindError`.

## Performance Gate

The ignored Windows release benchmark emits `EDITOR271_BYTE_SCAN_DOCUMENT_KIND_BENCH_V1`. It
compares the legacy split/character validation with the byte state scan over 8,192 identifiers
across 31 interleaved sample pairs and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only staging, commit, push to `origin/main`, and the one-shot
WeCom result containing exact performance data, test result, commit SHA, and branch.
