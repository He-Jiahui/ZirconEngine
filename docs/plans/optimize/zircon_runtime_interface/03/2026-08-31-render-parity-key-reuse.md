# RuntimeInterface03 Renderer Parity Key Reuse

## Status

`implementation_complete; managed_validation_pending`

## Scope

- Owner Session: `root-runtime-interface03-activate-link-failure-20260831`
- Parent plan: `03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md`
- Finding: parity projection rebuilt each paint element's complete batch key a
  second time only to read resource and text backend metadata.

## Change

Each parity paint row now constructs `UiBatchKey` once and reuses its resource
and text-backend fields. The duplicated private key projection helpers were
removed. Resource fallback chains, draw-effect vectors, public parity fields,
and serialized output remain unchanged.

## Performance Contract

- Full batch-key projections per paint row: `2 -> 1`.
- Repeated draw-effect vector clones per row: `2 -> 1`.
- Benchmark fixture: 4,096 image paint elements, 24 draw effects, a two-level
  fallback resource chain, and 11 alternating samples.
- Acceptance threshold: reused-key P95 must be at least 20% lower than repeated
  projection P95.
- Exact P50/P95: pending managed Windows release benchmark output.

## Verification

- TDD RED: static contract failed before key metadata reuse and the benchmark
  existed.
- Focused static contract after implementation: `3/3` passed.
- Python compile check and `git diff --check`: passed.
- Managed `cargo +1.94.1 test -p zircon_runtime_interface --locked --release
  --jobs 1`: pending asynchronous batch submission.
- Managed ignored release benchmark: pending the same batch.

No commit, push, or WeCom notification is permitted until managed validation is
terminal-successful and the coordinator finalizes the attributed union.
