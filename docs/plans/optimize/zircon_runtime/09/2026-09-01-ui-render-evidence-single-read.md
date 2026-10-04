---
status: local_candidate
implementation_files:
  - tools/analysis/performance/ui/ui_render_segment_evidence.py
  - tools/analysis/performance/ui/ui_render_dependency_delta_evidence.py
  - tools/analysis/performance/ui/ui_render_dependency_memory_evidence.py
  - tools/tests/test_ui_render_evidence_single_read_performance_contract.py
---

# UI render evidence single-read snapshots

## Problem

The stable-segment, dependency-delta, and dependency-memory evidence tools read
their JSON evidence twice: first as UTF-8 JSON and then again through the
streaming SHA-256 helper. The memory tool repeated this work for timeline,
interaction evidence, and source manifest. Availability was also probed again
when constructing evidence bindings.

## Change

All three tools now load each JSON input once as bytes and retain an immutable
document/SHA-256 snapshot. Evaluation consumes the parsed document, while the
evidence binding reuses the digest. A missing source manifest still produces
the existing blocker; malformed input, output schema, and uppercase digest
semantics are unchanged.

The shared static performance contract covers all three evidence tools and prevents
timeline or manifest hashing from reopening the files.

## Validation

```text
python -m unittest tools.tests.test_ui_render_evidence_single_read_performance_contract tools.tests.test_ui_render_segment_evidence tools.tests.test_ui_render_dependency_delta_evidence
31/31 passed in 0.041s
```

## Performance

Windows local benchmark with one 8 MiB timeline and one 8 MiB source manifest,
two warm-ups and 41 alternating-order iterations. The baseline reproduces
`read_text` followed by streaming SHA-256; the candidate parses and hashes one
byte payload. p95 is the 39th ordered sample.

| Metric | Baseline | Candidate | Improvement |
| --- | ---: | ---: | ---: |
| Reads per input | 2 | 1 | -50.000% |
| Bytes read per run | 33,554,316 | 16,777,158 | -50.000% |
| p50 | 103,942,500 ns | 86,765,400 ns | -16.526% (1.20x) |
| p95 | 197,241,700 ns | 162,100,400 ns | -17.816% (1.22x) |

Dependency-memory evidence was measured separately because it binds three
inputs. With three 8 MiB JSON files, bytes per run fell `50,331,480 ->
25,165,740`; p50 improved `143,460,400 -> 98,057,000 ns` (-31.649%, 1.46x),
and p95 improved `182,829,000 -> 160,997,400 ns` (-11.941%, 1.14x).
