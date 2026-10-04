---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/91/2026-09-21-shader-import-direct-path.md
related_records:
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/core/framework/render/shader/module_import.rs
tests:
  - zircon_runtime/src/core/framework/render/shader/module_import/direct_path_tests.rs
  - tools/tests/test_runtime875_shader_import_direct_path_performance_contract.py
---

# Runtime875 Shader Import Direct Path

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime91 shader import derivation | Borrow normalized module components and append each normalized segment into one bounded import-path buffer; preserve extension stripping, folder fold, namespace and error semantics while removing staging owners. | Combined RED `2/11` → GREEN `11/11`; adjacent static `57/57`; ten-input lower parity and ignored 101-pair `RUNTIME875_SHADER_IMPORT_DIRECT_PATH_BENCH_V1` wired. The 4,096-path/32-segment deterministic model eliminates `131072` module clones, `131072` module slots, `131072` child strings, `135168` staging slots, and `4096` joins. | implemented_pending_validation |

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/core/framework/render/shader/module_import.rs` | `083D9B26672B0400763369E42E3E2CDA659A1FD875CD65A7D341853A79129108` |
| `zircon_runtime/src/core/framework/render/shader/module_import/direct_path_tests.rs` | `1E55B1037EE1763699CE4A4BBFAD1B8702BCF6FB44D75ECF0AE29CDEB3843B3B` |
| `tools/tests/test_runtime875_shader_import_direct_path_performance_contract.py` | `A3D8F66BB02728524346F0CFCDB965C07B2EA0B07F825364105C4287CF049737` |

## Managed gate

Runtime875 was submitted with Editor894 in asynchronous v21 (PID `34372`) at
`2026-09-21T22:39:27.9275627+08:00`. Keep it pending until current-source
Runtime compilation, lower/ignored Release execution, allocator evidence, and
shader-import product p50/p95/p99 evidence exist. A single v21 snapshot
identified an earlier owned Runtime871 `E0596` compile failure. That source
was repaired, and the combined matrix was resubmitted as v22 (PID `35456`)
at `2026-09-21T22:51:32.5348309+08:00`. No v22 receipt was read or monitored;
no per-task Cargo run was made.
