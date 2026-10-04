---
title: Tooling06 milestone manifest single metadata probe
category: zircon_tooling
report_id: Tooling06
date: 2026-09-01
status: local_candidate
implementation_files:
---

# Tooling06 milestone manifest single metadata probe

## Change

`MilestoneWorkflowService._manifest_hash_at` now classifies each manifest path
from one `stat()` result rather than calling `is_file()` and then `is_dir()`.
Files, directory rejection, deletion entries, and special-file fallback retain
their previous semantics.

## Performance evidence

Windows local benchmark, 256 deletion paths, 25 rounds of 20 manifests:

| Metric | Before | After | Improvement |
| --- | ---: | ---: | ---: |
| p50 | 393,809,000 ns | 313,363,400 ns | 20.428% lower, 1.26x |
| p95 | 540,832,200 ns | 449,522,200 ns | 16.883% lower, 1.20x |
| Metadata probes per deletion | 2 | 1 | 50.000% lower |

## Validation

The expanded milestone contract and workflow-attempt batch passed 7/7 in 23.184
seconds. The existing longer milestone lanes remain delegated to the Coordinator.
