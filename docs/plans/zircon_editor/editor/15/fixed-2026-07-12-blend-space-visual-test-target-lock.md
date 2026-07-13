---
handoff_kind: fixed
status: fixed
created_at: 2026-07-12
summary_slug: blend-space-visual-test-target-lock
origin_plan: docs/plans/zircon_editor/editor/15-build-export-and-publishing.md
fixing_plan: docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md
origin_child_dir: docs/plans/zircon_editor/editor/15
fixing_child_dir: docs/plans/zircon_editor/editor_layout/15
related_code:
  - zircon_editor/src/tests/host/retained_menu_pointer/visual_screenshot/blend_space_workspace.rs
tests:
  - tests::host::retained_menu_pointer::visual_screenshot::blend_space_workspace::capture_blend_space_workspace_visual_artifacts
resolved_at: 2026-07-12
---


# Layout 15：Blend Space 视觉测试占用共享测试程序

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/15-build-export-and-publishing.md`
- 来源执行切片：Editor15 typed export error 硬切后的向上 Windows 聚焦验证
- 修复责任计划：`docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md`
- 交接原因：失败进程属于 Blend Space 原生视觉截图路由，其生命周期和验证通道由 Layout15 视觉验收切片负责；Editor15 不终止、不接管该进程，也不把它误报成导出代码失败。

## 失败现象与复现证据

2026-07-12 14:13（Asia/Shanghai），Editor15 当前源码已通过 `zircon_editor` lib-test 编译，但链接阶段报告 `LNK1104`，无法覆盖协调器 target 中的 `zircon_editor-3d08c315be91d95a.exe`。

进程审计确认该文件正由 PID `15812` 执行 ignored Blend Space 截图用例：

`tests::host::retained_menu_pointer::visual_screenshot::blend_space_workspace::capture_blend_space_workspace_visual_artifacts --ignored --exact --nocapture --test-threads=1`

其父进程是直接启动测试程序的 PowerShell，而不是当前 Editor15 Cargo job。该占用导致同一兼容 target 的其他受管验证无法重链接，即使待验证源码本身没有新增编译错误。

14:24 的第二次证据进一步确认通道归属问题：Layout15 直接运行 `cargo test ... blend_space_workspace:: --no-run --target-dir <shared-pool>`，没有先通过协调器取得该 pool；与此同时协调器把同一 pool 独占登记给 Editor15 job，Editor15 Cargo 因 artifact directory 文件锁等待。即使截图进程已退出，绕过协调器的同 target Cargo 仍会造成交叉阻塞。

## 最低共享层根因

Layout15 视觉截图执行未把“测试程序文件仍在运行”纳入共享 target 生命周期门禁。Cargo target 可复用并不等于其中的 Windows `.exe` 可在并发执行时被链接器覆盖；ignored 截图路由必须独占自己的验证通道，或在退出后确认进程和文件句柄均已释放，不能只释放/超时 Cargo job。

## 架构修复验收

- Blend Space ignored screenshot 必须通过协调器获取独立或互斥兼容的 Windows test target，并登记完整进程生命周期。
- 截图命令退出后确认 `zircon_editor-*.exe` 无残留进程/句柄，再释放验证通道。
- 重新运行 inactive-host sibling 回归和两档真实窗口截图，不复用仍被其他任务执行的测试程序。
- 用一个并行受管 `zircon_editor` lib-test 链接验证证明视觉截图不会再造成 `LNK1104`。

## 禁止临时方案

- 禁止由其他功能 owner 强杀视觉进程、删除共享 target 或忽略 `LNK1104` 后宣称验证通过。
- 禁止把共享 target 锁冲突伪装成 Editor15 导出错误回归。
- 禁止绕过协调器直接长期运行 retained screenshot 二进制。
- 禁止在其他 session 已持有的协调器 pool 上直接追加 `cargo test --target-dir <shared-pool>`；`--no-run` 同样会编译、链接并持有 Cargo artifact lock。

## 产出记录与时间

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| Layout15 S15.5 | Blend Space 原生视觉验证通道生命周期 | `open-待Layout15处理` | 2026-07-12 | ignored screenshot PID `15812` 占用协调器复用 target 的 `zircon_editor-3d08c315be91d95a.exe`，并行 Editor15 链接失败为 `LNK1104`；随后 Layout15 又绕过协调器对同 pool 启动 `--no-run` Cargo，使已登记的 Editor15 job 等待 artifact lock。 |

## 修复结果与回传

- 根因：Blend Space ignored screenshot test process kept a shared Windows test executable open while another session attempted to relink the same coordinator target; a later direct --target-dir build also bypassed the lease.
- 架构修复：Waited for the owning coordinator job, acquired and registered a Layout15 Cargo lease for the complete build lifecycle, removed temporary diagnostic image output, and verified the screenshot test process releases the executable before channel release.
- 验证：Managed build lease 96e284e9ec5a4ae4a0096ff930fcc545 succeeded; componentized painter 4/4, Blend Space non-ignored 3 passed plus 1 ignored, ignored screenshot 1/1; target executable accepted an exclusive FileShare::None open after exit; no Blend Space PNG exists in Cargo targets.
- 回传：Layout15 has fixed and validated the visual-test target lifecycle; Editor15 no longer owns this failure.
