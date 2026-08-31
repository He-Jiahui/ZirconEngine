---
title: Tooling28 streamed front matter audit
category: zircon_tooling
report_id: Tooling28
date: 2026-08-31
baseline_head: 14c89f9776bed828cc85e05e4b9914b3f8d1e784
baseline_epoch: 575
status: release_validation_submitted
session_id: root-tooling28-streamed-front-matter-release-r2-20260831
implementation_files:
  - tools/check_conventions.py
tests:
  - tools/tests/test_tooling28_streamed_front_matter_performance_contract.py
  - tools/tests/test_check_conventions.py
---

# Tooling28 streamed front matter audit

## Problem

`audit_document_paths` only consumes the leading YAML front matter from each Markdown file, but `_front_matter_path_fields` used `read_text().splitlines()`. The required documentation gate therefore materialized every document body before discarding it, including large plans and evidence records.

## Change

The parser now opens each document with the existing `utf-8-sig` decoding contract, rejects ordinary Markdown after the first line, and returns immediately at the closing front matter delimiter. Field selection, quoting normalization, malformed-front-matter behavior, and report output remain unchanged.

The performance contract supplies a guarded stream that rejects both whole-file `read_text()` and any read after the closing delimiter. It also proves that a document without front matter consumes only its first line.

## Performance evidence

Acceptance thresholds were at least 80% fewer input bytes and at least 10% end-to-end or 15% parser P50 latency reduction on the live documentation tree.

| Measurement | Legacy | Optimized | Reduction |
|---|---:|---:|---:|
| Documentation bytes eligible for parsing | 101,973,801 | 14,744,263 upper bound | 85.54% |
| Five-round parser P50 | 4,662,950,800 ns | 3,666,267,500 ns | 21.37% |
| Five-round parser P95 | 6,168,389,600 ns | 4,763,775,800 ns | 22.77% |
| Full `--only docs --json` wall time | 23,854 ms | 20,414 ms | 14.42% |

The five-round model scanned 4,832 Markdown files, found 3,910 front matter documents in both implementations, alternated legacy/optimized execution order, and produced checksum `3910`. Samples in nanoseconds were:

- legacy: `4662950800, 6168389600, 4214651100, 4277840200, 5184959500`
- optimized: `3416807700, 4763775800, 3954028500, 3666267500, 3619185000`

The shared worktree added five Markdown files between the initial and final measurements. Final byte accounting and the five-round model both use the later 4,832-file corpus frozen immediately before this record was created.

### Repeatable actual-parser acceptance

The managed acceptance contract now invokes the production `_front_matter_path_fields` against the same 9,961,608-byte UTF-8 document as a local legacy `read_text().splitlines()` oracle. The six-line front matter is followed by 262,144 body lines. Both parsers produce checksum `83`; the production parser semantically consumes only the 130-byte front matter. Four alternating warmup pairs precede 21 alternating sample pairs, and the marker emits both raw arrays plus nearest-rank P50/P95.

The pre-submission Windows run produced the following non-terminal evidence:

| Measurement | Legacy | Optimized | Reduction |
|---|---:|---:|---:|
| Semantically consumed bytes | 9,961,608 | 130 | 99.9987% |
| 21-pair P50 | 158,005,900 ns | 464,200 ns | 99.7062% |
| 21-pair P95 | 664,457,300 ns | 2,286,900 ns | 99.6558% |

The acceptance gate requires at least 80% fewer consumed bytes and at least 85% lower P50 and P95. The coordinator run remains authoritative because the local preflight shares the active workstation with unrelated processes.

## Validation

- Red phase: both new performance-contract tests failed on the legacy `read_text()` call.
- Focused batch after the change: the two new tests and all document-path audit tests passed.
- Existing combined suite: 31 of 32 tests passed. The unrelated `test_rust_exemption_audit_accepts_current_enforced_members` baseline currently reports two foreign working-tree violations in `zircon_runtime_interface/src/project/session_lock/record.rs` and the untracked `zircon_runtime_interface/src/runtime_build_set/slot_catalog_build_tests.rs`; this change does not edit those paths.
- Full documentation audit preserves its expected nonzero exit because the repository already contains documented path findings; its measured wall time decreased as shown above.
- Current focused preflight: all three Tooling28 tests passed, including the actual-parser latency and byte gate.
- Managed request `6acc21366277491a93f10aab353098db` runs the three-test Tooling28 batch asynchronously; terminal coordinator evidence and integration remain pending.
