---
handoff_kind: fixed
status: fixed
created_at: 2026-08-28
summary_slug: platform-bundle-compile-host-fixture-drift
origin_plan: docs/plans/zircon_plugins/13-standalone-plugin-build.md
fixing_plan: docs/plans/zircon_plugins/13-standalone-plugin-build.md
origin_child_dir: docs/plans/zircon_plugins/13
fixing_child_dir: docs/plans/zircon_plugins/13
plan_link_mode: child_record_only
failure_scope: local
related_code:
  - tools/zircon_export/pipeline_report_compile_host.py
tests:
  - tools/zircon_export/tests/platform_bundle_report_test_support.py
  - tools/zircon_export/tests/test_pipeline_report_platform_bundle_native_plugins_payload.py
resolved_at: 2026-08-31
---

# Plugins13 PlatformBundle CompileHost fixture drift

## 来源执行者

- 来源计划：`docs/plans/zircon_plugins/13-standalone-plugin-build.md`
- 来源执行切片：non-NativeDynamic pipeline report behavior sweep
- 修复责任计划：`docs/plans/zircon_plugins/13-standalone-plugin-build.md`
- 交接原因：Plugins13 owns PlatformBundle and CompileHost report fixtures.

## 失败现象与复现证据

Two PlatformBundle native-plugins acceptance cases fail before their target
assertions. The shared fixture emits a retired CompileHost `link_plan`, omits
`staged_engine_root`, and uses a direct Cargo command instead of the hard-cut
`tools/zircon_build.py` command.

## 最低共享层根因

`platform_bundle_report_test_support.py` duplicates an old CompileHost report
shape rather than using the canonical writer already shared by other pipeline
tests. Thirty-seven PlatformBundle report modules consume this stale fixture.

## 架构修复验收

- Delegate CompileHost report construction to `_write_compile_host_report`.
- Use the client-runtime staged `ZirconEngine/zircon_hub.exe` identity as the
  PlatformBundle host source.
- Remove legacy `link_plan` construction from the shared fixture.
- Pass the original native-plugins payload cases and PlatformBundle report
  consumer suites.

## 禁止临时方案

- Do not restore `link_plan` acceptance or weaken CompileHost schema checks.
- Do not copy current report fields into another handwritten fixture.
- Do not suppress fatal CompileHost diagnostics in PlatformBundle tests.

## 修复结果与回传

- 根因：The shared PlatformBundle test fixture duplicated a retired CompileHost report shape with link_plan and direct Cargo command instead of delegating to the canonical staged-build report writer.
- 架构修复：Commit e225a7489 delegates fixture construction to the canonical CompileHost writer and uses the staged client-runtime Hub executable identity, removing the handwritten legacy shape without changing production semantics.
- 验证：Fresh focused PlatformBundle native-plugins payload module passed 20/20. The committed repair also records the five-module consumer slice at 68/68 and report owner guard at 3/3; all lifecycle and related owner paths are clean.
- 回传：Plugins13 PlatformBundle fixtures now consume the canonical staged CompileHost report; the legacy fixture drift is fixed.
