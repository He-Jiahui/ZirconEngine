# Runtime19 movement sequence admission

## Scope

- Owner report: `docs/plans/optimize/zircon_runtime/19-woc-command-protocol-payload-codec-admission-movement-outcome-runtime-review.md`
- Findings: `PROTOCOL-P0-005`, `PROTOCOL-P1-060`
- Baseline: `681588f7a1cbfaae3147e8b93e1be6705d810f21`, epoch `516`
- Session: `root-runtime19-two-task-performance-batch-r2-20260831`
- Production: `examples/woc/native/crates/woc_protocol/src/movement_input.rs`
- Behavior regression: `examples/woc/native/crates/woc_protocol/tests/movement_input.rs`
- Structural contract: `tools/tests/test_runtime19_movement_sequence_admission_performance_contract.py`

## Problem

`MovementInputRelay::apply_batch` accepted every shape-valid packet even when its sequence was equal
to or lower than the retained acknowledgement. A delayed or duplicated packet could therefore
replace newer flags, facing, and accepted tick while the acknowledgement alone remained monotonic.

The same hot loop also searched the retained-input `BTreeMap` up to three times per frame: once for
the acknowledgement, once for fallback facing, and once for insertion/replacement. At the maximum
65,536-frame batch bound this created avoidable tree traversal before locomotion consumed the
retained input.

## Change

- Classify each frame as `Applied`, `Duplicate`, or `Stale` by comparing its sequence against the
  retained acknowledgement.
- Preserve flags, facing, acknowledgement, and accepted tick for duplicate and stale frames.
- Preserve the prior facing when a strictly newer frame omits a facing update.
- Replace the repeated `get`/`get`/`insert` sequence with one `BTreeMap::entry` lookup and an in-place
  occupied-entry update.
- Keep batch ordering, tick-regression checks, stale-input clearing, actor keys, and public relay
  ownership unchanged.

The enum is re-exported without changing the module boundary. A physical Rust-source scan found no
consumer outside this module and its integration test, so the new dispositions do not leave an
unupdated exhaustive product match.

## TDD and local evidence

- RED: the four-test source contract failed because the prior implementation had no occupied-entry
  sequence classification.
- GREEN: direct pure-Python execution passes `6/6` source contracts, including guards for the
  release benchmark's real relay API, nearest-rank percentile, and probe/latency gates.
- The integration regression now covers initial apply, stale, duplicate, and newer sequences and
  asserts that rejected sequence classes cannot replace retained state.
- `rustfmt +1.94.1 --edition 2021 --check` passes for both Rust files.
- Candidate `git diff --check` passes apart from the checkout's existing LF/CRLF notice.
- No Cargo command was run directly in the shared checkout.

## Local release evidence

The independent release model uses 8,192 retained actors, 31 alternating sample pairs, five
warmups, and an equivalent all-newer workload so both implementations produce the same final
state. The gate requires at least 60% fewer logical tree probes and at least 40% lower P50 and P95
latency.

| Metric | Repeated lookup | Entry lookup | Change |
|---|---:|---:|---:|
| Tree probes | 24,576 | 8,192 | -66.667% |
| Allocations | 1 | 1 | unchanged |
| Allocated bytes | 32,768 | 32,768 | unchanged |
| P50 | 1,043,000 ns | 487,500 ns | -53.260% |
| P95 | 1,138,600 ns | 519,700 ns | -54.356% |

The final-state checksum is `9879330329816191350` for both paths. The historical standalone model
has now been moved into the real `movement_input` integration binary: its optimized side calls
`MovementInputRelay::apply_batch`, emits the structured marker
`RUNTIME19_MOVEMENT_SEQUENCE_ADMISSION_BENCH_V1`, and enforces at least 60% fewer logical tree
probes plus at least 40% lower P50 and P95.

## Async validation

This candidate shares one Runtime19 coordinator ticket with command-value owned object keys. The
managed Windows Rust 1.94.1 command is `cargo test --manifest-path
examples/woc/native/Cargo.toml -p woc_protocol --test command_value --test movement_input --locked
--release -- --include-ignored --nocapture --test-threads=1`, so ordinary behavior regressions and
both release-only performance gates run from the same immutable source manifest. The Session must
continue other useful work before reconciling the ticket once. Commit and automatic WeCom
finalization remain pending until that managed result is green and an independent reviewer is
available.
