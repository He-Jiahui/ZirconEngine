---
title: Editor Showcase State Flag Stack
category: zircon_editor
report_id: Editor895-showcase-state-flag-stack-2026-09-21
date: 2026-09-21
session_id: root-runtime-editor-async-optimization-20260921
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor895 Showcase State Flag Stack

## Finding and optimization

The component showcase value fallback previously allocated a heap `Vec<&str>`
for at most eight known state labels on each summary projection, then joined
them into the required output. It now fills an eight-slot stack array and
joins only the initialized prefix into that output. This supersedes the
Editor145 fixed-capacity heap-vector choice without changing flag priority,
labels, commas, empty fallback, UI state ownership, or the required result.
The Editor145 source-shape regression now checks stack storage; its older
ignored capacity-vs-unreserved-Vec microbenchmark remains historical evidence
for that previous implementation, not an acceptance measurement for Editor895.

## TDD and deterministic evidence

The combined Editor895/Runtime876 contracts were RED `2/10` and GREEN `10/10`.
A lower regression compares all `256` flag masks with the retired heap-vector
implementation. Across 4,096 all-enabled summaries, the deterministic model
eliminates `4096` temporary vector allocations and `32768` reference slots;
the final joined output remains. Ignored
`EDITOR895_SHOWCASE_STATE_FLAG_STACK_BENCH_V1` records 101 alternating Release
p50/p95/p99 pairs and requires optimized p95 <= 110% of the retired path;
it has not yet been run.

The pair plus adjacent Runtime871, Shader, Showcase, and material contracts
pass `49/49`. Exact Rustfmt, Python bytecode compilation, and scoped diff
checks pass. There was no per-task Cargo run. The current-source
Runtime→Editor→App batch v23 was launched asynchronously as PID `23296` at
`2026-09-21T22:58:13.7141657+08:00`; no v22/v23 status was monitored.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/template_runtime/showcase_demo_state/state_panel.rs` | `00E0238BF7156A2B00B0EC2474D525B838474C98CB06EA364ACF197038BC1BF9` |
| `zircon_editor/src/ui/template_runtime/showcase_demo_state/state_panel/capacity_tests.rs` | `AC6312A005C50A05318BA206F103BAEECBD2C499DD7206F5FDCD87C86724ADE4` |
| `zircon_editor/src/ui/template_runtime/showcase_demo_state/state_panel/stack_buffer_tests.rs` | `B281233F6FF0CC5A73E012E5DDEFE7C66CAEB88E705B108F7351A6AEB030FCFB` |
| `tools/tests/test_editor895_showcase_state_flag_stack_performance_contract.py` | `C6B1C7F4D5D51653B969029379AFB9D82C6BBF43EB517405A449D8BE4399F8E9` |

## Acceptance boundary

Static allocation targets are met; current-source Windows compilation, Rust
lower tests, ignored Release benchmarks, allocator measurement, and actual
Showcase product p50/p95/p99 remain pending. An async launch alone is not
performance acceptance.
