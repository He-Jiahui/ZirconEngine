# RuntimeInterface03 Renderer Parity Stats Fusion

## Status

`implementation_complete; managed_validation_pending`

## Scope

- Owner Session: `root-runtime-interface03-activate-link-failure-20260831`
- Parent plan: `03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md`
- Finding: renderer parity snapshot construction scanned every completed paint
  row again for clip, resource, and text statistics.
- Boundary: this is diagnostic snapshot cost, not a product GPU-frame speedup.

## Change

Clip, resource, and text counters are now accumulated while each parity paint
row is projected. The two post-projection full scans are removed; public stats,
paint rows, batch rows, and serialized shape remain unchanged.

## Performance Contract

- Stats passes over parity rows: `3 -> 1` fused projection pass contribution.
- Additional allocations: `0 -> 0`.
- Benchmark fixture: 65,536 mixed clip/resource/text flags and 11 alternating
  separate/fused samples.
- Acceptance threshold: fused P95 must be at least 20% lower than separate-scan
  P95.
- Exact P50/P95: pending managed Windows release benchmark output.

## Verification

- TDD RED: static contract failed while the post-projection scans and no
  benchmark were present.
- Focused static contract after implementation: `3/3` passed; combined parity
  performance contracts: `9/9` passed.
- Python compile check and `git diff --check`: passed.
- Managed `cargo +1.94.1 test -p zircon_runtime_interface --locked --release
  --jobs 1`: pending asynchronous batch submission.
- Managed ignored release benchmark: pending the same batch.

No commit, push, or WeCom notification is permitted until managed validation is
terminal-successful and the coordinator finalizes the attributed union.
