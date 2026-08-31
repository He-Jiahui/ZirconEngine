# Backend source-range invariant owner record (2026-08-30)

## Scope

Affected plans: Text02, Text03, the engine code-structure convention, and the June code-review
findings. This record tracks the shaping-request identity guard completed while managed product
validation remains pending.

## Finding

`BackendShapeRequest` carries a local UTF-8 text view and an absolute source identity range. The
existing canonicalization path normalized language and features but did not verify that the range
span covered the supplied text. Malformed requests could therefore reach fallback, cache-key
construction, or backend projection with incompatible source identities.

The valid non-zero-offset form is intentionally preserved: a slice such as `"locale"` may use
`TextRange { start: 11, end: 17 }`, because shaping publishes local clusters translated by the
absolute start. The guard rejects only a reversed range or a span whose byte length differs from
the supplied UTF-8 text.

## Completed implementation

- `BackendShapeRequest::canonicalized` now uses checked subtraction and requires
  `source_range.end - source_range.start == text.len()`.
- Invalid source identity is returned as `TextLayoutError::BidiInvariant` before language
  normalization, fallback itemization, cache lookup, or backend shaping.
- Model tests cover a valid non-zero absolute range, a mismatched span, and a reversed range; the
  multi-hard-line line-break test also records the expected absolute ranges for each local slice.
- The line-break review was corrected: hard-line shaping passes a local slice together with its
  matching absolute range by design, so no erroneous local/absolute offset rewrite was made.

## Evidence and status

- Targeted Rustfmt: passed for `shaped_run.rs` and its model tests.
- Scoped `git diff --check`: passed, with existing LF/CRLF conversion warnings only.
- Python/static contract checks: Composite activation 20/20, text pointer 1/1, text segment cache
  13/13, decoration source-map pressure 4/4, and layout-order authority 4/4. The broad framework
  boundary script exceeded the local 120-second scan budget without producing a test failure.
- Managed Cargo, real WGPU/PNG, 31-sample profile, RSS/power, and matched Unreal workload:
  pending.
- Screenshot: not produced; this identity-guard slice has no independently valid rendered frame.

Status:
`backend_source_range_invariant_static_implemented /
malformed_identity_rejected_before_backend /
managed_product_validation_pending`.
