# Runtime19 command-value owned object keys

## Scope

- Owner report: `docs/plans/optimize/zircon_runtime/19-woc-command-protocol-payload-codec-admission-movement-outcome-runtime-review.md`
- Findings: `PROTOCOL-P1-024`, `PROTOCOL-P1-060`
- Baseline: `681588f7a1cbfaae3147e8b93e1be6705d810f21`, epoch `516`
- Session: `root-runtime19-two-task-performance-batch-r2-20260831`
- Production: `examples/woc/native/crates/woc_protocol/src/command_value.rs`
- Behavior regression: `examples/woc/native/crates/woc_protocol/tests/command_value.rs`
- Structural contract: `tools/tests/test_runtime19_command_value_object_key_performance_contract.py`

## Problem

Both `CommandValue::object` and the object branch of `Reader::read_value` received an owned
`String` key, cloned it unconditionally, and inserted the clone into a `BTreeMap`. The original key
was retained only so a duplicate-key error could own its diagnostic text. Every unique key on the
normal path therefore performed an avoidable heap allocation and byte copy. The configured object
limit permits 4,096 entries, so one maximum legal value could trigger 4,096 redundant string
allocations in either construction path.

## Change

- Use `BTreeMap::entry(key)` in the public object constructor and decoder object branch.
- Move each unique owned key directly into its vacant entry.
- Clone `occupied.key()` only on the rare duplicate-key error path required by the existing owned
  `ProtocolError::DuplicateCommandObjectKey` contract.
- Keep canonical `BTreeMap` ordering, encoded bytes, recursive decode order, size/depth limits, and
  error type unchanged.
- Extend the integration regression so direct object construction and wire decode both preserve the
  exact duplicate key diagnostic.

The change remains inside the existing command-value codec owner and does not add a second registry
or parsing authority.

## TDD and local evidence

- RED: the four-test source contract failed because the prior implementation imported no entry API
  and called `values.insert(key.clone(), value)` in both normal paths.
- GREEN: direct pure-Python execution passes `6/6` source contracts, including guards for the
  release benchmark's real production API, nearest-rank percentile, and allocation/latency gates.
- `rustfmt +1.94.1 --edition 2021 --check` passes for production and integration-test Rust files.
- Candidate `git diff --check` passes apart from the checkout's existing LF/CRLF notice.
- No Cargo command was run directly in the shared checkout.

## Local release evidence

The independent release model uses seven consecutive maximum legal 4,096-key objects per timed
sample, 31 alternating sample pairs, and five warmups. Input cloning stays outside the timed and
allocation-counted region. Both implementations produce the same sorted-map checksum.

| Metric | Clone then insert | Owned entry | Change |
|---|---:|---:|---:|
| Allocations | 33,439 | 4,767 | -85.744% |
| Allocated bytes | 3,166,352 | 1,818,768 | -42.560% |
| P50 | 23,375,500 ns | 18,121,700 ns | -22.476% |
| P95 | 48,798,400 ns | 37,456,100 ns | -23.243% |

The checksum is `1194180288289350949` for both paths. A preceding identical expanded-window run
reported P50 `-23.554%` and P95 `-41.409%`; the table deliberately retains the more conservative
repeat. The earlier single-object P95 sample was rejected as scheduler-noisy before recording any
acceptance claim. This historical standalone model has now been moved into the real
`command_value` integration binary: its optimized side calls `CommandValue::object`, emits the
structured marker `RUNTIME19_COMMAND_VALUE_OBJECT_KEYS_BENCH_V1`, and enforces at least 80% fewer
allocations, 40% fewer allocated bytes, 15% lower P50, and 10% lower P95.

## Async validation

This candidate shares one Runtime19 coordinator ticket with movement-sequence admission. The
managed Windows Rust 1.94.1 command is `cargo test --manifest-path
examples/woc/native/Cargo.toml -p woc_protocol --test command_value --test movement_input --locked
--release -- --include-ignored --nocapture --test-threads=1`, so ordinary behavior regressions and
both release-only performance gates run from the same immutable source manifest. The Session
continues other useful work before one-shot reconciliation. Commit and automatic WeCom finalization
remain pending until managed validation is green and an independent reviewer is available.
