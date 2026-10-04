# Milestone manifest streaming file hash

## Change

`MilestoneWorkflowService._manifest_hash_at` now hashes file contents through
`hashlib.file_digest` instead of `Path.read_bytes()`. Manifest JSON shape,
ordering, SHA-256 values, deletion entries, and directory rejection remain
unchanged.

## Performance evidence

Windows local benchmark, 32 MiB file, 15 rounds:

| Metric | Before | After | Improvement |
| --- | ---: | ---: | ---: |
| Peak Python allocation p50 | 33,555,153 bytes | 395,177 bytes | 98.822% lower, 84.91x |
| p50 | 50,789,300 ns | 49,537,200 ns | 2.465% lower |
| p95 | 101,629,700 ns | 87,753,500 ns | 13.654% lower |

Both paths produced the same digest prefix `e09320c5b00b34bb`.

## Validation

- Streaming contract plus workflow-attempt tests: 4/4 passed in 10.743 seconds.
- Two broader milestone batches exceeded the 120-second local command budget.
  They were terminated and are neither green nor failed; the Coordinator must
  run them in the longer combined validation lane.
