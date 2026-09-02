# M1 证据

- Gate: design-ready
- Owner session(s): current session
- Changed scope: 收敛 Workbench main band 的侧栏 shrink 策略、viewport 保护优先级和区域间语义间距；压缩 Scene Tree panel 内部密度；保留并发会话已落入 toolbar/activity rail/legacy shell/theme 的现有修改。
- Manifest: `docs/plans/zircon_editor/editor_ui/manifests/m1-zui-shell-layout.yaml`
- Commands actually run: `git diff --check -- <M1 files>`；`python -m unittest tools.tests.test_editor_zui_workbench_layout_contract tools.tests.test_editor_zui_scene_tree_header_contract tools.tests.test_editor_zui_workbench_product_drawer_authority_contract`
- Result summary: 26 tests passed；TOML/ZUI 可解析；main band 现在允许左右 drawer 在窄宽度下收缩，同时 center viewport 保持最高布局优先级且不被分配负最小宽度；Scene Tree 使用更紧凑但 token 化的 gap。
- Repaired failures: none
- Deferred external checks: Windows managed Cargo、真实 Editor 产品截图/resize pressure、Penpot A2-P parity。
- Evidence links: `zircon_editor/assets/ui/editor/components/workbench/shell/workbench_main_band.zui`; `zircon_editor/assets/ui/editor/components/workbench/shell/workbench_scene_tree_panel.zui`。
- Unlocks: M2 输入、焦点和 typed command 提交边界。

## 布局策略

- Activity rail 固定宽度，不参与 shrink。
- 左右 drawer 使用 `StretchContent` 且 `shrink_value = 1.0`，窄窗口优先压缩侧栏。
- Viewport 使用 `Stretch`、`priority = 100`、`weight = 4.0`、`shrink_value = 0.0` 与 `min = 0.0`，保护中央文档并允许容器在极窄窗口稳定求解。
- 区域分隔使用 `$editor.density.gap.xsmall`，Scene Tree 内容使用 xsmall/small token，避免散落新颜色或无语义像素常量。

## 并发安全

`workbench_shell.zui`、`workbench_top_toolbar.zui`、`workbench_activity_rail.zui`、`editor_workbench_strict.zui` 和 `layout/presets.toml` 在本切片开始前已有未提交修改。本里程碑没有覆盖或重写这些并发改动；提交时只 stage M0 文档、`workbench_main_band.zui` 和 `workbench_scene_tree_panel.zui`。
