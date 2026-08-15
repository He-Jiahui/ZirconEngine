---
status: implemented_static
validation: pending
milestone: S15.1
updated_at: 2026-08-13
---

# Workbench Palette Component Baseline

已完成工作台原子调色板的统一收敛。`EditorPaletteTokens::workbench_dark()` 现在以 `docs/ui-and-layout/ai-workbench-style/prototype/base.css` 为基准：外壳、chrome、面板、输入 recessed surface、hover/selected state、accent、border、popup、track 和三级文字均由一个中央令牌表定义。控件高度、圆角、间距、响应断点和业务状态机未改动；success/info/warning/error 语义色保持原值。

`separator_strong` 与常驻 dock 的 `--line` 收敛为 `#223037`，而 `separator_soft` 使用其在 `#10181c` 面板上的预混合结果 `#1b2428`。保留宿主 CPU 截图帧当前覆写 RGBA 像素而不作 source-alpha blending，因此没有直接导入 CSS 的半透明分隔线，保证截图与运行时命令流可一致重放。

`cascade_registry` 的回归测试锁定可见原子令牌基线；`editor_design_tokens` 锁定 Runtime 接口中的默认工作台投影。二次静态审查确认编辑器保留宿主经 `EditorDesignTokens -> HostMaterialPalette` 消费这些值，未发现生产路径硬编码旧工作台 RGB。`UiThemeDocument::dark()` 是通用 UI 的独立兼容主题，未被本编辑器视觉切片修改。

同一 S15.1 切片已继续收束 Inspector 可编辑字段组。字段、诊断、动态属性、向量列、动作按钮的行高、面板内边距、列间距、圆角和边框不再保留本地 `INSPECTOR_*` 度量常量，而是读取已按宿主缩放解析的 `HostControlMetrics`。默认工作台仍保持紧凑基线（28 行高、8 内边距、6 间隙、4 圆角、84x24 动作按钮）；定向回归同时锁定自定义度量下字段、圆角、边框、按钮与相邻行的同步变化。间隙推导会钳制为非负，防止异常紧凑主题导致字段重叠。

已执行目标 Rust 2021 `rustfmt --check` 和 scoped `git diff --check`。当前未运行受管 Cargo，未生成或修改截图；后续当前源码视觉验证仅可写入 `docs/tests/editor`，不得写入 `target`。本记录保持 `implemented_static / validation_pending`，由协调器执行构建和截图后才能进入 accepted closeout。
