---
status: local_candidate
implementation_files:
  - tools/audits/runtime_domain_dependency_audit.py
  - tools/tests/test_runtime_domain_dependency_line_index_performance_contract.py
  - tools/tests/test_runtime_domain_dependency_audit.py
---

# Runtime domain dependency line index

## Problem

The runtime domain audit converted every parsed `use` leaf and grouped crate
reference to a line number by rescanning the source prefix with
`str.count("\\n", 0, offset)`. Large files with many imports therefore paid a
repeated O(file-size) scan for each reference.

## Change

The parser now builds one newline-offset index per source view and resolves
line numbers with `bisect_right`. Both Rust use-tree parsers accept and reuse
the same index; direct callers retain the existing API because the index is
constructed lazily when omitted. Report ordering, line numbers, and masking
semantics are unchanged.

## Validation

```text
python -m unittest -v tools.tests.test_runtime_domain_dependency_line_index_performance_contract tools.tests.test_runtime_domain_dependency_audit
14/14 passed in 0.535s
```

The real repository audit completed successfully. A 10,000-line synthetic
source with 4,096 use leaves produced equivalent parser results.

## Performance

Windows local benchmark, 31 samples after two warm-ups:

| Metric | Baseline | Candidate | Improvement |
| --- | ---: | ---: | ---: |
| p50 | 419,804,500 ns | 48,698,000 ns | -88.400% (8.62x) |
| p95 | 677,496,800 ns | 60,257,100 ns | -91.106% (11.24x) |
| line lookup work | O(leaves x source bytes) | O(source bytes + leaves log lines) | asymptotic reduction |
