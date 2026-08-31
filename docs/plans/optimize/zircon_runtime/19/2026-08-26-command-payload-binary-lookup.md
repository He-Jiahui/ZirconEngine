# Runtime19 generated command payload binary lookup

## Scope

- Owner report: `docs/plans/optimize/zircon_runtime/19-woc-command-protocol-payload-codec-admission-movement-outcome-runtime-review.md`
- Finding: `PROTOCOL-P1-019`
- Baseline: `38c0e7f5d48189ac2637ed010e452b19c32f459d`, epoch `437`
- Session: `root-runtime19-command-payload-binary-lookup-20260826`
- Generator authority: `examples/woc/tools/command_payload_codegen.mjs`
- Generated Rust: `examples/woc/native/crates/woc_protocol/src/generated_command_payloads.rs`
- Structural contract: `tools/tests/test_runtime19_command_payload_descriptor_performance_contract.py`

## Problem

`command_payload_descriptor` linearly scanned all 157 generated descriptors for every lookup. The catalog is already generated in strictly increasing command-id order, so both successful and rejected command paths paid an avoidable O(n) scan. Direct compilation also exposed an existing generator drift: the catalog emitted `CommandPayloadKind::WeaponSkinChange`, while the generated enum omitted that variant.

## Change

- Generate `binary_search_by_key` against the existing sorted slice and return the borrowed descriptor without allocation or cloning.
- Keep a linear lookup only inside the test module as an independent equivalence and benchmark oracle.
- Exhaustively compare both implementations for every `u16` id, covering all catalog hits and sparse misses.
- Generate an ignored release benchmark with 100,000 mixed hit/miss queries, 21 alternating sample pairs, raw sample arrays, and a 65% P50/P95 gate.
- Restore the missing `WeaponSkinChange` enum variant and add a structural exhaustiveness guard that rejects any catalog kind not declared by the generated enum.
- Keep the generator as the sole authority; the schema fingerprint remains `33343bc135579f1` and the generated Zr payload file remains byte-identical.

The 2,000-line generator and 1,900-line Rust catalog remain single-purpose generated/table authorities. This change does not add a second responsibility, so splitting either file would weaken rather than improve generator ownership.

## TDD and local evidence

- RED: the source contract failed the production linear lookup in both generator and generated Rust; direct `rustc --test` then exposed the pre-existing missing `WeaponSkinChange` variant.
- GREEN: `python -m unittest tools.tests.test_runtime19_command_payload_descriptor_performance_contract` passes `5/5`.
- `node examples/woc/tools/command_payload_codegen.mjs --check` passes for all 157 descriptors.
- `rustfmt 1.8.0` targeted check and candidate `git diff --check` pass.
- Direct `rustc 1.94.1 --test -O` compiles the actual generated file, and the full `u16` equivalence regression passes `1/1`.

## Local release evidence

The actual generated Rust benchmark performs 100,000 deterministic mixed hit/miss queries per sample, alternates legacy-first order across 21 pairs, and reports nearest-rank percentiles.

| Metric | Linear scan | Binary lookup | Change |
|---|---:|---:|---:|
| P50 | 9,193,600 ns | 1,480,800 ns | -83.893% |
| P95 | 13,554,600 ns | 2,859,700 ns | -78.902% |

## Async validation

No Cargo build or test is run directly in the shared checkout. One coordinator batch contains:

1. all five Python source contracts;
2. Node syntax and generator consistency checks;
3. targeted Rust formatting and candidate diff checks;
4. the generated full-`u16` Cargo equivalence regression;
5. the ignored Cargo release benchmark with external parsing of both 21-sample arrays and enforcement of the 65% P50/P95 gate.

The candidate remains pending until the coordinator reports both managed Cargo groups green. Commit and automatic WeCom finalization must quote the managed benchmark row rather than the standalone local result.
