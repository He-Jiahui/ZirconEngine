# App02 Artifact Zstd Bound Underflow Repair

## Finding

Main continuation job `06d69e9c76204ccf94fb9d79cde60ab2`, run
`093eabe96ddc4d5dbffa3c6d718265ed`, passed the preceding App02 Rust groups
but the real viewer panicked in `validate_artifact_compressed_payload_bytes`
before screenshot and GPU timing collection. `bool::then_some` eagerly
evaluated the small-input subtraction for a payload larger than 128 KiB,
causing unsigned underflow even though that margin did not apply.

## Scope

- Add focused tests for a large payload at and immediately above its Zstd
  compressed-size bound.
- Evaluate the sub-128 KiB margin lazily without changing the public artifact
  API or the accepted bound formula.
- Re-run this regression together with the complete App02/Runtime02/Runtime04/
  Runtime06 continuation batch; do not create a standalone Cargo job.

## Acceptance

- Both focused large-payload cases pass in the managed validation copy.
- The real App02 viewer reaches screenshot and 31-sample GPU timing evidence.
- The complete Main continuation batch passes before this repair is committed.
- The coordinator commit notification includes the resulting performance data;
  this correctness repair does not claim an independent speedup.
