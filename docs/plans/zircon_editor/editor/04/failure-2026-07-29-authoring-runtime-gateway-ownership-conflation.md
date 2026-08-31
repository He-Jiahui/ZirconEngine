---
handoff_kind: failure
status: open
created_at: 2026-07-29
summary_slug: authoring-runtime-gateway-ownership-conflation
origin_plan: docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
origin_workflow_node: M2
fixing_plan: docs/plans/zircon_editor/editor/04-pie-and-simulation.md
origin_child_dir: docs/plans/zircon_editor/editor/01
fixing_child_dir: docs/plans/zircon_editor/editor/04
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/ui/host/editor_host_event_controller.rs
  - zircon_editor/src/core/gateway/contract.rs
  - zircon_editor/src/core/gateway/session.rs
  - zircon_editor/src/core/editing/authoring_world.rs
  - zircon_editor/src/ui/retained_host/app.rs
tests:
  - attach a SessionGateway while the edit domain remains readable and mutable through its authoring facade
  - SessionGateway with_world and with_world_mut remain RequiresSerializedAccess
  - play attach and detach route only the play domain and restore edit-domain selection and undo access
  - cargo test -p zircon_editor --lib --locked
---

# Editor04: authoring/runtime gateway ownership is conflated

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md`
- 来源执行切片：M2.4 gateway UI deep-path cutover
- 修复责任计划：`docs/plans/zircon_editor/editor/04-pie-and-simulation.md`
- 交接原因：Editor01 可守卫 `core/ -> ui` 单向依赖，但不能把 UI 当前的 authoring world 直接替换为 runtime session transport。Edit/Play 双域、play attach/detach 与 gateway 生命周期属于 Editor04 `PlaySessionController`/`PlayDomainLink` 的权威。

## 失败现象与复现证据

1. `EditorHostEventController::set_runtime_gateway` 将传入的 runtime transport 直接 `replace` 到 `EditorContext.gateway()`。
2. `EditorRuntimeGateway` 对 `with_world` 与 `with_world_mut` 的默认定型行为是 `RequiresSerializedAccess`；`SessionGateway` 不应获得 local-world 借用旁路。
3. 原始审计时 Workbench 仍经 `EditorWorldSlot` 直接持有 `LevelSystem`，用于 edit-domain selection、scene command、save 与 viewport input。

因此，若 M2.4 将 UI 的 authoring world 调用机械改为 `EditorContext.gateway()`，在 runtime session attach 后该 facade 会变成 `SessionGateway`，所有 edit-domain borrowed-world 操作都会被拒绝。保留 `EditorWorldSlot` 作为第二条 UI 私有 world 路径又违反 Editor01 的 hard cutover 目标。

当前源码已删除 `EditorWorldSlot`，并由 `core/editing/authoring_world.rs` 的 `EditorAuthoringWorld` 持有稳定 edit-domain facade；play attach/detach 回归也覆盖 authoring access 保持可用。该事实只收敛了原始旁路，不替代本记录要求的 managed validation、独立复审和 fixed return，因此状态仍为 open。

## 最低共享层根因

当前单一 `EditorContext.gateway()` 同时被当作长期 authoring-world facade 与可替换 runtime-session transport。两者生命周期不同：edit world 在编辑器会话内稳定存在，play/session gateway 是可 attach、detach、destroy 的域资源。没有显式 domain owner 时，replace 语义会吞掉 authoring access，或迫使 UI 保留 `CoreHandle`/`LevelSystem` 旁路。

## 架构修复验收

- Editor04 为 `WorldDomain::{Edit, Play(PlayInstanceId)}` 定义 typed gateway routing：稳定的 edit-domain facade 不随 runtime/play attach 被 replace；`PlayDomainLink` 只持 attached play gateway 与其生命周期凭据。
- `PlaySessionController`/`PlayDomainLink` 明确暴露当前 attached play gateway，attach/detach/stop 仅改变 play-domain routing，不改变 edit-domain authoring gateway、selection、undo 或 save owner。
- Session-backed gateway 继续返回 `RequiresSerializedAccess` 给 direct borrowed-world API；不得为绕过迁移给 `SessionGateway` 添加 local `LevelSystem`、`CoreHandle` 或伪造 world clone。
- UI scene/edit consumers 按 domain 取得 facade；M2.4 最终 guard 才能删除 `src/ui/**` 中所有 `LevelSystem`/`CoreHandle` 直用，且 gateway implementation 是唯一允许的 runtime-owner 深路径。
- 回归覆盖：runtime session attach 后 edit-domain selection/read/write 仍可用；play attach 使用 play gateway；detach/stop 后 edit-domain state 不变；SessionGateway 借用拒绝不回归；编辑世界与 play 世界保持零污染隔离。

## 禁止临时方案

- 不得把 SessionGateway 的 `with_world` 改成借用 editor 本地世界，或在 SessionGateway 内缓存 authoring `LevelSystem`。
- 不得保留 `EditorWorldSlot`、`CoreWeak`、raw `CoreHandle` 作为 UI 侧兼容旁路。
- 不得通过 attach 时复制 authoring world 到第二个长期 UI state 绕开 domain routing；Play 只接受 Editor04 规定的副 session/DTO 注入和生命周期管理。

## 修复结果与回传

Open state：`source_hardcut_complete / managed_validation_and_fixed_return_pending`。typed dual-domain routing、Edit 域命名边界、focused source contract 与 UI raw world-owner guard 已落地；尚无 immutable Cargo、产品零污染验收或 fixed return，因此本记录仍保持 open。

## 产出记录与时间

| 时间 | 状态 | 完成项目与证据 |
| --- | --- | --- |
| 2026-08-27 CST | `authoring_gateway_name_hardcut_source_complete / managed_validation_pending` | 复读 current source 与 Unreal `FWorldContext` 的 `Editor`/`PIE` 分域后，确认 Zircon 数据 owner 已分开，但 `EditorContext.gateway()` / builder `with_gateway()` 仍保留可替换通用 transport 的旧语义。现已无兼容别名硬切为 `authoring_gateway` / `with_authoring_gateway`，Edit 路由显式消费 authoring owner，Play 仍只从 `PlaySessionController::play_gateway(instance)` 取得；旧 context/builder accessor 与调用点扫描为 0。新 Python 合同 2/2 先 RED 后 GREEN，联跑 Editor04 现有 Play 合同合计 9/9 GREEN；scoped rustfmt 通过。Editor03 旧源码字符串合同仍有 7 项与既有 App/EditorState/Gizmo 重构漂移失败，未为迎合其恢复旧架构；未运行 Cargo，failure 继续 open。 |
| 2026-08-26 CST | `embedded_session_ownership_source_complete / viewport_and_managed_acceptance_pending` | App-owned factory 从重校验的同一 BuildSet 创建独立 runtime-profile session，Editor 仅持 gateway + opaque retirement lease；authoring `editor` profile 不进入 play domain。start 自动 attach，terminal 统一 consumer stop -> backend stop -> identity detach -> lease retire，outstanding owner/destroy failure可重试；欢迎页后开工程同样已有 embedded backend。相关 Python 21/21、scoped format/diff/source audit 通过。UI 双域深迁移、PIE viewport、真实 session/零污染、managed Cargo 与独立 review 尚未完成，failure 继续 open；详见 [M2 基础设施子记录](2026-08-26-play-domain-startup-hardcut.md)。 |
| 2026-08-26 CST | `source_hardcut_static_green / embedded_session_and_managed_validation_pending` | retained startup 已删除把 projectless editor-profile `runtime_gateway` 无条件 attach 到 play domain 的临时路径；无明确 gateway 的 backend 默认 `attachable=false`。normal stop、consumer-start compensation、crash、project close 与 host shutdown 统一走 active-mode refusal 和 identity-qualified terminal detach，16 个 Editor04 Python 合同、scoped rustfmt/diff/source scan 通过。P2 App session factory、DTO world payload、viewport、两阶段 destroy、Cargo/产品零污染证据未完成，failure 保持 open；详见 [M2 前置子记录](2026-08-26-play-domain-startup-hardcut.md)。 |
| 2026-07-29 CST | `OPEN / architecture handoff` | Editor01 M2.4 清点发现 `set_runtime_gateway` 覆盖 context gateway、SessionGateway 依法拒绝 borrowed-world access，而 UI edit state 仍直接持有 `LevelSystem`。该三点构成 authoring/runtime ownership 冲突；未修改 Editor04 生产代码、未启动 Cargo。交给 Editor04 的 Edit/Play 双域 gateway 与 attach lifecycle owner 处理。 |
