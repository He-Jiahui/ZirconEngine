# Zircon Editor/Runtime UI Rust-Owned Retained Host 重构计划

## Summary
- 将 `zircon_editor` 从当前 Slint 编辑器壳硬切到 Zircon 自己的 retained-mode UI 宿主；不保留长期兼容层。
- 以 Fyrox 为主参考：状态化 retained UI tree、消息队列、冒泡/直达路由、DockingManager、Inspector/属性编辑器模式。
- 以 Bevy 为辅参考：无样式 headless widgets、外部状态驱动、事件/observer 风格、focus/picking/a11y 分层。
- 复用并增强现有 `zircon_runtime::ui` 作为编辑器和运行时共享 UI 核心；`zircon_editor` 只保留编辑器状态、Workbench、场景/资产工具逻辑。
- UI 视觉改为接近 Fyrox 的紧凑深色工具界面：低圆角、密集面板、清晰 dock/tab/inspector 层级、克制蓝灰强调色。

## Key Changes
- `zircon_editor`
  - 删除 `ui::slint_host` 编辑器主壳、Slint build/asset 绑定、Slint 专用 pointer/data bridge。
  - 新增 Rust-owned retained UI host，例如 `ui::retained_host`：负责 winit 事件循环、窗口、输入/IME、frame scheduler、viewport composition、WGPU UI 提交。
  - 保留并迁移现有 `EditorManager`、`EditorUiHost`、`WorkbenchLayout`、`EditorState`、snapshot/layout 模型，让它们输出 shared UI tree/message 而不是 Slint rows/callbacks。
  - Workbench 重建为 retained UI：activity rail、dock panels、document tabs、viewport toolbar、hierarchy、inspector、asset browser、status bar、menu/command palette。
  - Inspector 改为 reflection/property-editor 驱动，参考 Fyrox inspector；资产浏览器和层级树必须虚拟化，避免大项目下全量 widget rebuild。
- `zircon_runtime` / `zircon_runtime_interface`
  - 将现有 `UiSurface`、incremental layout、hit test、focus、navigation、render extract 扩展为通用 retained UI 引擎。
  - 引入或收敛统一消息协议：`UiMessage`、direction、routing、delivery/handled 语义，优先复用现有 `dispatch` 模块，避免重复协议。
  - 补齐 headless widget 行为层：button、checkbox、slider、text input、tree/list、tab、dock、menu、property field；样式由 theme/token 决定。
  - 渲染路径从 command cache 升级为 batch/cache driven：paint element、batch key/plan、damage rect、glyph/SDF atlas、icon/image atlas、persistent GPU buffers。
  - 运行时游戏 UI 继续使用 `.ui.toml`/template 能力，但内部走同一 retained surface、layout、message、renderer 管线。
- Theme / Assets
  - 新增 Fyrox-like editor theme，替换当前 Material 取向 token 命名和视觉默认值。
  - 编辑器 UI 资源归 `zircon_editor/assets/ui/editor`，运行时 UI 资源归 runtime 侧；生产资源不放在 `src` 下。
  - 删除 Slint 专用 UI 资源、生成代码入口和 CI 中不再需要的 Slint 依赖。

## Performance Requirements
- 鼠标 hover、selection、scroll、tab drag 不允许触发全 UI tree rebuild；只更新 dirty nodes、dirty layout roots 和受影响 render batches。
- frame scheduler 只在 UI dirty、viewport 有新帧、动画/timer 到期或输入事件产生时 request redraw；pointer move 要合并。
- Dock/tree/list/asset grid/inspector 必须支持虚拟化和池化，目标场景包括 10k hierarchy nodes、1k inspector fields。
- UI renderer 使用持久 vertex/index buffer、批次合并、scissor、glyph/SDF atlas cache、icon/image atlas cache；暴露 batch count、cache hit/miss、dirty node count、layout/render elapsed micros。
- Idle 状态不应持续重排、重建 render commands 或重传大 buffer。

## Test Plan
- Contract tests：UI message routing、handled/stop propagation、focus/navigation、widget external-state events、dock layout serialization、theme token resolution。
- Runtime tests：保留并扩展现有 runtime UI tests、text render contract、runtime-ui integration captures、clipping/wrapping/SDF/native text/image/icon cases。
- Editor tests：retained host input/IME、dock save/restore、tab drag/drop、menu command dispatch、hierarchy virtualization、inspector property edit、asset browser filtering、viewport pointer routing。
- Visual captures：1280x720、1920x1080、高 DPI 下验证 Fyrox-like chrome、文本、图标、hover/pressed/selected、dock split、floating panel、无重叠。
- Stress/perf tests：10k hierarchy nodes、1k inspector fields、连续 scroll/hover/resize/tab drag、60s idle no rebuild；断言 dirty/skipped/cache ratio。
- Final validation：运行 `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1`，并覆盖 workspace build/test、plugin workspace check/build/test、runtime-ui feature tests。

## Assumptions
- 已确认允许替换 Slint 编辑器主界面；本计划不设计 Slint 兼容模式。
- 不新增根 crate；继续使用固定结构：`zircon_app`、`zircon_runtime`、`zircon_editor`、`zircon_runtime_interface`。
- “接近 Fyrox 风格”指密集深色编辑器工具界面与交互结构，不复制 Fyrox 资产或逐像素皮肤。
- Bevy 作为 widget 行为和状态分离参考；Fyrox 作为 retained UI、消息、dock、inspector 主参考。
- 旧 Slint tests/fixtures 在对应 retained UI 功能落地后删除或重写，不通过 shim 保留。
