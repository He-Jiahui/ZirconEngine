---
related_code:
  - tools/session_tray/src/app.rs
  - tools/session_tray/src/menu.rs
  - tools/session_tray/src/startup.rs
implementation_files:
  - tools/session_tray/src/app.rs
  - tools/session_tray/src/menu.rs
  - tools/session_tray/src/startup.rs
plan_sources:
  - docs/superpowers/specs/2026-07-11-workflow-control-center-and-tray-design.md
  - docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
tests:
  - tools/session_tray/src/menu.rs
  - tools/session_tray/src/startup.rs
doc_type: plan-output-record
---

# Windows Tray Startup Action Confirmation

Owner plan: `../01-workflow-control-center-and-tray.md`

## 状态与产出记录

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| M6 | 托盘启动项变更两阶段确认 | `completed` | 2026-07-13 | Install/Update/Remove 首次点击只查询协调服务与托盘启动项并保存有界结果指纹；再次点击前重新查询，动作或状态不一致则拒绝执行。Cancel Pending 同时清除生命周期与启动项预览，诊断只投影 pending action kind。Windows 受管 Cargo 池 `925f3a77…` 聚焦 4/4、完整 Tray 33/33 通过，`cargo fmt --check` 通过；release 在受管池 `3107230c…` 构建成功，生产进程已热替换且源/运行 SHA-256 同为 `0AE25A7C…9775`。 |
