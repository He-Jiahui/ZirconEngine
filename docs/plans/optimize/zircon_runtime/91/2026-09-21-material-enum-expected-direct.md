---
title: Runtime Material Enum Expected Direct
category: zircon_runtime
report_id: Runtime876-material-enum-expected-direct-2026-09-21
date: 2026-09-21
session_id: root-runtime-editor-async-optimization-20260921
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Runtime876 Material Enum Expected Direct

## Finding and optimization

Invalid material-option values previously caused enum choices to be joined
into a whole intermediate string and then formatted into the final
`one of ...` diagnostic. The enum branch now computes its final UTF-8 byte
capacity from borrowed choices and delimiters, then appends all values into
one result. Empty choices, empty-string entries, exact comma placement,
Unicode, and the independent bool/empty-enum text are unchanged. This
affects shader-schema mismatch explanation, not material option validation
authority or valid-value dispatch.

## TDD and deterministic evidence

The combined Editor895/Runtime876 contracts were RED `2/10` and GREEN `10/10`.
A lower regression compares singleton, empty, interior-empty, CJK/emoji,
and bool/empty-enum branches with the retired text. For 4,096 invalid enum
descriptions the deterministic model removes `4096` joined child strings;
the required final diagnostic remains. Ignored
`RUNTIME876_MATERIAL_ENUM_EXPECTED_DIRECT_BENCH_V1` records 101 alternating
Release p50/p95/p99 pairs and requires optimized p95 <= 110% of the retired
path; it has not yet been run.

The pair plus adjacent Runtime871, Shader, Showcase, and material contracts
pass `49/49`. Exact Rustfmt, Python bytecode compilation, and scoped diff
checks pass. There was no per-task Cargo run. The current-source
Runtime→Editor→App batch v23 was launched asynchronously as PID `23296` at
`2026-09-21T22:58:13.7141657+08:00`; no v22/v23 status was monitored.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/core/framework/render/shader/material_property_layout.rs` | `B3059B36C7944EB9227E67E017AE5BEF995B5213D0D49C62482D81B045BE8DCF` |
| `zircon_runtime/src/core/framework/render/shader/material_property_layout/enum_description_tests.rs` | `19F468DCCCEFED106E7E109267309C4CDFDEC8DF85457B5C0BBEC31ED02331C6` |
| `tools/tests/test_runtime876_material_enum_expected_direct_performance_contract.py` | `0323C299EFAB362079F27A4CD1DC4815620CC4250E5683BD37860D032DBF3418` |

## Acceptance boundary

Static allocation targets are met; current-source Windows compilation, Rust
lower tests, ignored Release benchmarks, allocator measurement, and actual
material-schema product p50/p95/p99 remain pending. An async launch alone is
not performance acceptance.
