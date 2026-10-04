---
title: Tooling06 milestone plan output single read
category: zircon_tooling
report_id: Tooling06
date: 2026-09-01
status: local_candidate
implementation_files:
tests:
  - tools/tests/test_tooling06_milestone_plan_output_single_read_performance_contract.py
---

# Tooling06 milestone plan output single read

## Change

`_valid_plan_output` previously read the same record once for headings and again
inside `_plan_output_fields`. It now parses fields from the already loaded text.
The path-based `_plan_output_fields` API remains available and delegates to the
same pure text parser.

## Performance evidence

Windows local benchmark, 256 KiB record, 25 rounds of 100 validations:

| Metric | Before | After | Improvement |
| --- | ---: | ---: | ---: |
| p50 | 602,795,800 ns | 288,773,200 ns | 52.094% lower, 2.09x |
| p95 | 845,493,600 ns | 664,970,100 ns | 21.351% lower, 1.27x |
| File reads per validation | 2 | 1 | 50.000% lower |
| Bytes read per validation | 524,472 | 262,236 | 50.000% lower |

## Validation

Milestone manifest, plan-output, and workflow-attempt coverage passed 9/9 in
6.725 seconds. Wider milestone suites remain in the Coordinator long lane.

The corrected local submission scope for Tooling06, Tooling29, and Editor01 was
also validated as one batch: 29/29 tests passed in 0.656 seconds. All related
Python modules passed `py_compile`, and `git diff --check` reported only expected
line-ending normalization warnings. Coordinator validation remains asynchronous;
this task did not poll its status.
