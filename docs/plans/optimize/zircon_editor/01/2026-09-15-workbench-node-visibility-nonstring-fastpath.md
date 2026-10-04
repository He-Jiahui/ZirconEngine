---
title: Editor01 Workbench Node Visibility Non-String Fast Path
category: zircon_editor
report_id: Editor01-workbench-node-visibility-nonstring-fastpath-2026-09-15
date: 2026-09-15
session_id: root-runtime-editor-async-optimization-20260915
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor01 · Workbench 节点可见性非字符串快路径

## Scope

The Workbench projection checks a node's `visibility` property against a fixed
set of string spellings. Integer, float, and boolean property values were first
formatted into temporary strings even though none can match that product
vocabulary. This slice removes those scalar formatting allocations while
leaving string normalization and structural-value behavior unchanged.

## Implementation

- Keep string values on the existing underscore-insensitive ASCII comparison.
- Return `false` directly for integer, float, and boolean visibility values;
  these are not valid spellings for the fixed visibility property.
- Retain the existing `Datetime`, array, and table fallback (`expected ==
  "visible"`) and all caller-side `visible`/`collapsed` semantics.

## Regression and performance contract

The lower module
`zircon_editor/src/ui/retained_host/ui/workbench_window_projection/node_index/visibility_nonstring_tests.rs`
compares every scalar/structural branch with the legacy formatter and guards the
production source shape. Its ignored paired benchmark emits
`EDITOR789_WORKBENCH_NODE_VISIBILITY_NON_STRING_BENCH_V1`; the Python source
contract is `tools/tests/test_editor_workbench_node_visibility_nonstring_performance_contract.py`.

Current source fingerprints:

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/retained_host/ui/workbench_window_projection/node_index.rs` | `500907098A5041D7D658D487DAE8F875285EF32B0D74373F2FC1E1E8B2EA7BC5` |
| `zircon_editor/src/ui/retained_host/ui/workbench_window_projection/node_index/visibility_nonstring_tests.rs` | `DA37D09B9F4778A5E585850EC48B02E7234F748E74944299EE9C9877BBFC3991` |
| `tools/tests/test_editor_workbench_node_visibility_nonstring_performance_contract.py` | `F8FCD11006606D9B7A4F7F8575B77F60103AF19D2BE1747140460F757FE85C46` |

The deterministic scalar model removes one temporary scalar-string format per
non-string visibility probe (65,536 probes in the lower benchmark). This is
allocation-shape evidence, not a CPU, allocator, RSS, or product p50/p95/p99
measurement.

## Local receipt

The focused TDD source contract passes `3/3`; the lower semantic regression and
ignored Release marker are wired and Rustfmt-clean. The refreshed single-process
Runtime/Editor source-contract batch loads `552` modules and passes `1975/1975`
tests in `4.790s`, with zero failures, errors, or skips. This is local
source/model evidence only.
The broader non-tooling Runtime/Editor Python regression discovery passes
`3724/3724` across `915` modules in `326.952s`, with zero failures, errors, or
skips.

## Validation boundary

Managed Windows Cargo/Release execution, allocator counts, and Workbench product
percentile evidence remain coordinator-owned and pending. No per-task Cargo run,
coordinator retry, or status query is made for this record. Tooling production
work remains deferred for the later Rust migration.
