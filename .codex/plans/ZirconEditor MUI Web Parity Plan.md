# ZirconEditor MUI Web Parity Plan

**Summary**
- 目标：让 `zircon_editor` 的 retained UI 与本地 `dev/material-ui` v9.0.1 的 MUI Web 风格、SVG 图标、组件变体、反馈状态和自定义能力高保真一致。
- 约束：不向 `zircon_editor` 或 `zircon_runtime` 引入新的 Slint 依赖；`zircon_hub` 现有 Slint 使用保持独立。
- 策略：底座优先，先补主题、样式、绘制、输入、叠层、组件契约，再全量组件化，最后替换编辑器实际界面。
- 验收：视觉+行为高保真，不做所有组件逐像素强制一致；关键状态用代表性截图和交互用例验收。

**Key Changes**
- 以 `dev/material-ui/packages/mui-material`、`mui-lab`、`mui-icons-material` 为权威源，建立 MUI 组件清单、变体清单、默认主题清单和 SVG 图标映射。
- 扩展 retained UI 底座：MUI palette、typography、spacing、shape、shadow、z-index、transition、state class、slot/slotProps、`sx` 等能力映射到现有 `.v2.ui.toml` / `.zui` / runtime style 体系。
- 强化 [zircon_editor/src/ui](<E:/Git/ZirconEngine/zircon_editor/src/ui>)：绘制 Paper elevation、ripple、focus ring、Backdrop、Popover/Popper、Modal、Progress、Skeleton、Slider/Switch/Checkbox/Radio 等 MUI 基础视觉原语。
- 强化 [zircon_runtime/src/ui/component/catalog/material_foundation](<E:/Git/ZirconEngine/zircon_runtime/src/ui/component/catalog/material_foundation>)：Core、Lab、X 组件全量登记，组件 props 对齐 MUI 的 `variant`、`color`、`size`、`disabled`、`slots`、`slotProps`、状态类和自定义样式。
- 将现有 Slint Material retained migration 的 M0-M2 作为底层基础继续使用，但从新计划开始以 MUI Web 源码为视觉和 API 权威，避免继续扩大 Slint 风格偏差。

**Milestones**
- M0 Inventory Gate：生成并测试 MUI Core/Lab/X 组件、变体、状态、图标、主题 token 覆盖清单；确认 `zircon_editor` 仍无 Slint 依赖。
- M1 Foundation Gate：完成 MUI 主题 token、SVG 图标注册、state class、slot/slotProps、`sx`/style override 到 retained style 的统一映射。
- M2 Renderer/Input Gate：补齐 MUI 所需绘制原语、动态状态、键盘焦点、文本输入、浮层定位、模态焦点管理和滚动/虚拟化基础。
- M3 Core Components Gate：生产级实现 `mui-material` Core 组件，包括按钮、输入、选择、导航、反馈、数据展示、表面、布局、过渡等全部类别。
- M4 Lab/X Gate：生产级实现 Lab/X 组件，包括 Data Grid、Charts、Date/Time Pickers、Tree View、Timeline、Masonry、高级 Autocomplete、Transfer List、Chat 类组件。
- M5 Editor Adoption Gate：用新 MUI 组件替换 ZirconEditor 主界面、工具栏、Inspector、Asset Browser、Material Lab 页面和弹窗体系。
- M6 Visual Acceptance Gate：建立组件矩阵截图、关键交互录制/截图、主题切换、禁用/悬停/按下/聚焦/错误状态验收，并更新 UI 文档。

**Public Interfaces**
- 不新增 Slint-facing API。
- 保留现有 retained UI 资源格式，扩展组件 descriptor、theme token、runtime style pseudo-state、slot/slotProps 和图标注册能力。
- MUI 自定义能力统一表达为：theme tokens、component default props、slot props、state classes、style overrides、template-local props。

**Test Plan**
- 静态边界：测试 `zircon_editor/Cargo.toml` 和源码不引入 Slint；测试本地 MUI 组件清单与 descriptor 覆盖一致。
- 解析与契约：测试 `.v2.ui.toml`、`.zui`、组件 props、变体、状态类、slot/slotProps、SVG 图标注册。
- 行为测试：覆盖 hover、pressed、focused、disabled、selected、checked、expanded、open、error、loading、dragging、modal/backdrop、keyboard navigation。
- 视觉测试：对代表性组件和编辑器主要页面做桌面截图验收，覆盖 light/dark、密度、关键变体和反馈状态。
- 构建验证：每个里程碑末尾运行 scoped `cargo check` / `cargo test`，失败时先诊断底层共享能力，再向上重跑相关测试。

**Assumptions**
- 权威基线固定为本地 `dev/material-ui` v9.0.1，不跟随线上最新版漂移。
- Core、Lab、X 都进入最终生产级目标，但按里程碑分批完成。
- 允许复用和改造现有 Material retained migration、Material Lab `.zui` 原型、editor SVG 资产和 runtime component catalog。
- “完全一致”解释为 MUI Web 风格、结构、状态和行为高保真一致，不要求所有组件跨平台逐像素相同。
