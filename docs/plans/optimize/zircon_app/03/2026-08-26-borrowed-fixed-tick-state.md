# Zircon App03 borrowed fixed-tick committed state

## Scope

- Owner report: `docs/plans/optimize/zircon_app/03-woc-product-role-host-zrvm-transaction-state-client-server-integration-review.md`
- Finding: `WOC-APP-P1-024`, first acceptance slice
- Baseline: `38c0e7f5d48189ac2637ed010e452b19c32f459d`, epoch `438`
- Session: `root-zircon-app03-borrowed-tick-state-20260826`
- Protocol contract: `examples/woc/native/crates/woc_protocol/src/contracts.rs`
- Protocol codec: `examples/woc/native/crates/woc_protocol/src/payload.rs`
- Runtime hot path: `examples/woc/native/plugins/woc_runtime/src/transaction.rs`
- Structural contract: `tools/tests/test_woc_runtime_borrowed_tick_state_performance_contract.py`

## Problem

`WocTransactionalRuntime::prepare_tick` cloned the complete committed state into an owned
`FixedTickInput`, after which `FixedTickInput::encode_payload` copied those bytes again into the
wire buffer. A successful tick therefore performed two full state copies before the VM could read
the payload. The same construction also cloned the optional first-tick bootstrap.

This is the narrow host-side copy identified by `WOC-APP-P1-024`. The wider snapshot-handle,
paged-view, command-delta, and qualified base-generation protocol remains future work; this slice
does not claim that the complete finding is closed.

## Change

- Add public `FixedTickInputRef<'a>` with borrowed command, state, movement, and bootstrap fields.
- Make the existing owned `FixedTickInput::encode_payload` delegate to the borrowed encoder, so
  both entry points share one validation and wire-format implementation.
- Build the borrowed view directly in `prepare_tick`; the committed state and bootstrap are no
  longer cloned before encoding.
- Preserve command validation, movement canonicalization, all size bounds, byte ordering, and the
  owned decoder. The VM trait and payload ABI are unchanged.
- Preserve the existing movement-vector canonicalization copy. Removing that copy requires a
  separate proof that every direct protocol caller supplies a canonical batch.

## TDD and static evidence

- RED: `python -m unittest tools.tests.test_woc_runtime_borrowed_tick_state_performance_contract -v`
  failed `5/5` contracts against the owned input construction and missing borrowed encoder.
- GREEN: the same command passes `5/5` after the implementation.
- The protocol regression compares owned and borrowed encoders byte-for-byte before decoding the
  payload back to the original owned input.
- The runtime regression performs a second tick and proves the VM still observes the first tick's
  committed state bytes.
- Targeted `rustfmt --edition 2021 --check` passes for all six owned Rust files.
- `git diff --check` passes for the candidate paths apart from Git's existing LF/CRLF checkout
  notice.

## Local release-model evidence

The independent Rust `-O` model uses a 16 MiB committed state, eight encodings per sample, 21
alternating owned/borrowed sample pairs, and nearest-rank percentiles. Both paths allocate and fill
the same output wire buffer; only the owned path first clones the state into its input DTO.

| Metric | Owned input | Borrowed input | Change |
|---|---:|---:|---:|
| P50 | 207,005,400 ns | 100,949,100 ns | -51.234% |
| P95 | 793,926,000 ns | 243,929,800 ns | -69.275% |
| pre-encode full-state copies | 1 | 0 | -100.000% |

The formal in-crate release benchmark uses the actual protocol encoders, emits both raw 21-sample
arrays in one machine-readable row, and requires the validator to recompute nearest-rank P50/P95.
Acceptance requires at least 35% improvement for both distributions.

## Async validation

No Cargo command is run directly in the shared checkout. One coordinator batch contains:

1. the five Python source contracts;
2. formatting and candidate diff checks;
3. the complete `woc_protocol` protocol integration target;
4. the complete `woc_runtime` transaction integration target;
5. the ignored release benchmark with `--nocapture`;
6. external parsing of the raw arrays plus P50/P95 gate enforcement.

The candidate remains pending until the coordinator reports all managed Cargo groups green. Any
lower-layer WOC compile failure must be reported at its owning protocol path instead of being
misclassified as an App03 behavior regression. Commit and automatic WeCom finalization must quote
the managed benchmark row rather than the standalone model.
