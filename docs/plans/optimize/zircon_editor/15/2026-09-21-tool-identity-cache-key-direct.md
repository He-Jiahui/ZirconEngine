---
title: Editor Tool Identity Cache Key Direct
category: zircon_editor
report_id: Editor896-tool-identity-cache-key-direct-2026-09-21
date: 2026-09-21
session_id: root-runtime-editor-async-optimization-20260921
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor896 Tool Identity Cache Key Direct

## Finding and optimization

Tool-version identity previously joined all version arguments into one
intermediate string and interpolated it into a second full cache key. The
export inventory now reserves a lower-bound final `String`, writes the key and
`OsStr` debug representation, and appends borrowed arguments with positional
NUL separators. It preserves debug escaping, empty arguments, embedded NULs,
Unicode, and cache-key identity. `OsStr` debug escaping can exceed the lower
bound, so this change only claims elimination of the joined child, not a
guarantee of exactly one output-buffer allocation.

## TDD and deterministic evidence

The combined Runtime877/Editor896 contracts were RED `1/8` and GREEN `8/8`.
A lower regression compares original bytes for empty, Unicode, escaped,
embedded-NUL, and multi-argument inputs. For 4,096 keys with 32 arguments,
the deterministic model removes `4096` joined child strings; the required
cache key remains. Ignored
`EDITOR896_TOOL_IDENTITY_CACHE_KEY_DIRECT_BENCH_V1` records 101 alternating
Release p50/p95/p99 pairs and requires optimized p95 <= 110% of the retired
path; it has not yet been run.

The pair plus adjacent animation/export source contracts pass `48/48`.
Exact Rustfmt, Python bytecode compilation, and scoped diff checks pass. There
was no per-task Cargo run. A combined current-source Runtime/Editor/App batch
v24 was launched asynchronously as PID `33516` at
`2026-09-21T23:12:59.6906963+08:00`; no v24 receipt has been read or monitored.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/core/export/inventory.rs` | `2855644FA027FE094BC2DF1D2A79447F8259F5F687FEEC7D843EA83251A7720F` |
| `zircon_editor/src/core/export/inventory/tool_cache_key_tests.rs` | `EF25988EF88C00F47CFF0F355E89ED87B07DBAF50C6D442AA539F8EA14B40E25` |
| `tools/tests/test_editor896_tool_identity_cache_key_direct_performance_contract.py` | `8B2DDDF53E8BA95100F19DB20ABD8A15A1C40BF7ABB2008C873C899C4FCF2A6A` |

## Acceptance boundary

A single bounded v24 receipt read after independent work shows the managed
Editor/App admissions failed because the external reuse pool was busy; it
does not establish either a compile pass or a source compile error. The
static join-elimination target is met; Editor compilation, Rust lower tests,
ignored Release percentiles, allocator measurement, and export-product
p50/p95/p99 remain pending. A `-SkipTest` build could not satisfy those gates.
