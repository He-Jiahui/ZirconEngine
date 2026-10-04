---
handoff_kind: failure
status: open
created_at: 2026-07-11
summary_slug: migrate-assets-commandlet-registry
origin_plan: docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md
fixing_plan: docs/plans/zircon_editor/editor/16-cli-args-and-hub-integration.md
origin_child_dir: docs/plans/zircon_editor/editor/10
fixing_child_dir: docs/plans/zircon_editor/editor/16
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/core/commandlet/mod.rs
  - zircon_editor/src/core/commandlet/runner.rs
  - zircon_editor/src/core/commandlet/tests.rs
  - zircon_editor/src/core/commandlet/runner/capability_error_tests.rs
  - zircon_editor/src/core/commands/defaults.rs
  - zircon_editor/src/core/commands/registry.rs
  - zircon_editor/src/core/commands/descriptor.rs
  - zircon_app/src/entry/cli/launch_args.rs
  - zircon_app/src/entry/entry_runner/editor.rs
  - zircon_runtime/src/asset/migration/mod.rs
  - zircon_runtime/src/asset/migration/run.rs
  - zircon_runtime/src/asset/migration/options.rs
  - zircon_runtime/src/asset/migration/transaction.rs
tests:
  - cargo +1.94.1 test -p zircon_editor --lib --locked --jobs 1 --no-run --message-format short --color never
  - cargo +1.94.1 test -p zircon_editor --lib --locked --jobs 1 migrate_assets_is_registered_once_as_a_remote_callable_commandlet -- --exact --test-threads=1
  - cargo +1.94.1 test -p zircon_editor --lib --locked --jobs 1 migrate_assets_dry_run_reports_success_without_writing -- --exact --test-threads=1
  - cargo +1.94.1 test -p zircon_editor --lib --locked --jobs 1 migrate_assets_apply_writes_the_runtime_migration_result -- --exact --test-threads=1
  - cargo +1.94.1 test -p zircon_editor --lib --locked --jobs 1 commandlet_parser_reports_unknown_commands_and_mutually_exclusive_modes -- --exact --test-threads=1
  - cargo +1.94.1 test -p zircon_editor --lib --locked --jobs 1 commandlet_execution_reuses_the_descriptor_resolved_during_parsing -- --exact --test-threads=1
  - cargo +1.94.1 test -p zircon_editor --lib --locked --jobs 1 commandlet_reports_missing_capability_without_starting_a_ui_host -- --exact --test-threads=1
  - cargo +1.94.1 test -p zircon_editor --lib --locked --jobs 1 commandlet_maps_runtime_errors_to_failure_json -- --exact --test-threads=1
  - cargo +1.94.1 test -p zircon_editor --lib --locked --jobs 1 commandlet_fixture_paths_stay_below_the_managed_target_directory -- --exact --test-threads=1
  - cargo +1.94.1 test -p zircon_app --lib --locked --jobs 1 unified_launch_args_route_run_to_the_editor_core_commandlet -- --exact --test-threads=1
  - cargo +1.94.1 test -p zircon_app --lib --locked --jobs 1 unified_launch_args_preserve_json_parameter_errors_for_commandlets -- --exact --test-threads=1
  - cargo +1.94.1 test -p zircon_app --lib --locked --jobs 1 unified_launch_args_route_help_before_host_or_commandlet_construction -- --exact --test-threads=1
  - cargo +1.94.1 test -p zircon_runtime --lib --locked --jobs 1 --no-run --message-format short --color never
  - cargo +1.94.1 test -p zircon_runtime --lib --locked --jobs 1 unreadable_utf8_document_is_not_counted_as_a_parse -- --exact --test-threads=1
  - cargo +1.94.1 test -p zircon_runtime --lib --locked --jobs 1 invalid_toml_document_records_the_attempted_parse -- --exact --test-threads=1
  - cargo +1.94.1 test -p zircon_runtime --lib --locked --jobs 1 rejected_retired_reference_retains_its_resolver_visit -- --exact --test-threads=1
  - cargo +1.94.1 test -p zircon_runtime --lib --locked --jobs 1 later_reference_failure_preserves_all_preceding_document_visits -- --exact --test-threads=1
  - cargo +1.94.1 test -p zircon_runtime --lib --locked --jobs 1 migration_reports_an_unsynced_commit_point_as_pending_recovery -- --exact --test-threads=1
  - cargo +1.94.1 test -p zircon_editor --lib --locked --jobs 1 -- --test-threads=1
  - cargo +1.94.1 test -p zircon_app --lib --locked --jobs 1 -- --test-threads=1
  - cargo +1.94.1 test -p zircon_runtime --lib --locked --jobs 1 -- --test-threads=1
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

