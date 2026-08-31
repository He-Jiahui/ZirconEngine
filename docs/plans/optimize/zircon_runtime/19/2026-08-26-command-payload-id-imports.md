# Zircon Runtime19 command payload ID import repair

## Scope

- Owner report: `docs/plans/optimize/zircon_runtime/19-woc-command-protocol-payload-codec-admission-movement-outcome-runtime-review.md`
- Related finding: `PROTOCOL-P1-019`
- Baseline: `38c0e7f5d48189ac2637ed010e452b19c32f459d`, epoch `437`
- Session: `root-runtime19-command-payload-binary-lookup-20260826`
- Production: `examples/woc/native/crates/woc_protocol/src/command_payload.rs`
- Production: `examples/woc/native/crates/woc_protocol/src/market_payload.rs`
- Structural regression: `tools/tests/test_runtime19_command_payload_import_contract.py`

## Problem

The generated command catalog already exported `ENTER_DELVE_COMMAND_ID` and
`MARKET_SEARCH_COMMAND_ID`, and both payload codecs referenced those names as bare crate-root
imports. Their grouped `use crate::{...}` declarations omitted the corresponding constants, so
`woc_protocol` could not resolve the names before any behavior or performance test executed.

This was a lower-layer compilation defect, not an App03/App04/App05 behavior regression. It also
prevented Runtime19's generated descriptor lookup benchmark from reaching its acceptance gate.

## Change

- Import `ENTER_DELVE_COMMAND_ID` in the shared command payload codec.
- Import `MARKET_SEARCH_COMMAND_ID` in the market payload codec.
- Add a generic source contract that removes grouped crate imports, discovers every remaining bare
  `_COMMAND_ID` reference, and requires the referenced identifier to be imported.
- Keep the generated catalog, command IDs, codec bytes, validation rules, and schema fingerprint
  unchanged.

## TDD and static evidence

- RED: `python -m unittest tools.tests.test_runtime19_command_payload_import_contract -v`
  reported exactly two missing identifiers; the generated-authority assertion already passed.
- GREEN: the same command passes `3/3` after the two imports are restored.
- Targeted `rustfmt --edition 2021 --check` passes for both production files.
- `git diff --check` passes for the repair paths apart from Git's existing LF/CRLF checkout notice.
- Existing `command_payloads` integration coverage contains 50 tests, including enter-delve and
  market-search encode/decode, raw-identity preservation, invalid finite values, bounds, and
  trailing-byte rejection.

## Performance relationship

The import repair deliberately changes no runtime instruction path. Its performance value is that
the Runtime19 binary descriptor lookup can compile and reach the existing release gate. The local
optimized lookup model remains:

| Metric | Linear lookup | Binary lookup | Change |
|---|---:|---:|---:|
| P50 | 9,193,600 ns | 1,480,800 ns | -83.893% |
| P95 | 13,554,600 ns | 2,859,700 ns | -78.902% |

No standalone speedup is attributed to adding imports.

## Async validation

Runtime19 r2 is a superset batch, not an isolated two-line compile ticket. It freezes the original
generator, generated lookup, five-test performance contract, and lookup record together with these
two production imports, the three-test import contract, and this repair record. One coordinator
ticket contains:

1. all eight Python source contracts;
2. generator syntax and authority checks;
3. formatting and candidate diff checks;
4. the complete 50-test `command_payloads` integration target;
5. full-u16 generated lookup equivalence;
6. the ignored release benchmark plus independent P50/P95 recomputation.

The r2 ticket supersedes r1 for integration. Commit and automatic WeCom finalization must quote the
managed binary-lookup performance row and identify this repair as the lower-layer test unblocker.
