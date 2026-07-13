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

Plan10 M2.3 已实现 `zircon_runtime::asset::migration`，但当前编辑器可执行体尚不能合法执行：

```text
zircon_editor --run migrate-assets --project <project-root> --dry-run
zircon_editor --run migrate-assets --project <project-root> --apply
```

曾尝试的 `zircon_app/src/commandlet` 本地解析/分派实现已删除：它建立了第二套注册事实，绕过 Plan16 规定的 `zircon_editor::core::commands` 唯一注册表，也无法诚实满足 0/1/2/3 退出码与 capability 门禁。当前保留该入口会让 Runtime 能力看似可用、实际却与统一命令架构分叉，因此 Plan10 不声明 CLI 验收通过。

`zircon_app/src/entry/cli` 同样尚不存在，因此不列入可机器验证的 `related_code`；未来 Plan16 落地合法进程宿主后再记录真实 owner。正文保留该缺口与验收要求，不以虚假路径或兼容目录把 open handoff 伪装为已完成。

## 最低共享层根因

Plan16 M2 的 `zircon_editor/src/core/commandlet/runner.rs`、统一 `EditorLaunchArgs --run` 路由与 Plan08 命令注册表 CLI 投影尚未落地。Runtime 任务实现可以独立完成，但缺少上层唯一合法的进程入口 owner；该缺口不能由 Runtime 或 `zircon_app` 自建兼容入口修补。

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
| Editor 16 M2 | `migrate-assets` 统一注册投影与无头 runner | `待修复-Runtime能力已就绪-CLI入口未接线` | 2026-07-11 | Plan10 M2.3 已提供 `zircon_runtime::asset::migration` dry-run/apply 与事务 API；错误的 `zircon_app/src/commandlet` 第二注册表实现已删除，当前无合法 `--run migrate-assets` 入口，须由 Plan16 M2 接入 Plan08 唯一注册表。 |

## 修复结果与回传

Open state: `待修复`; no pass is claimed.
