---
handoff_kind: failure
status: open
created_at: 2026-08-13
summary_slug: workbench-drawer-header-token-drift
origin_plan: docs/plans/zircon_editor/editor_ui/12-unreal-magicavoxel-zui-design-convergence.md
fixing_plan: docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md
origin_child_dir: docs/plans/zircon_editor/editor_ui/12
fixing_child_dir: docs/plans/zircon_runtime/runtime/09
plan_link_mode: child_record_only
related_code:
  - zircon_editor/assets/ui/editor/windows/workbench_window.zui
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/drawer_layout.rs
  - zircon_editor/src/ui/workbench/autolayout/workbench_chrome_metrics.rs
  - zircon_editor/src/ui/workbench/autolayout/region/tool_region/collapsed_constraints.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/workbench_projection/shell_layout.rs
tests:
  - cargo test -p zircon_editor --locked --lib collapsed_bottom_drawer_input_uses_the_callers_panel_header_metric
  - cargo test -p zircon_editor --locked --lib narrow_width_collapses_a_visible_bottom_drawer_to_its_tab_strip
  - cargo test -p zircon_editor --locked --lib componentized_narrow_workbench_keeps_a_token_sized_bottom_drawer_reopen_strip
  - cargo test -p zircon_editor --locked --lib mounted_workbench_batches_state_projection_into_one_layout_pass
---

