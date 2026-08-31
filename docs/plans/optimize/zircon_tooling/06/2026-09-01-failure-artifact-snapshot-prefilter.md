# Tooling06 Failure artifact snapshot prefilter

## Problem

`failure_artifact_snapshot` recursively enumerates Markdown under `docs/plans`, but
only files with controlled failure/fixed handoff names belong in its output. The
legacy comprehension called `Path.is_file()` for every Markdown before evaluating
the filename predicate, so ordinary plans and records generated unnecessary metadata
IO on every immutable-action snapshot.

## Change

The pure filename predicate now runs before `is_file()` and `resolve()`. Only
failure-artifact candidates touch file metadata or canonicalization. Candidate
SHA-256 uses `hashlib.file_digest` rather than materializing the full artifact.
Snapshot sorting, containment checks, digest values, and output schema remain
unchanged.

The performance contract creates 2,048 ordinary plans and two failure artifacts.
It proves only the two candidates call `is_file()` and both snapshot rows remain.

## Validation

```powershell
python -m unittest -q tools.tests.test_tooling06_failure_artifact_snapshot_prefilter_performance_contract tools.session_coordinator.tests.test_failures
```

Result: 36/36 tests passed. A wider failure closeout/return/milestone batch exceeded
the 120-second local budget and is explicitly delegated to the asynchronous combined
Coordinator validation queue.

An alternating 21-pair benchmark used one physical tree with 4,096 ordinary plans
and four failure artifacts:

- p50: `112,829,800 ns -> 6,100,500 ns` (`94.593%` reduction, `18.50x` speedup).
- p95: `198,749,000 ns -> 44,622,200 ns` (`77.548%` reduction, `4.45x` speedup).
- File metadata calls: `4,100 -> 4` (`99.902%` reduction).
- Both paths retained checksum `4`.

A separate alternating 15-pair SHA-256 benchmark used one 32 MiB artifact:

- Hash p50: `46,260,100 ns -> 44,450,300 ns` (`3.912%` reduction).
- Hash p95: `83,480,400 ns -> 77,937,100 ns` (`6.640%` reduction).
- Python peak allocation: `33,554,818 -> 394,417 bytes` (`98.825%` reduction).
- Both paths produced digest prefix `05f052c8f6da8ee5`.

## Acceptance

Accepted locally for the snapshot owner: all focused failure graph tests pass,
metadata calls are bounded by filename candidates, scan p50 improves by more than
94%, scan p95 improves by more than 77%, and large-file hashing reduces peak Python
allocation by more than 98%. Final acceptance requires the asynchronous combined
Coordinator failure-workflow gate before commit and push.
