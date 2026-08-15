---
handoff_kind: failure
status: open
created_at: 2026-08-05
updated_at: 2026-08-11
summary_slug: script-build-diagnostics-editor-log-source-bridge
origin_plan: docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md
fixing_plan: docs/plans/zircon_editor/editor/13-script-compilation-management.md
origin_child_dir: docs/plans/zircon_editor/editor/17
fixing_child_dir: docs/plans/zircon_editor/editor/13
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/core/script_build/orchestrator.rs
  - zircon_editor/src/core/script_build/diagnostics_sink.rs
  - zircon_runtime_interface/src/script_diagnostics
  - zircon_editor/src/core/logging
tests:
  - script diagnostic severity/module/jump projection
  - stale completion and replay deduplication
  - bounded diagnostic stream without a second activity history
---

# Editor13: script build diagnostics do not feed the canonical log source

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md`
- 来源执行切片：M3.1 six-source logging aggregation.
- 修复责任计划：`docs/plans/zircon_editor/editor/13-script-compilation-management.md`
- 交接原因：script compilation request/step identity, diagnostics and source locations belong to Editor13; Editor17 only owns the common log record contract.

## 失败现象与复现证据

The ScriptBuild plan defines a diagnostic sink and typed runtime-interface DTO, yet no production producer publishes those diagnostics through `EditorLogService`. Script warnings/errors therefore cannot participate in the common channel/severity filter or typed Activity jump dispatch.

## 最低共享层根因

Editor13 has compilation diagnostic identities and source locations, but no bounded request/step/generation bridge projects those immutable facts into the shared log authority.

## 架构修复验收

- Editor13 emits new script diagnostic facts through the existing `EditorLogService` as `LogSource::script_build()` with explicit severity mapping and source-location jump data where the DTO supplies it.
- Request/step/generation identity provides bounded cursor/deduplication, so delayed/stale completion never writes a record for a superseded build and replay does not duplicate a record.
- Script build state/Progress remains its own authority; logging consumes immutable facts only and does not become a second diagnostics panel or queue.
- Tests cover compile warning/error, source jump, stale completion, duplicate replay, failure-before-refresh and a bounded diagnostic storm.

## 禁止临时方案

- Do not write compiler text only to a private ScriptBuild panel, raw stdout, or a retained-host queue.
- Do not introduce a script-specific logger, a new log channel, or an unbounded `Vec` retained for Activity history.

## 修复结果与回传

Open state: `current_source_implemented / static_contract_green / managed_cargo_artifact_gate_blocked`. Editor13 now owns a serializable `ScriptDiagnostic` DTO and a fixed-size request/step/generation projection cursor. Accepted completion facts map all three severities and optional source locations into Editor17's existing `EditorLogService` / `LogSource::script_build()` authority; replay and delayed older generations do not append records. The failure remains open until coordinator-managed Rust tests can run after the repository-wide unregistered-artifact gate clears.

## 产出记录与时间

| 日期 | 切片 | 状态 | 完成项目与验证证据 |
| --- | --- | --- | --- |
| 2026-08-05 | Editor13 script diagnostics -> Editor17 M3.1 source bridge | `open_handoff_recorded` | Source inspection confirms the planned ScriptBuild diagnostic domain and core log channel exist, while no production projection is present. No script/runtime/editor source was changed. |
| 2026-08-11 | Typed DTO + bounded canonical-log projection | `current_source_implemented_validation_pending` | Added runtime-interface DTO JSON round-trip coverage, explicit info/warning/error mapping, script-location jumps, and a single fixed-size cursor keyed by generation/request/step. Rust regression cases cover stale dispatch, delayed accepted completion, replay, failure-before-refresh, and a 256-diagnostic bounded-store storm. `python -m unittest tools.tests.test_editor13_script_build_orchestrator_contract -v` is 8/8 GREEN; scoped `rustfmt --check` and `git diff --check` are GREEN. Coordinator Cargo acquisition was rejected before job creation by the repository-wide unregistered D/E/F artifact gate, so no closeout or return is claimed. Coordinator snapshot: `1625`. |
