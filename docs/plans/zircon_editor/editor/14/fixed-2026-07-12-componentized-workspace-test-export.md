---
handoff_kind: fixed
status: fixed
created_at: 2026-07-12
summary_slug: componentized-workspace-test-export
origin_plan: docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
fixing_plan: docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
origin_child_dir: docs/plans/zircon_editor/editor/14
fixing_child_dir: docs/plans/zircon_editor/editor_ui/08
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/scene_layers/overlay.rs
tests:
  - cargo test -p zircon_editor --lib --locked --jobs 1 core::jobs::tests -- --test-threads=1 --nocapture
resolved_at: 2026-07-12
---


# Editor UI 08：componentized workspace 测试导出边界断裂

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md`
- 来源执行切片：Editor14 M2 线程所有权与终态资源合同最终重编译
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`
- 交接原因：失败符号属于 retained workbench renderer 的 scene-layer 测试导出边界，不属于 job scheduler 或线程所有权。

## 失败现象与复现证据

受管 Windows job `8a813aefe7294e93981b3925466f08ed` 在测试体启动前失败：`paint_workbench_renderer.rs:18` 无法从 `scene_layers` 导入 `paint_componentized_extension_workspace_for_test`（E0432）。原命令为 `cargo test -p zircon_editor --lib --locked --jobs 1 core::jobs::tests -- --test-threads=1 --nocapture`，stderr 为 `D:/cargo-targets/editor14-final-focused-20260712.err.log`。

## 最低共享层根因

最低可证实边界是 `paint_workbench_renderer` 根模块与 `scene_layers/overlay.rs` 的测试可见性/重导出漂移：根模块仍依赖该测试辅助符号，但 scene-layer public-within-crate 路径未提供同名导出。具体应恢复唯一 owner 还是删除过期根导出，由 Editor UI 08 按当前硬切架构判定；不得在 Editor14 加别名绕过。

## 架构修复验收

- retained workbench renderer 的测试辅助符号只有一个当前 owner，根模块与 scene-layer 导出一致。
- focused renderer/workbench 测试通过。
- 原 Editor14 `core::jobs::tests` 重编译并自然结束。
- `cargo test -p zircon_editor --lib --locked --jobs 1` 能继续进入测试体。

## 禁止临时方案

- 禁止在 Editor14 或无关模块添加同名 stub、兼容 re-export、条件跳过或注释掉测试导出。
- 禁止弱化 renderer/workbench 或 Editor14 门禁来隐藏 E0432。

## 产出记录与时间

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| Editor UI 08 / Editor14 M2 | componentized workspace 测试导出边界 | `open-待对应功能修复` | 2026-07-12 | job `8a813aefe7294e93981b3925466f08ed`；E0432 位于 `paint_workbench_renderer.rs:18`；日志 `D:/cargo-targets/editor14-final-focused-20260712.err.log`。 |

## 修复结果与回传

- 根因：The retained renderer root expected a cfg(test) helper whose scene-layer export chain stopped below the root module.
- 架构修复：Restored one cfg(test)-only ownership chain from overlay through scene_layers and paint_workbench_renderer to host_contract; no production stub, alias, or scheduler workaround was added.
- 验证：Current zircon_editor test binary: original componentized workspace painter prefix 3/3 passed. Editor14 managed rerun: core::jobs::tests 36/36 passed, exit 0; E0432 did not recur.
- 回传：The componentized workspace test-helper export boundary is fixed and the originating Editor14 suite compiles and passes.
