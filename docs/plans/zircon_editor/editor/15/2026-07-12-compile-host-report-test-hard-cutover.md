---
status: complete
owner_plan: docs/plans/zircon_editor/editor/15-build-export-and-publishing.md
recorded_at: 2026-07-12
related_code:
  - tools/zircon_export
tests:
  - python -m unittest tools.zircon_export.tests
---

# CompileHost 旧报告测试硬切到 staged-build 合同

## 失败边界

`tools.zircon_export.tests.test_pipeline_report_compile_host_command_semantics` 与共享 fixture 曾继续
构造 Cargo 直编命令和旧 `link_plan` 字段。新版严格 schema 正确拒绝这些旧报告，最初产生
10 tests / 18 assertion failures。该失败归 Editor 15 测试合同迁移，生产代码未恢复任何兼容分支。

## Owner 修复

- CompileHost test fixture 改为 `staged/ZirconEngine/zircon_hub.exe` 与
  `python tools/zircon_build.py --targets ... --out ... --mode ...`。
- command semantics、stage schema、metadata/linkage 与 PlatformBundle handoff tests 删除旧
  Cargo/report 权威断言，改测 staged-build 参数、legacy option rejection、preset mode 权威与
  staged root/launcher 传递。
- Validate 内部的 LibraryEmbed Cargo plan 仍只在 Validate owner 内测试；它不再被复制成生产
  CompileHost stage report，也不再覆盖 `ExportPreset.debug`。

## 回归证据

| 测试组 | 结果 |
| --- | --- |
| staged CompileHost command/schema | 12/12 通过 |
| final report、metadata、cook/pack handoff | 91/91 通过 |
| Validate compile-plan/schema/selection | 106/106 通过 |
| PlatformBundle resume/input/native-dynamic handoff | 53/53 通过 |
| 合计 | 262/262 通过 |

## 产出记录与时间

| 时间 | 状态 | 产出 |
| --- | --- | --- |
| 2026-07-12 20:45 +08:00 | 未通过，已归档 | 旧 CompileHost report semantics tests 为 10 tests / 18 failures；生产代码未恢复旧架构兼容。 |
| 2026-07-12 21:18 +08:00 | 已修复 | 共享 fixture 与全部受影响测试硬切 staged-build 合同；四组 262/262 通过。本计划内部测试迁移不是跨计划 handoff，因此归档为普通完成记录。 |