# Runtime09 failure handoff: workbench drawer header token drift

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor_ui/12-unreal-magicavoxel-zui-design-convergence.md`
- 来源执行切片：M3 工作台壳层与高密度布局
- 来源执行 Session：`editor-ui12-m3-workbench-shell-hold-v1-20260812`
- 修复责任计划：`docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`
- 交接原因：Runtime09 拥有 componentized retained-host drawer layout、响应式 compaction 与单次布局投影；UI12 只拥有 `.zui` 壳层约束，不能在资产侧遮蔽 runtime override。

## 失败现象与复现证据

UI12 M3 已把 `workbench_window.zui` 的 bottom drawer 最小高度收敛到 `$editor.chrome.panel_header.height`，并由 `WorkbenchChromeMetrics::panel_header_height` 投影为共享 chrome metric。Autolayout 的 `collapsed_region_constraints` 也已把 bottom collapsed height 明确绑定到同一 metric。

当前 componentized retained-host bridge 仍在 `drawer_layout.rs` 私有声明 `AUTHORED_DRAWER_HEADER_HEIGHT: f32 = 42.0`，并在两个 live 分支使用它：

1. `WorkbenchDrawerLayoutInputs::from_model` 把它作为 bottom drawer 的 collapsed extent；
2. `compact_bottom_drawer_for_layout_tier` 在 Ultra/Narrow tier 强制把 bottom extent 重写为该值。

因此 `.zui`、autolayout 与 componentized bridge 对同一 reopen strip 存在两个 authority。默认 `WorkbenchChromeMetrics::panel_header_height` 与 UI12 token 当前为 25px，而 live bridge 可把窄宽 bottom drawer 重写为 42px。

UI12 已新增 focused behavior contract `componentized_narrow_workbench_keeps_a_token_sized_bottom_drawer_reopen_strip`，要求 640x420 下 shell/header 都等于 `metrics.panel_header_height` 且 content frame 折叠。动态 red 尚未形成：2026-08-13 五次 managed Windows 运行都在目标测试执行前被异步删除整个 Cargo target 目录，以 `os error 3` 终止；自定义 E/D 目录、coordinator-managed pool lane 和独立 ephemeral lane 均复现，目标测试执行数为 0。之后尝试迁移到 `F:\ZirconBuilds`，coordinator 在 acquire 阶段因兼容池作业 `f0403a1606764c6d934dcfb3e4fa1e28` 仍标记 running 而正确拒绝；该作业记录已无 live PID，UI12 不替外部 Session finish/release。以上基础设施失败不得冒充行为 red 或 green。

## 最低共享层根因

Drawer compaction 在 Runtime09 live bridge 内保留了截图时代的 42px 私有常量，没有复用同一 subsystem 已发布的 `WorkbenchChromeMetrics::panel_header_height`。这让 runtime responsive policy 覆盖声明式 token，而不是消费 token-resolved metric。

## 架构修复验收

- 删除 `AUTHORED_DRAWER_HEADER_HEIGHT`，bottom drawer 的 collapsed/model extent 与 Ultra/Narrow compaction 统一消费调用方传入的 `metrics.panel_header_height`。
- 保留 Runtime09 当前的持久 surface、单次 `prepare_layout_at_mount_with_scale`、profiled state/drawer/responsive/surface blocks 与 partial recompute 改造；不得为了修高度退回多次 layout pass。
- `drawer_region_input` 继续保持 generic collapsed extent 参数，不在更下层复制 editor token 或另建 UI12 常量。
- 现有 autolayout `collapsed_region_constraints` 与 componentized bridge 在 default/custom metrics 下产生相同 bottom collapsed height。
- focused contract 实际执行并通过；随后运行 Runtime09 的 componentized workbench layout-pass 回归，确认一次响应式更新仍只执行一次 layout pass。
- 修复完成后写 `fixed-*` return，附 current-source manifest、managed Windows job/run、实际测试数与独立 review 结果。

## 禁止临时方案

- 不得把 `.zui` 最小高度改回 42px、增加第二个 token，或在 UI12 projection 中补偿 runtime override。
- 不得保留常量 alias、兼容 shim、条件分支或 snapshot 特判。
- 不得吸收或回退 Runtime09 未归属给 UI12 的性能改动。
- 不得用静态源码检查代替 focused behavior test；目标测试执行数为 0 时记录保持 open。

## 修复结果与回传

Open state：`source_hard_cutover_complete / managed_behavior_green_blocked_by_cargo_coordinator`。

- 根因：已确认是 Runtime09 componentized bridge 的模型输入与 Ultra/Narrow compaction 同时持有 42px 私有 authority，而不是 `.zui` 或 autolayout token 解析错误。
- 架构修复：已删除 `AUTHORED_DRAWER_HEADER_HEIGHT`；`WorkbenchDrawerLayoutInputs::from_workbench_model()` 和 `compacted_bottom_region_input()` 统一消费调用方 `WorkbenchChromeMetrics::panel_header_height`，generic `drawer_region_input()` 不变。新增 custom metric 低层合同，分别覆盖模型折叠输入与窄屏 compaction。
- 静态验证：目标文件 `rustfmt --edition 2021 --check` 通过；scoped `git diff --check` 通过（仅 CRLF 提示）；仓库内 `AUTHORED_DRAWER_HEADER_HEIGHT` 零引用；plan output audit 通过。
- 非 Cargo 验证：drawer resize、workspace watcher、bounded refresh、watcher generation、hit index、presentation generation 共 17/17 条通过；另 1 条 plan output audit 通过。这些只证明相邻架构合同，没有替代本交接要求的四条 Rust 行为测试。
- 动态 blocker：reuse-pool 申请在目标执行前被 job `f0403a1606764c6d934dcfb3e4fa1e28` 占用拒绝；该 job 的 supervisor/process tree 已退出且 `live_process_pids=[]`，但 coordinator 仍记录 running。随后 `-Ephemeral` 申请又被 reservation `25af0f9160ec4fc39d824a2c143c71ba`（session `plugins01-callback-factory-hardcut-r4-20260811`）拒绝。目标测试执行数仍为 0。
- 回传：尚未回传 UI12；等 managed Windows lane 可用后，必须按低层 custom metric -> componentized shell -> 单次 layout pass 顺序实际执行并通过，完成 current-source review 后再由 coordinator `failure return` 转成 `fixed-*`。
