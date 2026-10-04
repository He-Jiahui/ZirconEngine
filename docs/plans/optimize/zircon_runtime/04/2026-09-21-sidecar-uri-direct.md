---
title: Runtime Sidecar URI Direct
category: zircon_runtime
report_id: Runtime879-sidecar-uri-direct-2026-09-21
date: 2026-09-21
session_id: root-runtime-editor-async-optimization-20260921
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Runtime879 Sidecar URI Direct

## Finding and optimization

Missing sidecar generation previously collected the migration source's
relative path components as `Cow<str>` values, joined them into a complete
slash-separated child, then formatted a second complete `res://` URI for
`AssetUri::parse`. It now appends lossy component text and separators directly
to the URI output, reserving the available source-path byte bound. On Windows
invalid Unicode replacement can exceed that lower bound, so this change does
not guarantee exactly one output allocation; it removes the component vector,
joined child, and separately formatted wrapper. Component normalization,
error mapping, `AssetUri` validation, and migration inventory ownership stay
unchanged.

## TDD and deterministic evidence

The focused contract was RED with two missing-function/lower-module errors,
then GREEN `3/3`. A lower regression compares empty, dot, collapsed,
parent-relative, nested, and CJK/emoji paths with the retired component join;
Windows additionally exercises a lossy invalid-Unicode component. A
4,096-path/32-component deterministic model eliminates `131072` temporary
component-vector slots and `8192` joined/formatted child strings; the
required final URI remains. Ignored `RUNTIME879_SIDECAR_URI_DIRECT_BENCH_V1`
records 101 alternating Release p50/p95/p99 pairs with an optimized p95 <=
110% retired-path guard; it has not been run.

Runtime879 plus recent Runtime/Editor source contracts pass `48/48`, with
exact Rustfmt and scoped diff checks. This slice was implemented after v25
source submission, whose Runtime package hit foreign `compile_input_changed`.
No per-task Cargo run or competing coordinator submission was made; it joins
the next grouped wave.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/asset/migration/sidecar.rs` | `D196B0D396197B4045A2BDE6F7274DB84CA7276A25D708F460C479AE0EB9327D` |
| `zircon_runtime/src/asset/migration/sidecar/direct_uri_tests.rs` | `3BDA7648DD58876BCCB243D365C02C762FC033A5B77E99E8BCFAC343456DBA99` |
| `tools/tests/test_runtime879_sidecar_uri_direct_performance_contract.py` | `8BF4822168112230A2C8F2E87EDB1B7EDD4A65DBD58FBC74F6E6D1D84607B506` |

## Acceptance boundary

The static intermediate-elimination target is met. Current-source managed
Runtime compilation, Rust lower tests, ignored Release percentiles, allocator
count/bytes, and sidecar-migration product p50/p95/p99 remain pending. The
earlier v24 Runtime development receipt predates this source and is not
acceptance evidence for Runtime879.
