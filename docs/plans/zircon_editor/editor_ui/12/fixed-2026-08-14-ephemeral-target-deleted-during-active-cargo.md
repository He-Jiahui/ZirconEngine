---
handoff_kind: fixed
status: fixed
created_at: 2026-08-13
summary_slug: ephemeral-target-deleted-during-active-cargo
origin_plan: docs/plans/zircon_editor/editor_ui/12-unreal-magicavoxel-zui-design-convergence.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_editor/editor_ui/12
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/cargo_jobs.py
  - tools/session_coordinator/artifact_governance.py
  - tools/session_coordinator/cleanup_deletion.py
  - tools/session_coordinator/server.py
  - tools/session_coordinator/tests/test_cargo_jobs.py
  - tools/session_coordinator/tests/test_artifact_governance.py
  - tools/session_coordinator/tests/test_server.py
tests:
  - python -X dev -W error::ResourceWarning -m unittest tools.session_coordinator.tests.test_cargo_jobs -v
  - python -X dev -W error::ResourceWarning -m unittest tools.session_coordinator.tests.test_artifact_governance tools.session_coordinator.tests.test_windows_tree_delete -v
  - python -X dev -W error::ResourceWarning -m unittest tools.session_coordinator.tests.test_server -v
  - powershell -File .codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_editor -Ephemeral -SkipBuild -LibTests -TestFilter workbench_toolbar_breakpoints -VerboseOutput
resolved_at: 2026-08-14
---


