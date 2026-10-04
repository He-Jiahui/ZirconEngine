---
title: Runtime Shader Import Direct Path
category: zircon_runtime
report_id: Runtime875-shader-import-direct-path-2026-09-21
date: 2026-09-21
session_id: root-runtime-editor-async-optimization-20260921
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Runtime875 Shader Import Direct Path

## Finding and optimization

Shader import derivation cloned module path components, transformed every
component into a child `String`, staged those strings in another vector, then
joined them into the final import path. The module selector now borrows a slice
of the already normalized asset path and reports whether its terminal extension
needs stripping. The derived import appends each normalized segment directly
to one input-bounded output. Existing namespace validation, case folding,
punctuation collapse, leading-digit underscore, `.wgsl`/`.zshader` handling,
matching-directory fold, errors, and historical `folded_terminal_directory`
flag semantics remain in their original owners.

## TDD and bounded evidence

The combined Editor894/Runtime875 source-model contracts were RED `2/11` and
GREEN `11/11`. A lower regression compares ten success/error inputs with the
retired derivation, including backslashes, Unicode, reserved namespaces, and
the `shaders/shaders.zshader` edge case. Across 4,096 paths of 32 module
segments, the deterministic model removes `131072` cloned module strings,
`131072` borrowed module-vector slots, `131072` normalized child strings,
`135168` staging-vector slots, and `4096` joined path strings. Existing path
normalization, error-path text and the required final import remain. Ignored
`RUNTIME875_SHADER_IMPORT_DIRECT_PATH_BENCH_V1` records 101 alternating
Release p50/p95/p99 pairs and requires optimized p95 <= 110% of the retired
implementation; it has not yet been run.

The pair and adjacent popup, visual, live-state, notification, and diagnostic
contracts pass `57/57`. Exact Rustfmt, Python bytecode compilation, and scoped
diff checks pass. No per-task Cargo run was made. The combined current-source
Runtime→Editor→App Windows validation v21 was launched as PID `34372` at
`2026-09-21T22:39:27.9275627+08:00`; no live receipt was read or monitored.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/core/framework/render/shader/module_import.rs` | `083D9B26672B0400763369E42E3E2CDA659A1FD875CD65A7D341853A79129108` |
| `zircon_runtime/src/core/framework/render/shader/module_import/direct_path_tests.rs` | `1E55B1037EE1763699CE4A4BBFAD1B8702BCF6FB44D75ECF0AE29CDEB3843B3B` |
| `tools/tests/test_runtime875_shader_import_direct_path_performance_contract.py` | `A3D8F66BB02728524346F0CFCDB965C07B2EA0B07F825364105C4287CF049737` |

## Acceptance boundary

The one-time v21 receipt exposed an earlier Runtime871 `E0596` double-borrow
error, so v21 does not establish this shader derivation's compilation. The
owned Runtime871 source was repaired and the combined current-source
Runtime→Editor→App matrix was resubmitted as v22 (PID `35456`) at
`2026-09-21T22:51:32.5348309+08:00`. No v22 receipt was read.

Implementation and static allocation-model targets are met, but current-source
Windows compilation, lower regression, ignored Release marker, allocator
measurement, and shader-import product p50/p95/p99 gates are pending. Do not
claim performance acceptance from this model or the asynchronous launch alone.
