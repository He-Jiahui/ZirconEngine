---
handoff_kind: failure
status: open
created_at: 2026-07-11
summary_slug: migrate-assets-commandlet-registry
origin_plan: docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md
fixing_plan: docs/plans/zircon_editor/editor/16-cli-args-and-hub-integration.md
origin_child_dir: docs/plans/zircon_editor/editor/10
fixing_child_dir: docs/plans/zircon_editor/editor/16
related_code:
  - zircon_runtime/src/asset/migration
  - zircon_editor/src/core/commands
tests:
  - cargo test -p zircon_editor --lib --locked commandlet
  - cargo test -p zircon_app --locked cli
---

# Editor 16：migrate-assets 尚无统一命令注册表投影与无头 CLI runner

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md`
- 来源执行切片：Plan10 M2.3 `migrate-assets` commandlet 与持久引用硬切
- 修复责任计划：`docs/plans/zircon_editor/editor/16-cli-args-and-hub-integration.md`
- 交接原因：Runtime 已拥有迁移扫描、严格解析、dry-run/apply 与跨文件事务能力；`--run` 的唯一注册表投影、无头引导和退出码合同明确属于 Plan16 M2，且依赖 Plan08 将命令注册表收敛到 `zircon_editor::core`。在 `zircon_app` 本地补第二套命令表会直接违反计划边界。

## 失败现象与复现证据

Plan10 M2.3 已实现 `zircon_runtime::asset::migration`。源码复核（2026-08-15）确认当前编辑器可执行体已通过 `zircon_app::entry::cli::EditorLaunchArgs` 在 GUI 启动前路由：

```text
zircon_editor --run migrate-assets --project <project-root> --dry-run
zircon_editor --run migrate-assets --project <project-root> --apply
```

`zircon_editor::core::commandlet::runner` 使用 `EditorCommandRegistry::default_workbench()` 的 `migrate-assets` 描述符、Headless capability 投影与 Runtime migration API；`EntryRunner::run_editor_with_args_exit_code` 输出稳定 JSON，并返回 0/1/2/3。未重新引入 `zircon_app/src/commandlet` 的第二注册表或命令专用旁路。

本 failure 仍保持 open：声明的受管 Cargo 验收尚未在共享验证窗口执行。commandlet 文件系统 fixture 现强制使用 `CARGO_TARGET_DIR/zircon_editor_commandlet_tests`，并在 Windows 拒绝 C: 以外未受管的输出盘；这消除了测试在系统临时目录产生 C 盘产物的路径。

## 最低共享层根因

交接建立时，Plan16 M2 的 `zircon_editor/src/core/commandlet/runner.rs`、统一 `EditorLaunchArgs --run` 路由与 Plan08 命令注册表 CLI 投影尚未落地。当前源码已落实这条唯一入口；剩余根因是没有与当前快照哈希匹配的受管编译/单测证据，因此不能将源码存在误写为 CLI 验收通过。

## 架构修复验收

- 将唯一命令注册表收敛到 `zircon_editor::core::commands`，`migrate-assets` 以 `callable_from_remote=true` 的命令描述注册；CLI 不另建注册表。
- 在 `zircon_editor::core::commandlet::runner` 通过 Headless profile 调用 Runtime migration API，不创建窗口或物化工作台。
- `--run migrate-assets --project ... --dry-run|--apply` 使用统一参数解析；成功/任务失败/参数错误/能力缺失分别返回 0/1/2/3，并输出稳定 JSON 报告。
- 聚焦测试覆盖 dry-run 零写入、apply 成功、未知命令、互斥模式错误、能力缺失与 Runtime typed error 到退出码/JSON 的映射；随后回跑 Plan10 M2.3 旧样本迁移与幂等验收。

## 禁止临时方案

- 禁止在 `zircon_app`、Runtime 或二进制入口内增加第二套 commandlet 注册表、手写特判或 `migrate-assets` 专用旁路。
- 禁止让 App 反向依赖 Editor UI；App 只持有统一入口参数与进程宿主职责。
- 禁止保留旧 `--operation` 或其他兼容入口来冒充 `--run migrate-assets` 已接线。
- 禁止把 Runtime API/静态门通过写成 CLI 或端到端行为门通过。

## 产出记录与时间

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| Editor 16 M2 | `migrate-assets` 统一注册投影与无头 runner | `修复中-源码已接线-受管验证待执行` | 2026-08-15 | `EditorLaunchArgs` 已将 `--run` 路由至 Editor core 唯一注册表；runner 已覆盖 migration 与 capability/退出码 JSON 映射；fixture 输出已改为受管 `CARGO_TARGET_DIR`。尚未取得声明 Cargo 验收的当前快照证据，failure 保持 open。 |

## 修复结果与回传

Open state: `源码修复已完成，受管验证待执行`; no pass is claimed.