# Tooling01 failure handoff: ephemeral target deleted during active Cargo

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor_ui/12-unreal-magicavoxel-zui-design-convergence.md`
- 来源执行切片：M3 工作台壳层与工具栏断点 managed Windows validation gate
- 来源执行 Session：`editor-ui12-m3-workbench-shell-v2-20260813`
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：Tooling01 独占 coordinator-managed ephemeral Cargo lane 的创建、运行态投影、release 与目录清理；UI12 不能在 Editor 源码或验证命令中补偿活动 target 被删除。

## 失败现象与复现证据

在 artifact audit 为空、无活跃 `cargo`/`rustc`、M3 snapshot `1668` 为 18/18 `would_change=false` 后，以下官方命令连续两次在约 60 秒的依赖编译阶段失败：

```powershell
& .\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_editor -Ephemeral -SkipBuild -LibTests -TestFilter workbench_toolbar_breakpoints -VerboseOutput
```

第一次 job `6fa07d311eb64e0396633040d821f607` 使用 `D:\cargo-targets\zircon-engine\ephemeral\test\6fa07d311eb64e0396633040d821f607`。`cargo.start_accepted` event `141362` 为 2026-08-13 16:49:09 +08，多个依赖随后统一因 `.fingerprint`、`debug/deps` 和 build output 路径不存在而报 `os error 3`；job 16:50:08 记为 exit 1，16:50:12 release，`cleanup.ephemeral_lane_deleted` event `141363` 为 16:50:13。

Tooling01 随后对相邻的 terminal projection / collector 竞争条件形成修复，`python -X dev -W error::ResourceWarning -m unittest tools.session_coordinator.tests.test_cargo_jobs -v` 报告 52/52 通过并回传 `cargo-run-terminal-projection-before-release`。但正在服务的 coordinator daemon 自 2026-08-11 启动，UI12 未获授权重启或替换该共享服务。

在 Tooling 复测进程与 handoff return 自然结束后，UI12 再次确认 artifact audit 为空、无活跃 Cargo 进程、HEAD 未变、M3 snapshot 仍为 18/18，并原样重试。第二次 job `6122d9bdd7534859b7a2fc2e9e446eba` 使用 `D:\cargo-targets\zircon-engine\ephemeral\test\6122d9bdd7534859b7a2fc2e9e446eba`，17:05:42 +08 启动，约 17:06:05 退出；`autocfg`、`smallvec`、`cfg-if`、`windows-link`、`memchr`、`unicode-ident`、`arrayvec`、`log` 等并行 `rustc` 统一报告 target 下的 `debug/deps` 已不存在。目标 `zircon_editor` 源码与 `workbench_toolbar_breakpoints` 测试执行数均为 0。

两次失败都发生在 UI 源码编译之前，不能作为 M3 red/green 或产品行为证据。

同日 17:16:42 +08，独立的 Runtime09 Session `runtime09-shell-content-transaction-20260813` 通过 coordinator job `2da2b5924e964bdaa3fe1bf0a8564c88` 运行 drawer token focused test，使用另一个 ephemeral target。该 run 已编译到 `syn`、`indexmap`、`rayon-core`、`khronos_api` 等依赖后，于 17:17:26 +08 以 exit 101 结束；`libm`、`xml-rs`、`foldhash`、`anyhow`、`crossbeam-epoch`、`serde_core` 等同时报告 target 下 `.fingerprint` 不存在，build script 也因工作路径消失而 panic。job release 后目标目录不存在。这个跨 Session、跨 test filter 的第三次复现排除了 UI12 wrapper 参数或 `workbench_toolbar_breakpoints` 本身是删除触发者。

## 最低共享层根因

已确认的最低根因不是 UI12 wrapper 或主协调器 job 账本，而是同机另一个指向
`E:\zircon-profiles\runtime09-rail-snapshot-worktree`、监听 6519 的 schema 58
只读非主工作树协调器。该实例与主协调器共享 `D:\cargo-targets`，却使用独立数据库；
旧 maintenance loop 即使 `mode=read_only` 仍每 30 秒运行 unmanaged artifact
cleanup。它看不到 6518 的活跃 job，因此把整个
`D:\cargo-targets\zircon-engine` 当作未托管目录直接 `shutil.rmtree`。

旧实例事件 863/864 在 2026-08-14 03:02:34 +08 对该父目录记录
`artifact.unmanaged_delete_started/failed`，正落在 6518 job
`46a43e80972447ec961ed81e6ac06126` 的 03:02:24-03:02:40 活跃窗口内；
事件 865/866 又于 03:03:05 记录 started/deleted。主实例没有提前删除事件，因而
此前只能看到 Cargo 的 `os error 3` 和 release 后的正常清理。这一跨实例账本证据直接
确定了删除者。

## 架构修复验收

- 当前共享 coordinator daemon 明确加载含 collector ownership 修复的 current source，部署或重启过程由 Tooling01 所有者执行并记录服务身份。
- 非 main/read-only 协调器在启动与周期维护中不得执行 Cargo orphan/reservation recovery、Cargo target cleanup、artifact cleanup、validation-copy recovery 或 validation ticket worker；只读必须覆盖所有共享文件系统 mutation，而不只是 HTTP command admission。
- 对 locally collected job，任何 terminal reconciliation、release、retention 或 maintenance cleanup 均不得在 root supervisor/process tree 实际退出且 owning collector 完成 finish 之前删除 target。
- 增加确定性回归，证明处于活动写入阶段的 ephemeral target 不会被 status polling、maintenance tick、artifact cleanup 或 release observer 删除；测试需覆盖 Windows file lifetime 与事件顺序。
- `python -X dev -W error::ResourceWarning -m unittest tools.session_coordinator.tests.test_cargo_jobs -v` 全量通过，无 `ResourceWarning` 与 teardown 文件占用错误。
- 原始 UI12 官方命令原样运行，实际编译 `zircon_editor` 并执行 `workbench_toolbar_breakpoints`；不得只以协调器单测替代 upward gate。
- 若原命令越过历史删除窗口并到达无关的当前源码编译错误，必须将 Coordinator 基础设施修复与上层产品测试未执行分别记录，不得把无关编译错误改写成 Coordinator 回归。
- 修复完成后用 coordinator `failure return` 回传 fixed artifact，附 daemon/current-source identity、managed job ID、测试执行数与结果。

## 禁止临时方案

- 不得让 UI12 去掉 `-Ephemeral`、改用未声明 target、手工保留 target 目录或绕开 coordinator-managed validation。
- 不得通过延长 sleep、降低并行度、吞掉 `os error 3`、自动无限重试或把 Cargo 失败改写为通过来规避竞争条件。
- 不得由 UI12 重启共享 coordinator、删除外部 artifact、finish/release 外部 job，或编辑 Tooling01 源码。
- 不得把协调器基础设施失败计作 M3 源码、测试或产品截图验收证据。

## 修复结果与回传

- 根因：A schema-58 read-only coordinator in a non-main worktree shared D:\\cargo-targets with the main daemon but used an independent database; its maintenance loop still ran artifact cleanup and deleted D:\\cargo-targets\\zircon-engine while the main job was active.
- 架构修复：Read-only coordinator startup and maintenance now suppress all shared-artifact mutations: Cargo orphan/reservation recovery, cleanup recovery and eviction, artifact cleanup, validation-copy recovery, validation ticket execution, and interrupted benchmark-grant/result recovery that can terminate a durable process identity. The deployed schema-60 daemon also uses durable deletion reservations and managed-overlap gates.
- 验证：Deterministic RED/GREEN server test proves zero mutation calls for non-main startup and one maintenance loop, including zero benchmark-grant reconciliation and validation-result recovery. Independent review first reproduced the omitted grant recovery as one forbidden call, then the guarded implementation passed server 60/60; Cargo+cleanup 99/99, artifact+Windows deletion 24/24, and workspace-copy 58/58 passed with ResourceWarning fatal. Current-source schema-60 instance `47b2e20f3ad243db951cb0b67f5995d0` (PID 28800) is healthy after rollout. Original job 1f1fc213310f4fcaa92b90811ac348f9 compiled for 202 seconds without path loss; delete began only after release. Product test then stopped at unrelated current zr_rhi_wgpu compile errors, so UI test count remains zero.
- 回传：Read-only/non-main coordinators can no longer mutate shared Cargo or validation artifacts; the original ephemeral lane survived until managed release and cleanup.
