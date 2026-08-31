# Tooling06 Legacy Cargo diagnostic prefilter

## Problem

Legacy Cargo diagnostics recursively enumerate `.codex`, but only filenames that
encode a Cargo lease or target slot can become diagnostics. The legacy loop called
`Path.is_file()` and projected a repository-relative path before applying that pure
name predicate. Ordinary state, logs, and metadata therefore incurred unnecessary
filesystem and path work during every rollout audit.

## Change

The filename predicate now runs immediately after discovery. Only matching
candidates call `is_file()` and `_relative()`. Legacy note SHA-256 uses
`hashlib.file_digest` instead of materializing the whole note. Existing exclusions for
`.codex/sessions` and `.codex/targets`, diagnostic sorting, and target-lane reporting
remain unchanged.

The performance contract creates 2,048 ordinary records, two reportable candidates,
and one candidate under the excluded sessions root. It proves only those three
candidate names call `is_file()` while output retains only the two reportable paths.

## Validation

```powershell
python -m unittest -q tools.tests.test_tooling06_legacy_cargo_diagnostic_prefilter_performance_contract tools.session_coordinator.tests.test_rollout_audit
python -m unittest -q tools.session_coordinator.tests.test_legacy_migration tools.tests.test_tooling06_legacy_cargo_diagnostic_prefilter_performance_contract
```

Result: 16/16 tests passed across the two batches. An earlier command referenced a
nonexistent `test_legacy_archive` module and was discarded; the actual archive and
recovery owner is `test_legacy_migration` and passed.

An alternating 21-pair benchmark used one physical `.codex` tree with 4,096
ordinary records and four legacy candidates:

- p50: `85,446,100 ns -> 4,613,300 ns` (`94.601%` reduction, `18.52x` speedup).
- p95: `128,947,200 ns -> 10,963,300 ns` (`91.498%` reduction, `11.76x` speedup).
- File metadata calls: `4,101 -> 4` (`99.902%` reduction).
- Both paths retained checksum `4`.

A separate alternating 15-pair SHA-256 benchmark used one 32 MiB note:

- Hash p50: `46,260,100 ns -> 44,450,300 ns` (`3.912%` reduction).
- Hash p95: `83,480,400 ns -> 77,937,100 ns` (`6.640%` reduction).
- Python peak allocation: `33,554,818 -> 394,417 bytes` (`98.825%` reduction).
- Both paths produced digest prefix `05f052c8f6da8ee5`.

## Acceptance

Accepted locally: rollout audit and legacy migration suites pass, metadata calls
are bounded by filename candidates, scan p50/p95 improve by more than 91%, and
large-file hashing reduces peak Python allocation by more than 98%. Final acceptance
requires the asynchronous combined Coordinator gate before commit and push.
