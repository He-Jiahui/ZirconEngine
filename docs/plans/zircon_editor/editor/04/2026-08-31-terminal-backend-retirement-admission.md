---
related_code:
  - zircon_editor/src/core/play/controller.rs
  - zircon_editor/src/core/play/controller/runtime_ownership.rs
  - zircon_editor/src/core/play/embedded_backend/mod.rs
  - zircon_editor/src/core/play/process_backend/child.rs
  - zircon_editor/src/core/play/process_backend/mod.rs
  - zircon_editor/src/core/play/snapshot/store.rs
  - zircon_editor/src/core/play/live_link.rs
  - zircon_editor/src/core/play/edit_protection.rs
  - zircon_editor/src/core/play/pending_edits
  - zircon_editor/src/core/editing/operation
  - zircon_editor/src/core/gateway/handle.rs
  - zircon_editor/src/core/play/tests.rs
  - zircon_editor/src/core/play/tests/terminal_backend_retirement.rs
  - zircon_editor/src/ui/host/editor_host_event_controller.rs
  - zircon_editor/src/ui/host/editor_host_event_controller/play_viewport_pick.rs
  - zircon_editor/src/ui/host/editor_host_event_controller/play_world_replacement.rs
reference_sources:
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Classes/Editor/EditorEngine.h
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/PlayLevel.cpp
plan_sources:
  - docs/plans/zircon_editor/editor/00-editor-architecture-overview.md
  - docs/plans/zircon_editor/editor/04-pie-and-simulation.md
  - docs/plans/engine-code-structure-convention.md
status: implementation_complete_validation_pending
---

# Editor04 terminal backend retirement admission

## 当前源码重审

`PlaySessionController::complete_runtime_stop` 在 backend 已停止时释放 edit protection，并把
mode 迁移到 `Edit` 或 `CleanupFailed`。源码重审发现，旧实现把“下一次 session 的 backend /
plugin 配置”与“当前 session 的实际 owner”混为一处：运行中替换配置后，stop、rollback 和
cleanup retry 可能操作错误 provider。进程与嵌入式 backend 还存在 terminal cleanup 失败后
丢失 child、lease 或 snapshot owner 的分支。

准入面也有独立缺口：`request_play` 原先只检查 pending edit decision 和陈旧 play gateway，
没有检查 terminal owner。结果是 UI 已看到 `Edit` 后可以再次调用 `backend.start`，使新
Play 实例与尚未退役的旧 owner 发生生命周期重叠。操作路由与 gateway detach 的复审又
发现 target authority 双源和 identity compare/replace 非原子窗口，均需要在同一生命周期
闭包内 fail closed。

## 参照与裁决

Unreal `UEditorEngine::EndPlayMap` 以 `bIsEndingPlay` 防止 teardown 重入，随后完成 PIE world
的 `BeginTearingDown`、`EndPlay`、`CleanupWorld`、root 释放，最后才清空 `PlayWorld`。
Zircon 不复制 Unreal 的对象模型，但采用同一生命周期裁决：用户可见 mode 与物理 runtime
owner 分开建模；新的 Play 只能在旧 backend terminal retirement 完成后准入。

该修复不是局部性能优化，不改变 stop/retire 的渐近复杂度，也不增加稳定帧工作。它收紧
Editor04 的结构不变量：

1. `Playing -> Edit` 可以先于后台 owner 最终退役，以便 UI 和 edit protection 及时恢复。
2. active/terminal session 必须持有启动时的 exact backend 与 plugin activation owner；
   后续配置替换只影响下一次 session。
3. terminal owner 非空时，`request_play` 必须在 plugin activation、edit protection 和
   `backend.start` 之前 fail closed；只有 exact owner 清理完成后才能重新启动。
4. embedded lease retire、process tree termination、output reader join 与 snapshot cleanup
   任一步失败都不能把 backend 恢复成 `Idle` 或丢失唯一可重试 owner。
5. operation registration 是 edit target 的唯一权威；gateway identity compare 与 detach
   必须在同一个 replacement gate 内原子完成。

## 实现

