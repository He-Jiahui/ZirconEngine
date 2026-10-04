---
title: Editor Play Process Borrowed Arguments
category: zircon_editor
report_id: Editor886-play-process-borrowed-arguments-2026-09-21
date: 2026-09-21
session_id: root-runtime-editor-async-optimization-20260921
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor886 Play Process Borrowed Arguments

## Finding

Every process launch rebuilt the fixed eight play arguments as a
`Vec<OsString>`. Diagnostic rendering then consumed another argument vector,
converted every item into an owned lossy `String`, collected a second vector,
and joined it. These arguments already borrow stable command fields and static
flags, so both layers of owned staging were avoidable.

## Optimization

- Return the complete fixed argument contract as `[&str; 8]`.
- Make `Command::args` and diagnostic rendering consume that same borrowed
  argument authority.
- Remove the `OsString` materialization and the diagnostic's second owned
  argument projection while preserving exact order and spelling.
- Keep executable resolution, project root, runtime profile, scene snapshot,
  report-pipe routing, and process configuration unchanged.

## TDD and deterministic evidence

The Editor886 source/model contract was observed RED at `1/5` and GREEN at
`5/5`. Lower regressions lock the exact borrowed array, diagnostic text, and
the arguments installed on the real `Command`.

Across 4,096 diagnostic renders, the deterministic model changes owned
argument strings from `32768` to `0` and temporary vector slots from `65536`
to `0`; the required final diagnostic string remains. Ignored marker
`EDITOR886_PLAY_PROCESS_BORROWED_ARGUMENTS_BENCH_V1` emits 101 alternating
p50/p95/p99 sample pairs and requires borrowed rendering p95 to remain within
10% of the retired owned staging path.

## Local validation boundary

- Exact-file Rustfmt, Python bytecode compilation, and scoped
  `git diff --check` pass.
- Editor879–887 plus adjacent Play/preview/world-space/overlay contracts pass
  `71/71` in one batch.
- Editor886 received no per-task Cargo run and was submitted with Editor887 in
  asynchronous v14 (PID `6460`).
- Local evidence does not establish Windows compilation, allocator behavior,
  or real play-launch p50/p95/p99 latency.
- Tooling production remains deferred for the later Rust migration.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/core/play/process_backend/command.rs` | `ED290B3C740AA8C78BEFC4D9A49C6AE5D6632418CF9D5F2937266604CBC2A3E3` |
| `zircon_editor/src/core/play/process_backend/mod.rs` | `D07BE7053B10C06E64C1D6D1CEE07C158DAB1EF4FCEA8EFE17FA0E245D078346` |
| `zircon_editor/src/core/play/process_backend/tests.rs` | `2702AFD07E9BBC439D8276CDEDB1BFD2F9D4479B60E56FB85EB9E69571944E1F` |
| `zircon_editor/src/core/play/process_backend/command/borrowed_arguments_tests.rs` | `0EA2BDE67FE40C15FD606E2DAD9CE7F8D0142ABC7107F7E6B39D6DBD3782A5DD` |
| `tools/tests/test_editor04_process_play_backend_contract.py` | `16B0C438B852644F1BD9A673613D5E108AF9959AFE8A0CD0BA30355DD327B90B` |
| `tools/tests/test_editor886_play_process_borrowed_arguments_performance_contract.py` | `2DA831EA0A55DD27A0256863649E710EF56DFE70CB3E9D40C6938310C3D4C593` |

## Acceptance boundary

Keep this record `implementation_complete` / `managed_validation_pending` until
a combined current-source Windows lane compiles Editor, executes the lower
regression and ignored Release marker, and supplies allocator plus real
play-launch p50/p95/p99 evidence. The deterministic allocation model is not
product acceptance.
