# Zircon Hub 全页面视觉一致性与设计图补齐计划

## Summary
- 以 `E:/Git/ZirconEngine/docs/ui-and-layout/hub.png` 为像素基准，把 Hub 全页面统一到同一套深色工程工具风格：青绿色主色、紧凑卡片密度、8-12px 圆角、固定按钮/徽章/列表状态、清晰面板层级。
- 方向已切换为用户确认的两阶段工作流：先用 `hub.png` 作为风格参考，通过流式 AI 生成每个逻辑页面/状态的结构方向稿；再用真实 HTML/CSS 在 `docs/ui-and-layout/hub-web-reference/` 复刻最终参考页，保证文本、图标、间距、菜单、弹层和状态面板可控。
- 最终验收资产不是 AI 草图，也不是旧 Pillow 占位生成器；最终 `1568x1003` 参考 PNG 由 web reference 导出脚本生成，AI 草图只作为总体交互结构布局和局部功能内容方向记录。

## Key Changes
- 建立 `HubVisualSpec` 视觉标准：统一背景、面板、边框、阴影、品牌色、状态色、字体层级、按钮状态、卡片比例、toolbar 密度、列表行高和页面留白；标准落到 `theme.slint`、`tokens.slint`、`surfaces.slint`、`inputs.slint`、`data_display.slint`、shell/page component wrappers。
- Projects Dashboard 保留 `hub.png` 作为固定像素基准；web reference 的 dashboard capture 只用于校准 HTML/CSS 复刻效果，不替换 `hub.png`。
- 其它页面套用同一组件标准：Editor、Builds、Assets、Plugins、Cloud、Team、Learn、Settings，以及 Projects New/Browser/Detail，不允许页面私自定义另一套按钮、卡片、列表、空态或状态色。
- 补齐设计图到 `docs/ui-and-layout/`：
  - 主页面：`hub-editor.png`、`hub-builds.png`、`hub-assets.png`、`hub-plugins.png`、`hub-cloud.png`、`hub-team.png`、`hub-learn.png`、`hub-settings.png`
  - Projects 二级页：`hub-projects-new.png`、`hub-projects-browser.png`、`hub-projects-detail.png`
  - 交互状态：`hub-projects-browser-filter-menu.png`、`hub-projects-browser-sort-menu.png`、`hub-projects-detail-delete-confirm.png`、`hub-source-engine-popup.png`、`hub-user-menu.png`
  - 全局状态：`hub-state-empty.png`、`hub-state-loading.png`、`hub-state-error.png`
- 补齐 `docs/ui-and-layout/hub-ai-reference-manifest.json`，记录 `hub.png` 源参考、流式 AI 生成端点/模型/partial image 策略、每个页面的 AI 草图、最终 HTML/CSS export 文件名，以及“AI 草图不是验收证据”的限制。
- 新增 `docs/ui-and-layout/hub-design-board/` 作为设计稿模式：`hub-design-structure-layout.png` 和 `hub-design-structure-supplement.png` 是总体交互结构布局主核查图，`hub-design-functional-details.png` 是功能内容局部图，三者都是补充评审材料，不进入最终 `hub-*.png` 验收清单。
- 退役 `tools/generate-hub-design-assets.py` 的权威地位；合同应要求它不存在或不能作为 Hub PNG 真源恢复。
- 更新 `docs/ui-and-layout/index.md` 和 `docs/zircon_hub/ui/responsive-component-system.md`，记录每张设计图对应页面、状态、验收点和真实截图对照路径。
- 新增或扩展 Hub 静态合同测试：禁止页面级自绘视觉分支、禁止私有颜色/圆角/行高、要求主页面只通过共享组件和 token 组合。

## Test Plan
- 导出最终 web reference：`node docs/ui-and-layout/hub-web-reference/export-pages.mjs`，确认 19 张 `hub-*.png` 和 dashboard web capture 是 `1568x1003`。
- 验证最终参考页：`node docs/ui-and-layout/hub-web-reference/validate-visuals.mjs`，覆盖 PNG 尺寸/内容、根目录 Hub PNG inventory、旧生成器退役、`EXPORTS.md`、AI manifest、docs matrix、README policy、AI 草图和代表性人工 spot-check 清单。
- 验证静态页面交互：`node docs/ui-and-layout/hub-web-reference/validate-interactions.mjs`，覆盖未知 `data-route`/`href?page=` 目标、导出文件名回放、`EXPORTS.md` 回放路径和代表性点击路线。
- 验证设计稿模式：`node docs/ui-and-layout/hub-design-board/validate-design-board.mjs`，覆盖 3 张补充设计板尺寸/动态范围、manifest、导出索引、结构核查文档、覆盖矩阵、固定画布无滚动溢出、主结构图几何边界和关键标签 fit。
- 聚焦合同：`cargo test --manifest-path zircon_hub/Cargo.toml --locked --offline --jobs 1 --test ui_visual_standard_contract`。
- 视觉验收：用 Hub 截图脚本捕获 Projects Dashboard/New/Browser/Detail、所有主导航页、菜单、确认、空态、加载态、错误态；对照新增 PNG 设计稿检查无重叠、无裁剪、无密度漂移、按钮状态一致。
- 重点断点：`1568x1003` 作为主像素基准，同时保留 `1600x1024`、`1280x900`、`1024x720` 响应式截图验收。

## Assumptions
- `hub.png` 保留为 Projects Dashboard 的最高优先级视觉基准；其它页面设计图从它抽象出的组件/token 标准派生。
- AI 草图和设计板是仓库文档资产，不替代真实 Hub 截图；实现完成后必须同时保留最终 HTML/CSS 导出参考图、设计方向记录和运行截图证据。
- 本轮不新增业务数据模型，不改变 Hub 页面路由语义；主要改 Slint 视觉组件、页面组合、合同测试、文档和设计图。
