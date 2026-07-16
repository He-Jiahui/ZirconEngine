---
handoff_kind: fixed
status: fixed
created_at: 2026-07-14
summary_slug: compute-fullscreen-descriptor-compile-regression
origin_plan: docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
fixing_plan: docs/plans/zircon_runtime/shader/04-material-binding-and-renderer-contract.md
origin_child_dir: docs/plans/zircon_editor/editor/02
fixing_child_dir: docs/plans/zircon_runtime/shader/04
related_code:
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/hzb.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/compute_workload.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/fullscreen_pass.rs
tests:
  - cargo test -p zircon_runtime --locked
resolved_at: 2026-07-14
---


# Shader04：compute/fullscreen descriptor 编译回归

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md`
- 来源执行切片：M1 测试阶段 / `zircon_runtime` Windows 正式门禁
- 修复责任计划：`docs/plans/zircon_runtime/shader/04-material-binding-and-renderer-contract.md`
- 交接原因：失败发生在 Shader04 正在迁移的 compute/fullscreen descriptor 目录，低于且独立于 Editor02 的 scene world-sync 变更；当前 Shader04 活动 Session 已声明该目录为写入范围。

## 失败现象与复现证据

2026-07-14 在协调器管理的 Windows test lane `D:\cargo-targets\zircon-engine\pool\841a130ffbd3fd2e938e76b488988119044b676acced751dae7166d95d7f1025` 执行：

```text
cargo test -p zircon_runtime --locked
```

两次正式复现均以 Cargo `exit 101` 停在 `zircon_runtime` 编译期；第二次非详细输出完整保留以下两个错误：

1. `hzb.rs:9` 从 `compute_workload` 导入 `HZB_BUILD_PIPELINE_LABEL` 与 `HZB_BUILD_WORKGROUP_SIZE`，但该模块中不存在这两个符号（`E0432`）。
2. `fullscreen_pass.rs:18-22` 在独立语句中调用按值接收 `self` 的 `FullscreenPassBuilder::with_pipeline_label(...)` 后又返回原 `builder`，造成 moved value（`E0382`）。

Editor02 新增的 `zircon_runtime_interface` world-sync 契约门禁已独立通过；本次 broad runtime gate 尚未编译到可执行测试阶段，因此不借该结果宣称 Editor02 M1 runtime 测试通过。

## 最低共享层根因

已证明的最低边界是 Shader04 compute/fullscreen executor migration 的 descriptor 组装层：HZB descriptor 与其常量 owner 的公开面未同步，fullscreen helper 仍按可变借用式链调用一个已经硬切为 consuming-builder 的 API。具体提交时序与最终 owner 拆分仍由 Shader04 Session 定稿；没有证据指向 scene inspection、World generation 或 Editor02 DTO。

## 架构修复验收

- HZB pipeline label/workgroup size 只有一个明确 owner，`hzb.rs` 从该 owner 使用真实符号，不复制常量或加兼容 re-export。
- fullscreen descriptor 按 consuming-builder 契约完成一次所有权链，禁止通过 clone、旧签名 shim 或调用点例外掩盖迁移未完成。
- `cargo check -p zircon_runtime --lib --locked` 与 Shader04 的 compute/fullscreen/HZB 聚焦测试通过。
- 原复现 `cargo test -p zircon_runtime --locked` 至少越过这两个编译错误；Editor02 随后重跑其 `cargo test -p zircon_runtime --lib scene:: --locked` 上行门禁。

## 禁止临时方案

- 不添加 alias、compatibility shim、silent fallback、重复常量真相、test-only bypass 或调用点特例。
- 不削弱测试或计划验收标准来隐藏失败。
- 不由 Editor02 会话抢改 Shader04 活动 Session 正在占用的 descriptor 实现。

## 修复结果与回传

- 根因：HZB descriptor imported build-dispatch constants from the compute-workload owner after that API had moved, while the fullscreen helper reused a builder after a consuming method call.
- 架构修复：HZB build now consumes the typed hzb_build_dispatch_plan from the graphics shader owner; the obsolete fullscreen descriptor owner was deleted and builtin fullscreen plans use a single consuming builder chain in builtin_global_shader_contracts, with no shim or duplicate constants.
- 验证：Windows managed runtime build job 6c5620111c344c14ba8e758963ae0834 exited 0. Managed broad runtime job f4aec4eca4c445aea8e7e19adcef1c0a compiled past both original Shader04 E0432/E0382 errors and stopped only at Editor02/Plugins08 diagnostics; later exact core-min jobs also contain neither original Shader04 error.
- 回传：Shader04 compute/fullscreen descriptor compile regression is fixed and returned to Editor02; remaining Runtime15 and Frameworks05 failures are separately routed.
