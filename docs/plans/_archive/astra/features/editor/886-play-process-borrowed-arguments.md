---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/07/2026-09-21-play-process-borrowed-arguments.md
related_records:
  - docs/plans/astra/features/editor/664-20260911-editor-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/core/play/process_backend/command.rs
  - zircon_editor/src/core/play/process_backend/mod.rs
  - zircon_editor/src/core/play/process_backend/tests.rs
tests:
  - zircon_editor/src/core/play/process_backend/command/borrowed_arguments_tests.rs
  - tools/tests/test_editor04_process_play_backend_contract.py
  - tools/tests/test_editor886_play_process_borrowed_arguments_performance_contract.py
---

# Editor886 Play Process Borrowed Arguments

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Editor07 Play process launch | Replace the fixed eight-argument `Vec<OsString>` and diagnostic re-projection with one borrowed `[&str; 8]` authority shared by `Command::args` and diagnostic rendering. Executable, project/profile, scene, pipe, order, and exact spelling remain unchanged. | Intentional RED `1/5` → GREEN `5/5`; lower exact-array/diagnostic/real-Command regressions and ignored `EDITOR886_PLAY_PROCESS_BORROWED_ARGUMENTS_BENCH_V1` are wired. The 4,096-render model changes owned argument strings/vector slots `32768/65536→0/0`; Editor879–887 plus adjacent contracts pass `71/71`. | implemented_pending_validation |

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/core/play/process_backend/command.rs` | `ED290B3C740AA8C78BEFC4D9A49C6AE5D6632418CF9D5F2937266604CBC2A3E3` |
| `zircon_editor/src/core/play/process_backend/mod.rs` | `D07BE7053B10C06E64C1D6D1CEE07C158DAB1EF4FCEA8EFE17FA0E245D078346` |
| `zircon_editor/src/core/play/process_backend/tests.rs` | `2702AFD07E9BBC439D8276CDEDB1BFD2F9D4479B60E56FB85EB9E69571944E1F` |
| `zircon_editor/src/core/play/process_backend/command/borrowed_arguments_tests.rs` | `0EA2BDE67FE40C15FD606E2DAD9CE7F8D0142ABC7107F7E6B39D6DBD3782A5DD` |
| `tools/tests/test_editor04_process_play_backend_contract.py` | `16B0C438B852644F1BD9A673613D5E108AF9959AFE8A0CD0BA30355DD327B90B` |
| `tools/tests/test_editor886_play_process_borrowed_arguments_performance_contract.py` | `2DA831EA0A55DD27A0256863649E710EF56DFE70CB3E9D40C6938310C3D4C593` |

## Managed gate

Editor886 was submitted with Editor887 in asynchronous v14 (PID `6460`) at
`2026-09-21T20:57:12.9209888+08:00` rather than receiving a per-task Cargo
run. Keep it pending until that combined Windows lane supplies current-source
Editor compilation, lower/ignored Release execution, allocator evidence, and
play-launch product p50/p95/p99 evidence.
