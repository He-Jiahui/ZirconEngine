---
handoff_kind: failure
status: open
failure_scope: local
created_at: 2026-08-22
summary_slug: module-graph-validation-ui-blockers
origin_plan: docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md
fixing_plan: docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md
origin_child_dir: docs/plans/zircon_runtime/runtime/02
fixing_child_dir: docs/plans/zircon_runtime/runtime/02
plan_link_mode: child_record_only
source_session: runtime02-module-graph-iterative-m1-r1-01a019a5-20260822
validation_job: 0cc224e1cbb74c0b962706fd30111c49
validation_log: D:/ZirconBuilds/runtime02-module-order-20260822-231940.stderr.log
related_code:
  - zircon_runtime/src/core/runtime/modules
  - zircon_runtime/src/tests/runtime_absorption/module_order.rs
tests:
  - .\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -LibTests -TestFilter module_order -VerboseOutput
  - full-default-feature WPR/xperf module graph evidence
---

# Runtime02: module graph managed validation pending UI dependency recovery

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md`
- 来源执行切片：Runtime02 module graph iterative M1 managed validation, session `runtime02-module-graph-iterative-m1-r1-01a019a5-20260822`
- 修复责任计划：`docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md`
- 交接原因：Runtime02 owns the exact `module_order` rerun and profiling acceptance after separately owned UI compile failures are repaired; it does not own the UI source fixes.

## 失败现象与复现证据

The managed Windows validator ran:

```powershell
& .\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -LibTests -TestFilter module_order -VerboseOutput
```

Coordinator job `0cc224e1cbb74c0b962706fd30111c49` used the governed D-drive target pool and finished at 2026-08-22 23:33:36 +08:00 with exit code `1`. Package-wide lib/lib-test compilation failed before the filtered `module_order` test executed.

The first production diagnostic was a private template-asset import plus inference failures in `ui/v2/component_instancer.rs`. Lib-test compilation then reported UI-owned errors including missing `TARGET_BINDING_COUNT`, missing `measure_line_width`, stale `UiAssetLoader::load_str`, removed `UiCompiledDocument::root`, and changed binding-transaction arity. No compiler error was attributed to Runtime02 module graph files. A local unused-variable warning in `module_order.rs` was repaired after this attempt, but static cleanup is not execution evidence.

The UI binding diagnostics are already routed by canonical records, including:

- `docs/plans/optimize/zircon_runtime/74/failure-2026-08-22-runtime74-ui-compiled-asset-id-public-reexport.md`
- `docs/plans/optimize/zircon_runtime/74/failure-2026-08-22-text03-compiled-binding-contract-compile.md`
- `docs/plans/optimize/zircon_runtime/74/failure-2026-08-22-ui-asset-binding-canonical-loader-api-tests.md`

## 最低共享层根因

`cargo test -p zircon_runtime --lib <filter>` must compile the package library and complete lib-test harness before the name filter can run. Current-source UI contract/test drift therefore blocks the Runtime02 test at a lower compilation gate even though the module graph files emit no errors. The individual UI owners remain authoritative; this local Runtime02 lifecycle only records the missing post-repair rerun and performance evidence.

## 架构修复验收

- The canonical Runtime74/Text owner records reach accepted current-source evidence, or a fresh managed Runtime02 run proves those exact diagnostics absent without absorbing their source scope.
- Rerun the exact managed `zircon_runtime` `module_order` command with `--locked`; the lib-test harness compiles and the filtered tests execute successfully.
- Collect the required full-default-feature WPR/xperf module graph evidence under host policy and bind it to the same current-source validation milestone.
- Only after both behavioral and profiling evidence are accepted may Runtime02 return this lifecycle as `fixed-*`.

## 禁止临时方案

- Runtime02 must not edit Runtime74/Text-owned UI sources or tests to force its validation through.
- Do not remove `--locked`, bypass the coordinator Cargo lane, narrow features to hide default UI compilation, or treat a core-min run as the full-default-feature gate.
- Do not duplicate the existing Runtime74/Text failure lifecycles or claim their static progress as this Runtime02 managed pass.
- Do not mark this record fixed from formatting, review, or a warning-only source scan.

## 修复结果与回传

Open state: the original managed job did not execute `module_order`, and no replacement managed test or accepted full-default-feature WPR/xperf receipt is recorded here. This update only restores canonical schema and clarifies owner boundaries; it does not claim Cargo green, profiling acceptance, `fixed-*` return, or completion notification.
