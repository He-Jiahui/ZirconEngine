# Zircon UI `.zui` 组件资产与 Unreal 风格入口重构计划

## Summary
当前主入口已经在走 v2 资产链路，但视觉仍是蓝色 Material 临时风格，且启动默认不是控件 Demo。下一轮改为：`zircon_editor.exe` 默认进入控件演示界面；`.zui` 成为正式 UI 组件资产后缀；每个 `.zui` 只允许定义一个独立组件；Unreal 风格主界面由运行时从这些组件原型拼接生成。

## Key Changes
- 新增 `.zui` 资产规范：内容仍使用 TOML 语法，但公开 API 使用 `load_zui_*`；`.zui` 只允许 `asset.kind = "component"`、单个 `[component]`、一组 `nodes`、可选组件内 tokens/styles/imports，禁止 `view`、`style`、多组件表。
- 硬切资产入口：资产导入器、插件 UI 组件校验、v2 文件缓存、内置 UI 注册从 `.v2.ui.toml` 切到 `.zui`；旧 `.v2.ui.toml` 仅保留在迁移/fixture 测试，不再是编辑器主路径。
- 主入口切换：无项目启动时默认打开已有控件演示，但演示资产改为 `.zui` 单组件拼接，不再依赖整棵 view TOML；默认 Workbench 暂时保留为菜单入口。
- Unreal 风格主界面：建立 `editor_unreal_dark` 主题和组件目录，按 `.zui` 拆成 MainMenu、PrimaryToolbar、ViewportToolbar、Outliner、Details、ContentBrowser、Console、StatusBar 等单组件，再由 Rust 运行时 composition builder 拼成主界面。
- 性能修复优先点：定位并缩小 `host_lifecycle` 全量 presentation 重建、`native_pointer` full-frame 返回、`presenter` 全帧像素拷贝、Dock resize 每次移动触发布局的热点；常规 hover/press/selection 必须走 render-only 或局部重绘。

## Milestones
- **M1: `.zui` loader 与资产规则**
  - 增加 `UiZuiAssetLoader` 和单组件 profile validator。
  - `.zui` 转换成现有 `UiV2AssetDocument` 内存结构供编译器复用。
  - 更新 importer/plugin/cache 后缀判断为 `.zui`。
  - 增加拒绝多组件、拒绝 view/style、拒绝旧后缀主路径的测试。

- **M2: Demo 作为默认入口**
  - 将控件演示迁移为多个 `.zui` 组件。
  - 启动状态默认打开控件演示窗口/面板。
  - 欢迎页和默认 Workbench 仍可从菜单打开，但不再作为默认首屏。

- **M3: Unreal 风格组件化 Workbench**
  - 新建 Unreal 暗色 token、低圆角、细分割线、紧凑行高和中性灰面板。
  - 用 `.zui` 单组件重建顶部菜单、主工具栏、左侧面板、中央视口、右侧 Details/Outliner、底部 Content Browser。
  - Rust composition builder 只拼接组件引用和 props/state patch，不反序列化整棵 UI 树。

- **M4: 卡顿热点闭环**
  - 用现有 profiling/overlay 记录启动、hover、点击、拖拽、resize、资产刷新、视口刷新场景。
  - 将非结构性交互从 `PRESENTATION_DATA/LAYOUT` 降级为 paint-only/render-only。
  - 将 full-frame 请求改为 region request，只有布局结构变化才触发完整 frame update。

## Test Plan
- `.zui`：单组件通过；多组件、view/style、旧 `.v2.ui.toml` 生产导入失败。
- 启动：`zircon_editor.exe` 默认显示控件演示，不进入旧蓝色 Workbench。
- 性能：hover/press 不增加 presentation rebuild；视口图像更新只 paint-only；dock resize 拖动不逐事件全量重建。
- 构建：使用 `python tools/zircon_build.py --targets editor,runtime --out E:\zircon-build --mode debug` 验证 editor/runtime。
- 视觉：截图检查 Unreal 风格主界面为深灰紧凑布局，蓝色不再主导，面板/文字无重叠。

## Assumptions
- `.zui` 是正式新资产后缀，不做长期 `.v2.ui.toml` 双轨兼容。
- `.zui` 只表达组件原型；完整界面由运行时拼接。
- Unreal 风格只参考布局、密度、配色和交互结构，不复制 Unreal 商标、Logo 或专有资源。
