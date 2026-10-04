---
title: Runtime Export Plugin List Direct
category: zircon_runtime
report_id: Runtime878-export-plugin-list-direct-2026-09-21
date: 2026-09-21
session_id: root-runtime-editor-async-optimization-20260921
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Runtime878 Export Plugin List Direct

## Finding and optimization

The generated export bootstrap used to format each selected plugin name into
its own quoted/escaped source expression, collect all expressions into a
temporary `Vec<String>`, and join it into a second full string. The
`string_vec_expr` helper now appends each debug-formatted name, literal
`.to_string()`, and positional delimiter directly to the required final
output with a conservative capacity bound. Rust `Debug` escaping can expand
beyond that bound, so the change eliminates child strings/vector slots but
does not claim exactly one final-buffer allocation. The generated Rust
syntax, empty/singleton/multi-plugin order, escapes, and external export
contracts are unchanged.

## TDD and deterministic evidence

The combined Editor897/Runtime878 contracts were RED `2/7` with three missing
lower-test/module errors, then GREEN `7/7`. Lower tests compare the retired
template for empty and Unicode strings, embedded NUL, quotes, and newline
escaping. A 4,096-template/32-plugin deterministic model eliminates
`131072` formatted child strings and `131072` temporary vector slots; the
final generated expression remains. Ignored
`RUNTIME878_EXPORT_PLUGIN_LIST_DIRECT_BENCH_V1` records 101 alternating
Release p50/p95/p99 pairs and requires optimized p95 <= 110% of the retired
path; it has not been run.

The pair plus preceding animation/export/asset-palette contracts pass `31/31`.
Exact Rustfmt and scoped diff checks pass. No per-task Cargo run was performed.
The current Runtime/Editor/App source was submitted together in async v25 as
PID `28484` at `2026-09-21T23:28:01.8534392+08:00`, without monitoring.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/plugin/export_build_plan/plugin_selection_template.rs` | `05FC9E1A5DA9E2A56C467B91BEF822F2825E35C6517D22C6F07FBE3DE8360FDB` |
| `zircon_runtime/src/plugin/export_build_plan/plugin_selection_template/direct_string_vec_tests.rs` | `6C603D2EAD9B88A7A6BB8303A2DD9D8772C3A54E8C516F0A00E1DD31FF3A7789` |
| `tools/tests/test_runtime878_export_plugin_list_direct_performance_contract.py` | `09BBE3270D6ADBBDCD80708647CC179C33F9065D23E8CD7F0854C706B4330629` |

## Acceptance boundary

The deterministic intermediate-elimination target is met. A bounded v25
failure diagnosis found `compile_input_changed` while synchronizing a foreign
modified Runtime export-archive file; no Runtime source error or passing
compile receipt is attributed to this v25 snapshot. Managed current-source
Runtime compilation, Rust lower tests, ignored Release percentiles, allocator
measurement, and export-product p50/p95/p99 remain pending. The asynchronous
development build uses `-SkipTest` and cannot close them.
