---
handoff_kind: fixed
status: fixed
created_at: 2026-07-22
summary_slug: validation-copy-external-sibling-path-dependency
origin_plan: docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_editor/editor/02
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/workspace_copy.py
  - tools/session_coordinator/server.py
  - tools/session_coordinator/cli.py
tests:
  - validation copy materializes a pinned external Git sibling dependency
  - external repository dirty worktree is excluded from default pinned-HEAD input
  - restart preserves external mount identity and source hash evidence
resolved_at: 2026-07-23
---


# Coordinator01: validation copy 无法物化 workspace 外部 sibling path dependency

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md`
- 来源执行切片：Editor02 source-bound validation-copy Cargo gate
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：validation-copy 的 pinned 多 source-root 与完整 Cargo manifest closure 属于 Coordinator01 最低共享层。
- 来源会话：`editor02-message-inbox-backpressure-r3-20260722`
- 来源验证副本：`750454f293784c78970cdba435c947d2`，repo 内输入 18,839 路径
- 失败 job/run：`4edbb9ed2f13429eb7b9ca3c6ae3b7d0` / `961c11d453a14fb7bc7bdf2a7c1065b1`

## 失败现象与复现证据

focused Editor02 Cargo 在 1.7 秒内以 `exit_code=101` 终态，尚未进入 Rust 编译。workspace 解析链为：

`zircon_app -> zircon_first_party_runtime_catalog -> zircon_plugin_zr_vm_language_runtime -> zr_vm_rust_binding`

最终缺失路径：

`D:\cargo-targets\verify\750454f293784c78970cdba435c947d2\zr_vm\zr_vm_rust_binding\rust\zr_vm_rust_binding\Cargo.toml`

ZirconEngine 的插件 manifest 使用 `../../../../zr_vm/...` sibling path dependency。validation copy 只从 ZirconEngine repo-relative manifest 通过 pinned `git archive` 物化到 `job_root/source`，没有 canonical 输入描述 sibling repo，也无法在 `job_root/zr_vm` 建立受管 mount。

## 最低共享层根因

`WorkspaceCopyService.plan/materialize` 的 manifest authority 只有一个 `repo_root` 和一个 `head_commit`。路径规范化禁止逃出该 root，这是正确的安全边界，但当前模型缺少“多个 pinned source roots”数据结构，导致合法 workspace path dependency 只能在原 checkout 存在，无法在 source-bound validation copy 中复现。

外部仓库 `E:\Git\zr_vm` 当前 HEAD 为 `7e46440f5a5c3b1a976e3ef81164bc40de3948b6`，且 worktree 有大量未提交修改。直接 junction 到该 worktree 会破坏 immutable validation，不能作为修复。

## 架构修复验收

- validation-copy canonical request 支持外部 Git source descriptor：规范化 repo root、pinned commit/tree、job-root mount path、include roots 和 source hash。
- materializer 只从 pinned commit archive 提取外部输入；默认不读取外部 dirty worktree。需要 overlay 时必须有独立 owner/session/hash attribution，不得隐式吸收。
- mount path 必须位于 validation `job_root` 内，禁止 junction/symlink 逃逸；恢复、cleanup 和 orphan sweep 覆盖外部 mount。
- compatibility payload、validation evidence 与审计事件持久化外部 descriptor，重启后仍能证明相同输入。
- focused tests 覆盖 sibling Cargo path dependency、dirty external worktree 排除、缺失 commit/mount escape 拒绝、materialization rollback 与 restart recovery。
- async materialization 失败必须持久化 typed error code、stage、受限长度 detail 与失败路径；status/API/audit/restart 后可读取，rollback 不能吞掉诊断。
- materialize 在启动异步 archive 前应同步预检 untracked overlay 是否已有 Session attribution；缺失时直接返回 typed `validation_copy_unowned_path`，并明确 snapshot 不等于 attribution。

## 禁止临时方案

- 禁止把外部 dirty worktree 直接 junction 到 validation copy。
- 禁止用人工挑选 manifest 代替完整的 Cargo local path dependency closure。
- 禁止把 materialized 或 outer wrapper exit 0 当作 inner Cargo GREEN。

## 当前临时闭包

为继续 Editor02 验证，本次副本仅从外部仓库 pinned HEAD `7e46440f...` 执行 Git archive 到受管 `job_root/zr_vm`，并在后续 reservation `build_config` 中记录 SHA。该闭包不读取外部 dirty worktree，也不修改 ZirconEngine 产品源码；它是本次可复现输入补全，不是 Coordinator01 修复完成证据。

## 后续 materialization 可诊断性失败

Editor02 snapshot `879` 的 exact16 输入经 `snapshot preview` 复核为 16/16 无漂移。协调器随后为固定 ZirconEngine HEAD `f7627a0d2ba277be67e7b12abf2538b4d79d763c` 创建 validation-copy job `b50fdc2052f248ea8402a55d953655f3`，manifest 24,616 路径；job 经 `planned -> materializing -> failed` 终态，未创建 validation run、未启动 Cargo。

当前 `validation_copies` 仅保存 `status` 与 `materialization_started_at`，API 也只返回 `failed`，没有持久化 error code、失败路径或阶段。异步 worker 捕获异常后执行 rollback 并吞掉原始异常，执行者无法从受管证据判定是 archive、overlay、长路径、磁盘 I/O 还是 cleanup 失败。该事实属于本 failure 的同一 validation-copy ownership 范围；在 canonical 多 source root 之外，还需补齐 async materialization 的 typed terminal error 与 restart-visible audit evidence。

后续隔离重放定位了本组 job 的实际触发条件：snapshot `879` 已记录 exact16 hash，但 r6 的 `attributions` 仍为 0；snapshot 与 validation-copy overlay authority 是两个独立控制面。含 5 个 untracked 业务文件的 manifest 因而在 baseline archive 完成后触发 `validation_copy_unowned_path`，异步层只留下 `failed`。254 字符单文件写入探针与同服务 baseline extraction 均成功，排除了此前未证实的路径长度推测。

在重新取得 exact16 live lease并执行受管 `baseline attribute` 后，数据库得到 16/16 non-null attribution；同一 production dependency closure 的 job `663ea7bee09a4140a67a62915545e3fd` 随即成功 `materialized`，副本内 16 个 SHA-256 全部匹配 snapshot `879`。这解决了 Editor02 当前副本输入，不代表本 failure 已修复：async error 仍不可见，外部 sibling 多 source-root 合同仍不存在。

该副本后续 focused Cargo 证明“materialized”也不能替代 Cargo manifest closure 预检。reservation
`07be5ef3b1d24b79b478c596eee1a116` 合法成为 FIFO 队首后产生 job
`70e97f82a07b4456b7d1a1fb45ed830b` / run `8989e072185c4903b83c17f231eebb64`；进程自然终态
`released/exit_code=101/live_process_pids=[]`，0 tests，尚未进入 Rust 编译。Cargo 从 root workspace 解析
`zircon_app -> zircon_first_party_editor_catalog` 时缺失
`D:\cargo-targets\verify\663ea7bee09a4140a67a62915545e3fd\source\zircon_plugins\first_party_editor_catalog\Cargo.toml`
并以 os error 3 退出。原始 stderr 位于
`.codex/state/session-coordinator/cargo-runs/70e97f82a07b4456b7d1a1fb45ed830b/8989e072185c4903b83c17f231eebb64/stderr.log`。

这说明 canonical closure 还必须递归覆盖 workspace member 的 local path dependency manifest/source，而不是
只按顶层 package 选择“runtime production + plugin manifests”的人工集合。缺失的 first-party catalog 位于
同一 repo 内，和 `zr_vm` 外部 sibling 是两个层级但同一类 closure authority 缺口；修复验收需在 Cargo
启动前解析并校验完整 local path dependency graph，避免把 manifest-load failure 延迟到稀缺 CPU lane。

## 产出记录与时间

| 时间 | 状态 | 完成项目 | 证据与待办 |
|---|---|---|---|
| 2026-07-22 | `open_handoff_recorded` | Editor02 source-bound gate 的外部 sibling path dependency 缺口已复现、归因并路由 Coordinator01。 | job `4edbb9ed...`/run `961c11d...` 终态 exit101、0 tests；缺失 `job_root/zr_vm/.../Cargo.toml`。待 Coordinator01 实现多 pinned source roots、测试、独立复审和 failure -> fixed return。 |
| 2026-07-22 | `open_handoff_evidence_extended` | 补录 async materialization 的不可诊断终态，并通过隔离重放确认本组失败由 snapshot/attribution 双控制面遗漏触发。 | snapshot879 16/16 无漂移但初始 attribution=0；jobs `b50fdc20...`、`0ce11268...`、`0a1380f0...` 均在 0 Cargo 前 failed。显式 attribute 后 16/16 non-null，job `663ea7be...` materialized 且副本 hash 16/16。现有表/API 仍无 error payload，外部 sibling 多 source-root 仍缺失；failure 保持 open。 |
| 2026-07-22 | `open_manifest_graph_closure_missing` | materialized 副本首次受管 Cargo 证明 repo 内 local path dependency graph 仍不完整。 | reservation `07be...` -> job `70e97f82...` / run `8989e072...` 自然 `released/exit101/no PIDs/0 tests`；缺失 `zircon_plugins/first_party_editor_catalog/Cargo.toml`，manifest load 阶段 os error 3。Coordinator01 需在占用 CPU lane 前递归解析/校验 repo 内 local path dependency closure，并与外部 pinned source-root descriptor 一并持久化；Editor02 业务源码无归因。 |

## 修复结果与回传

- 根因：The validation-copy lifecycle lacked a durable multi-source Cargo dependency closure and failure-evidence boundary, so source-bound materialization could omit sibling inputs or delete the only nonzero diagnostic evidence.
- 架构修复：Validation copies now materialize pinned external Git sources, compute the complete local Cargo manifest closure, persist terminal diagnostics, and retain evidence-incomplete failures under immutable source identity.
- 验证：Current-source validation-copy gates passed 38/38; affected broad passed 153/153; failure-closeout deletion contract passed 17/17.
- 回传：Editor02 may create a fresh exact validation copy after the managed commit and controlled daemon reload; historical terminal runs remain immutable.