- Controller 引入 `PlaySessionOwnership::{Active, Terminal}`，保存启动时 exact backend /
  activation；start rollback、normal stop、terminal poll 与 cleanup retry 统一使用该 owner。
- `PlayBackend::start` 使用 `PlayBackendStartFailure` 显式报告 `retirement_pending`；即使 startup
  尚未进入 Playing，只要 process child、embedded lease 或 snapshot cleanup owner 仍存活，
  controller 就保留启动时 exact backend，拒绝新会话并经 `retire_terminal_backend` 重试。
- `PlaySnapshotStore::materialize` 使用 typed failure 携带写入失败后仍无法删除的 exact snapshot
  owner；embedded/process 都把该 owner 接入既有 `CleanupPending`，首次删除失败不会开放下一次
  Play 准入，并由后续 retirement retry 完成清理。
- `request_play` 在任何 terminal owner 尚未释放时返回 typed invalid transition；新增
  folder-backed 测试覆盖 backend retirement、配置切换和 plugin rollback/retry。
- Embedded backend 增加 `CleanupPending`，lease retire 成功但 snapshot cleanup 失败时保留
  scene；下一次 retire/start 先重试清理。Process child 将 tree 显式建模为
  `Persistent/Untracked/Terminated`，自然退出、显式 stop、startup rollback 和 Drop 四条路径
  都保留或最终回收 child/tree/output/snapshot；tree termination 失败时不丢 owner、不无条件
  `wait()`，未确认 inherited pipe 关闭时不阻塞 join reader。
- Deferred operation 携带 registration-owned `EditOperationTarget`，route/queue 对调用方 target
  做一致性校验；pending queue 冻结本轮 batch、保留 in-flight 容量，callback panic 先回队再
  unwind。
- Gateway 新增 replacement gate 内的 identity-qualified detach，`PlayDomainLink` attach/detach
  收口为 crate 内部生命周期 API；Play input 与 SIE camera 同样以采样到的 gateway identity
  调用 generation-qualified event dispatch，replacement race 时 fail closed。
- 生产 `PlaySessionController`/host 不再暴露手工 attach 或无 identity detach；UI 与 core 测试
  改用 test-only attachable backend，经真实 `request_play(Play/Simulate)` 创建网关和 owner，
  shutdown/selection/world-replacement 测试不再依赖“先挂载、后启动”的旧架构语义。
- `PlayTransitionReport.changed` 在首次进入 `CleanupFailed` 时为 true；模块文档与静态契约已
  同步到新 target owner、budgeted apply/discard API 和 cleanup 语义。

## 性能边界

本切片未声称性能优化或功耗收敛。当前仍有两个已登记的结构性性能 Failure：

- `failure-2026-07-22-play-snapshot-transition-main-thread-stall.md`：snapshot 序列化/落盘、
  process spawn 与 foreign backend/plugin 调用仍跨 UI transition gate，必须等待 Runtime11
  generation-owned immutable artifact/ticket 后再按 1/64 MiB/1 GiB 规模剖析和改造。
- `failure-2026-07-22-play-pending-edit-unbounded-queue.md`：typed retention、分页与 apply budget
  已进入当前源码，但单项 payload 上限、100k edit/RSS/p95 产品剖析和 managed validation
  未完成，failure 继续保持 open。

因此本切片只接受资源所有权与故障恢复正确性，不把静态 source guard 当作性能证据。

## 验证状态

- 精确 `rustfmt --edition 2021 --config skip_children=true`：通过。
- 精确 `git diff --check`：通过；只有仓库既有 LF/CRLF 提示。
- `python tools/tests/test_editor04_play_edit_protection_contract.py`：4/4 通过。
- 13 个 `PlayBackend` 实现的 typed start signature 扫描：13/13 通过；旧 production
  `attach_play_gateway` / 非 identity `detach` 调用扫描为 0。
- managed Cargo：待协调器执行；在对应 source manifest 的聚焦门终态前，本切片不是
  accepted milestone，不提交 commit、不发送里程碑企微。

## 产出记录与时间

尚无 accepted milestone 记录；待 exact source manifest 的 managed Cargo、独立复审与协调器
milestone gate 全部通过后，仅追加一条最终产出记录。
