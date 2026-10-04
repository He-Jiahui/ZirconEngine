---
status: local_candidate
implementation_files:
  - tools/analysis/validation/validate_performance_comparison_receipt.py
  - tools/tests/test_performance_comparison_receipt_single_report_read_performance_contract.py
---

# Performance comparison report single-read snapshot

## Problem

The performance comparison validator read each baseline and candidate report three
times: once for JSON parsing, once for the signed report binding, and once for the
distinct-report guard. Large render measurement reports therefore paid redundant
filesystem and payload-copy cost during every comparison validation.

## Change

`_load_json_report` now reads each report as bytes once, parses JSON from that
payload, and computes its SHA-256 digest from the same payload. The immutable
document/digest snapshot is reused by both the report binding and distinct-report
checks. Receipt, report, artifact, signature, and acceptance semantics are unchanged.

The static performance contract prevents report reads from returning to the
binding and distinct-report helpers.

## Validation

```text
python -m unittest tools.tests.test_performance_comparison_receipt_single_report_read_performance_contract tools.tests.test_validate_performance_comparison_receipt
18/18 passed in 6.042s
```

## Performance

Windows local synthetic comparison with two 8 MiB JSON reports, one warm-up and
31 measured iterations. The baseline reproduces the previous parse + binding hash
+ distinct hash reads; the candidate uses one payload snapshot per report.

| Metric | Baseline | Candidate | Improvement |
| --- | ---: | ---: | ---: |
| Reads per report | 3 | 1 | -66.7% |
| Bytes read per validation | 50,331,489 | 16,777,163 | -66.7% |
| p50 | 98,851,200 ns | 62,946,800 ns | -36.322% (1.57x) |
| p95 | 169,957,100 ns | 101,152,400 ns | -40.484% (1.68x) |
