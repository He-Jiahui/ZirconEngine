# Material UI 全组件原型陈列与渲染实现计划

## Summary
目标是在现有 `main` 工作区内，把刚才的深色 Material 组件实验室标准落成可执行实现：先完善 `docs/ui-and-layout` 的组件设计规范，再用现有 `.zui` 扁平 TOML 约定创建“单组件原型”，接着补通用 WGSL UI shader，最后在独立 demo 窗口陈列 Material UI + MUI X 组件原型并验证交互反馈。

参考源锁定为官方 [Material UI all components](https://mui.com/material-ui/all-components/)、[MUI X Tree View](https://mui.com/x/react-tree-view/)、[Charts](https://mui.com/x/react-charts/)、[Data Grid](https://mui.com/x/react-data-grid/) 和 [Chat](https://mui.com/x/react-chat/)，本地源码参考 `dev/material-ui/packages/mui-material`、`dev/Graphics` Canvas UI shader 和 Unreal Slate shader。

## Interfaces
- `.zui`：采用现有 `asset.kind = "component"`、`[components.<Name>]`、`[nodes.*]` 扁平 TOML，不新增业务 schema；每个 MUI 组件一个 `material_<component>.zui` 文件，统一使用 `variant`、`size/density`、`surface_variant`、`button_variant`、`text_tone`、`validation_level`、`hovered/pressed/focused/selected/checked/disabled/open/popup_open` 等属性。
- 组件目录：将 `zircon_runtime/src/ui/component/catalog/material_foundation.rs` 拆成 `material_foundation/{mod.rs, inputs.rs, data_display.rs, feedback.rs, surfaces.rs, navigation.rs, layout.rs, mui_x.rs}`，避免继续堆叠到接近 1000 行的大文件。
- Shader：新增 `zircon_runtime/src/rhi_wgpu/ui_surface/shaders/ui_material.wgsl`，由 `pipeline.rs` `include_str!` 使用；保留公开 `UiSurfaceCommandKind::{Quad, Border, Image, Text}`，内部新增 Material UI vertex/primitive 参数来支持 tint、alpha、clip、rounded SDF、border、texture sampling。
- Demo：新增 `zircon_editor/assets/ui/editor/material_component_lab.zui`，不破坏现有 `component_showcase.zui`；首屏结构按生成图标准：Top AppBar、左侧 Drawer、主内容分组网格、右侧状态/交互说明面板。

## Milestones
1. **文档与组件矩阵扩展**
   - 扩展 `docs/ui-and-layout/material-ui-component-design-matrix.md`：每个核心 MUI 组件补齐响应机制、外观变体、布局模式、`.zui` 映射、验证项。
   - 新增或更新 `docs/ui-and-layout/material-ui-token-component-audit.md`：记录 dark Material tokens、状态优先级、MUI X 扩展、shader 渲染约定。
   - 把 MUI X 作为新增组：Tree View、Data Grid、Charts、Chat/AgentChat，并标明 Community/Pro/Premium 只做公开文档可见原型，不实现商业高级业务能力。

2. **`.zui` 单组件原型规范**
   - 创建单组件 `.zui` 原型：Inputs、Data Display、Feedback、Surfaces、Navigation、Layout、Lab/Utils、MUI X 分组。
   - 每个可交互组件至少包含默认、hover、pressed、focus、disabled、selected/open/error 中的核心状态样例。
   - 非视觉工具类如 Portal、NoSsr、ClickAwayListener、useMediaQuery 只做 behavior/utility 说明和 demo 占位，不强行画成控件。

3. **通用 UI Shader 与 GPU Surface**
   - 将当前 inline solid/image/blit WGSL 收敛到 `ui_material.wgsl`，保留独立 entry points。
   - 使用 Unity Canvas 思路：vertex color tint、透明混合、clip rect、mask softness、premultiplied alpha。
   - 使用 Unreal Slate 思路：rounded box SDF、fill/border 双色、抗锯齿边缘、UI 专用无光照输出。
   - 更新 `geometry.rs`/`batching.rs`，使圆角、边框、裁剪、damage 后的批处理仍不退回 software fallback。

4. **Material Component Lab Demo**
   - 新建 `material_component_lab.zui`，导入所有单组件 `.zui` 原型。
   - 组件分组与官方一致：Inputs、Data display、Feedback、Surfaces、Navigation、Layout、Utils/Lab、MUI X。
   - MUI X 原型包含：Tree View 展开/选择/编辑态，Data Grid 表头排序/选中行/虚拟行，Charts line/bar/pie/sparkline/gauge，Chat conversation list/messages/composer/typing/error/streaming 状态。

5. **交互反馈与状态路由**
   - 为 Button、TextField、Switch、Slider、Select、Toggle、Checkbox、Menu、Dialog、Tooltip、Drawer、Pagination、TreeView、DataGrid、Chat composer 增加 retained event route 验证。
   - 状态反馈统一：hover = 亮状态层，pressed = 深状态层，focus = 2px teal ring，selected/open = primary container，disabled = opacity + no action，error/warning = 高优先级 tone。
   - 第一轮不做 ripple/motion 动画，只冻结静态反馈图示和事件触发结果。

6. **文档与验收证据**
   - 更新 `docs/ui-and-layout/index.md` 指向新矩阵、`.zui` 组件规范、shader 设计记录和 demo 验证证据。
   - 记录参考文件：MUI 源码、MUI 官方页面、Unity Graphics Canvas shader、Unreal Slate shader、Zircon runtime/editor 实现文件。
   - 若 `Cargo.lock` 仍有外部不一致，验证阶段只记录阻塞，不自动改锁文件。

## Test Plan
- 文档完整性：更新 `zircon_editor/src/tests/ui/boundary/material_ui_component_design_matrix.rs`，验证核心 MUI 目录 + MUI X 行都包含 response/appearance/layout/validation 字段。
- `.zui` 合约：新增测试批量加载所有 `material_*.zui`，验证每文件一个组件根、可解析、事件 route 合法、核心状态 props 存在。
- Catalog：测试 Material foundation registry 覆盖所有 demo 组件，组件类别、props、events、host/render capability 不缺失。
- Shader/GPU：测试 WGSL pipeline 可创建，rounded fill/border、clip、damage、batching、image/text 混合不退回 fallback。
- 交互：跑 focused event tests 覆盖 click/hover/press/focus/open/close/select/check/drag/expand/chat send。
- 最终命令：`cargo fmt --all -- --check`、`cargo test -p zircon_runtime --lib ui_surface component_catalog material_layout event_routing --locked --jobs 1`、`cargo test -p zircon_editor --lib material_ui_component_design_matrix material_meta_component_contracts native_material_painter component_showcase --locked --jobs 1`、`cargo check -p zircon_editor --lib --tests --locked --jobs 1`。
- 视觉证据：使用现有 `tools/ui-profile-capture.ps1` 捕获 Material Component Lab 的 startup、hover、click 场景截图并记录到文档。

## Assumptions
- “完全与 Material UI 保持一致”定义为组件族、variant、状态语义、暗色 Material 视觉语言一致；不复制 React API，也不逐像素复刻网页 light theme。
- 首个可见交付物是完整 demo 窗口原型陈列；复杂行为如 Data Grid 服务端数据、Charts WebGL、大规模 Chat adapter、Tree View Pro 虚拟化只做原型和 needs-support 记录。
- 本轮仍限定 Editor retained UI，不修改 `zircon_hub` Slint UI。
