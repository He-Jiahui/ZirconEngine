---
title: Tooling29 framework contract single source scan
category: zircon_tooling
report_id: Tooling29
date: 2026-09-01
status: local_candidate
implementation_files:
  - tools/audits/framework_contract_partition_audit.py
tests:
  - tools/tests/test_tooling29_framework_contract_single_source_scan_performance_contract.py
  - tools/tests/test_framework_contract_partition_audit.py
---

# Tooling29 framework contract single source scan

## Problem

`_framework_source_inventory` already enumerated every Rust source beneath the
framework root and retained the complete `set[Path]`. `audit_framework_partition`
discarded that path snapshot, then traversed the same directory tree again before
classifying the files. The second traversal added filesystem metadata work without
adding evidence or observing a deliberately newer snapshot.

## Change

The inventory now returns its `source_paths` set with the existing test-only path,
source-text, and code-view projections. The audit sorts and iterates that same set.
Production/test filtering, deterministic path ordering, source contents,
classification, and the JSON report shape are unchanged.

The static performance contract requires exactly one `rglob` in the inventory and
none in `audit_framework_partition`, preventing the redundant traversal from
returning.

## Performance evidence

The Windows benchmark created 10,000 Rust files across 100 directories, warmed both
paths, and measured 31 alternating pairs. Both implementations returned the same
20,000 aggregate path count.

| Measurement | Duplicate traversal | Reused snapshot | Improvement |
|---|---:|---:|---:|
| P50 | 518,266,500 ns | 397,682,400 ns | 23.267% (1.30x) |
| P95 | 910,357,000 ns | 775,410,100 ns | 14.824% (1.17x) |
| Framework tree enumerations | 2 | 1 | 50.000% |

## Validation

- RED: the new contract failed because `audit_framework_partition` contained one
  additional `rglob("*.rs")` call.
- GREEN: the new performance contract and existing framework audit module passed
  together, 19/19 in 0.395 seconds.
- The real repository audit completed successfully and reported 807 production
  framework files under `zircon_runtime/src/core/framework`.
- `git diff --check` passed for the implementation and contract.
- This remains a local candidate for the accumulated asynchronous validation lane;
  no coordinator status polling was performed.