## 当前源码调用链与快照（2026-09-25）

当前生产调用链已从入口参数贯穿唯一命令注册表、无头 runner，再落到 Runtime migration API：

```text
zircon_app/src/entry/cli/launch_args.rs
  -> zircon_app/src/entry/entry_runner/editor.rs
  -> zircon_editor/src/core/commandlet/{mod.rs,runner.rs}
  -> zircon_editor/src/core/commands/{defaults.rs,registry.rs,descriptor.rs}
  -> zircon_runtime/src/asset/migration/{mod.rs,options.rs,run.rs,transaction.rs}
```

`defaults.rs` 只注册一个 `migrate-assets` 描述符（`callable_from_remote=true`、typed headless route/name、payload schema）；`runner.rs` 在解析阶段解析该描述符并以 Headless capability 执行，`entry_runner/editor.rs` 在创建 GUI host 前处理 `--run` 并将报告映射到 0/1/2/3；Runtime `run.rs` 负责 dry-run 不写入、apply 事务提交/恢复和 typed error。没有重新出现 `zircon_app` 本地注册表、旧 `--operation` 兼容旁路或窗口物化。

本次静态核对冻结了 13 个直接生产/测试路径的 SHA-256（仅文档文件归本 Session；源码不作编辑）：

| 路径 | SHA-256 |
|---|---|
| `zircon_editor/src/core/commandlet/mod.rs` | `0de51fc3cdb911edfc0b5b908e20d9725e9eec0dab558f529049f19dca8f0fb5` |
| `zircon_editor/src/core/commandlet/runner.rs` | `51a519842bcf5d1c008f5434eb8cfeccd69fd89134cc6fca13d7fd462b4a0a97` |
| `zircon_editor/src/core/commandlet/tests.rs` | `ddef58156dd36e4acfdd42280fd30af28f8c507319a12d501a9cbe2bb07947d7` |
| `zircon_editor/src/core/commandlet/runner/capability_error_tests.rs` | `9681dfedc92cf1ff1570a2ff38755837794bd23f2cd86617b16fd8b982643375` |
| `zircon_editor/src/core/commands/defaults.rs` | `96367d04650ef79ec7eb9ea28172bb7b6052f6598e300172839732a72607eb27` |
| `zircon_editor/src/core/commands/registry.rs` | `8ab915746aa90ad9dd212cab3c26e1e377f13dbeeb5608b731328922a039c083` |
| `zircon_editor/src/core/commands/descriptor.rs` | `73bfe252b9f6ab4f7467eea9943a31f510802230cd356a6e5e4da7a5a24c7ca3` |
| `zircon_app/src/entry/cli/launch_args.rs` | `4b0caa81616c3a1a5572419ba3fe6274b30be17cca957577949e4c6d037bfd9a` |
| `zircon_app/src/entry/entry_runner/editor.rs` | `9f179d2b225838f2a9de8fba02c5f7c1e40adb9b745bfc2195e80f0c4147c86b` |
| `zircon_runtime/src/asset/migration/mod.rs` | `16c5cdb211ba75bc32558ee3a4dffab561e7dd891c7d36ea1c5651f94d2c4773` |
| `zircon_runtime/src/asset/migration/run.rs` | `a67f5d57b20825582a8e475a56be0835ab2009b4090ba113be346fdca6c6d3f0` |
| `zircon_runtime/src/asset/migration/options.rs` | `c6f951811ec99e0c84e3fa62d8094348e158435d5edc37bd8b0a37576175c84c` |
| `zircon_runtime/src/asset/migration/transaction.rs` | `de40f88dfbe81a9c6824d53d1075fa72569eeae38668b4bf91f96dd6aef30714` |

