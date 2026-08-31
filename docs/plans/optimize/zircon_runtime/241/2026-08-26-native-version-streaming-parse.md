# Runtime241 Native Version Streaming Parse

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime241-editor187-performance-batch-20260826gu-v1`

## Problem

Every native plugin compatibility comparison collected the engine release components into a Vec
before accepting the fixed `major.minor[.patch]` shape. Valid versions therefore allocated a
container even though the parser consumes at most four borrowed segments.

## Optimization

- Read the required major/minor, optional patch, and one extra sentinel segment directly.
- Reject an extra fourth segment before numeric parsing to preserve invalid-shape error precedence.
- Preserve prerelease/build suffix trimming, optional patch defaulting, and typed component errors.

## Regression Contract

The `optimization_batch_20260826gu_` Runtime tests cover two- and three-component versions plus
shape-versus-component error precedence, enforce the no-Vec source contract, and provide an ignored
paired release benchmark emitting `RUNTIME241_NATIVE_VERSION_STREAMING_PARSE_BENCH_V1`. It repeatedly
parses a valid suffixed version and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