`git status` 显示 `launch_args.rs`、`entry_runner/editor.rs`、`commandlet/tests.rs`、`commands/{defaults.rs,descriptor.rs,registry.rs}` 和 `migration/{run.rs,transaction.rs}` 存在其他 Session 的既有 dirty 修改；这些路径未被本 Session 取得 lease，也未吸收或重写。`E:\Git\zr_vm` 仍为外部 dirty 工作树，因而 Windows 受管 Cargo/product gate 继续保留为 pending。

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
| Failure 队列复核 | 当前调用链与验收快照 | `等待受管动态验证` | 2026-09-11 | 会话 `failure-roll-01a084c8-editor16-migrate-assets` 在 `c37155ba304740b3762b20585f77fb53a6da47fb` 冻结快照 `3397`：命令描述、唯一注册表、无头 runner、CLI 路由和稳定退出码映射仍完整。对冻结路径执行 `rustfmt --check --edition 2021` 时，工作树中既有且未归属的格式差异使检查返回非零，未将其写为 GREEN；声明的 Windows 受管 Cargo 验收因外部 `E:\Git\zr_vm` 工作树脏状态尚不能准入，未重复提交同类请求。 |
| Failure roll r2 | 当前源码链静态核对与精确验收命令封存 | `源码链已核对-动态验证待协调器窗口` | 2026-09-25 | Session `failure-roll-01a084c8-editor16-migrate-assets-r2` 通过路径存在性与源码过滤核对，封存 13 个当前 SHA-256 和 8 个 commandlet/app、5 个 Runtime 迁移 focused filters；未把源码存在、dirty foreign diff 或队列回执写成动态通过。 |
| Failure handoff schema | 全库 handoff 结构复验 | `通过` | 2026-09-25 | `.codex/skills/zircon-project-skills/handle-plan-failure-handoffs/scripts/validate_plan_failure_handoffs.py --repo-root E:\Git\ZirconEngine` → `Validated 828 handoff artifact(s): 0 errors.`；该结构证据不替代本 failure 的 Cargo 动态验收。 |

## 修复结果与回传

Open state: `源码修复已完成，受管验证待执行`; snapshot `3397` retains the prior source hashes and r2 freezes the current chain above. No static or dynamic Cargo pass is claimed; fixed return and closeout remain pending until the exact Windows `--locked` commands run against the same attributed snapshot.

### 2026-09-26 successor intake (failure-roll-01a084c8-editor16-migrate-assets-r3)

- The stale r2 lifecycle was cancelled through the coordinator after its
  heartbeat expired with no active lease. Successor
  `failure-roll-01a084c8-editor16-migrate-assets-r3` now owns only this
  failure record. Ownership transfer fingerprint is
  `4f179cbcb215703f52d3dfbb088acac45bd2b727a9462c8368b9e8472633d634`, and
  pre-review snapshot `3922` sealed the record at SHA
  `28145b087dcf13f17130f569cd3c9eae40904a1bdc2e7c62907cfafca96484b8`.
- The thirteen-path source chain was rehashed without editing or claiming any
  source. Twelve paths still match the r2 manifest; the current
  `zircon_app/src/entry/entry_runner/editor.rs` is instead
  `9e32739b7b7c7e49e8b731bb443999f46d7bbd88d64eae8bc389af08f8e63c08`,
  differing from the historical `9f179d2b225838f2a9de8fba02c5f7c1e40adb9b745bfc2195e80f0c4147c86b`.
  That drift is foreign/unowned and makes the prior source ticket and r2
  manifest non-reusable for dynamic acceptance.
- The static command-chain evidence and ticket
  `fff2517ebd0349ada225c016b8b6ca97` remain historical parse evidence only;
  the external `E:/Git/zr_vm` dirty admission blocker, fresh exact commandlet
  and Runtime filters, full Editor/Runtime gates, independent review, canonical
  `fixed-*` return, closeout, and WeCom notification remain pending. The
  failure stays open and no Cargo pass is claimed.

### 2026-09-26 independent successor review receipt

- Read-only reviewer `/root/review_editor16_migrate_assets_final` rehashed all
  thirteen listed paths. Twelve historical r2 hashes still match; the current
  `zircon_app/src/entry/entry_runner/editor.rs` hash is
  `9e32739b7b7c7e49e8b731bb443999f46d7bbd88d64eae8bc389af08f8e63c08`,
  explicitly differing from the sealed
  `9f179d2b225838f2a9de8fba02c5f7c1e40adb9b745bfc2195e80f0c4147c86b`.
- The stale r2/no-lease cancellation, successor fingerprint
  `4f179cbcb215703f52d3dfbb088acac45bd2b727a9462c8368b9e8472633d634`, and
  foreign dirty provenance are consistent. No source edit or ownership
  absorption occurred. C/I/M: **Critical=0 / Important=0 / Moderate=0**.
- The external `E:/Git/zr_vm` blocker, fresh managed commandlet/Runtime
  focused tests and full Editor/Runtime gates, canonical return, closeout, and
  WeCom remain pending. The failure stays open; no dynamic pass is claimed.
