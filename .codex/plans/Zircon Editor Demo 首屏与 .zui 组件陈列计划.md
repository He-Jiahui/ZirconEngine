# Zircon Editor Demo 首屏与 `.zui` 组件陈列计划

## Summary
先不继续扩大战术性能修复，第一刀把 `zircon_editor.exe` 无参数启动改成组件 Demo 首屏：不再自动打开 last project，也不先进入 Workbench。显式 `--project` / `--create-project` 仍按用户命令进入项目。Demo 使用现有 `editor.ui_component_showcase` 作为首屏，并开始把陈列内容拆成 `.zui` 复合组件资产，视觉切到 Unreal 风格深色编辑器皮肤。

## Key Changes
- 启动路径：给 `EditorStartupSessionDocument` 增加“启动后打开内置视图”的请求字段；无参数 `resolve_startup_session()` 返回 `editor.ui_component_showcase`，跳过 last-project reopen 和 Welcome recent validation。
- Welcome Demo 按钮：从 `editor.material_demo_window` 改为 `editor.ui_component_showcase`，状态文案改为组件陈列；Workbench 保留为菜单/按钮入口。
- 组件资产：保留 `.v2.ui.toml` 作为 view/style schema；新增 `.zui` 只做 component prototype。`component_showcase.zui` 逐步改为导入 `.zui` 组件卡片/分组，而不是一整份大 view 内联所有控件。
- Unreal 风格：新增/切换 `editor_unreal_dark` 主题，组件 Demo 首屏采用 Unreal 类布局：顶部工具条、左侧分类、中心组件画廊、右侧属性/事件状态、底部日志/控制区。
- 菜单/操作：Window 菜单和 operation registry 增加 “UI Component Showcase”，保留 Material Demo 但不再作为默认 Demo。

## Test Plan
- 启动测试：无参数启动 session 断言首屏目标是 `editor.ui_component_showcase`，且不调用 last-project reopen；显式 `--project` 仍打开项目。
- 路由测试：Welcome 的 `OpenStartupDemo` 断言打开组件陈列，不再打开 Material Demo。
- 资产测试：`.zui` importer/cache 继续只接受单 component；组件陈列 view 能通过 `res://.../*.zui#ComponentName` 加载并复用 prototype store。
- 视觉验收：构建后运行 `zircon_editor.exe`，第一屏必须是组件陈列 Demo，不出现 Scene/Game/Hierarchy 工作台默认布局。
- 构建命令：`python tools/zircon_build.py --targets editor,runtime --out E:\zircon-build --mode debug`，必要时补 `cargo check -p zircon_app --features target-editor-host --locked`。

## Completion Status
- 启动路径已切到无参数首屏 `editor.ui_component_showcase`；显式项目请求仍保留项目路径优先。
- Welcome Demo 入口、Window 菜单和 operation registry 都已指向/暴露 UI Component Showcase；Material Demo 保留为显式入口。
- `component_showcase.zui` 保留 view/schema/shell 责任；展示页的 toolbar、category nav、Visual/Feedback、Input/Numeric、Selection/References、Collections/Inspector、state panel、bottom log 均已拆为单 component `.zui` prototype。
- 视觉主题已使用 `editor_unreal_dark.zui`，首屏呈现顶部工具条、左侧分类、中心组件画廊、右侧状态面板和底部日志。
- 新的视觉验收截图：`.codex/run-logs/editor-noargs-smoke-zui-gallery.png`。截图显示无参数启动进入 UI Component Showcase，而不是 Scene/Game/Hierarchy 默认工作台。

## Acceptance Evidence
- `cargo test -p zircon_editor --lib component_showcase --locked --target-dir target/codex-zui-state-panel --message-format short -- --test-threads=1`：2026-05-15 通过，25 passed。
- `cargo test -p zircon_editor --lib explicit_project_open_session_bypasses_component_showcase_builtin_view --locked --target-dir target/codex-zui-state-panel --message-format short -- --test-threads=1`：2026-05-15 通过，1 passed，确认显式项目打开不会附带 Demo 内置视图请求。
- `cargo test -p zircon_editor --lib create_project_and_open_persists_recent_project_and_returns_project_session --locked --target-dir target/codex-zui-state-panel --message-format short -- --test-threads=1`：2026-05-15 通过，1 passed，确认创建并打开项目仍返回 Project session 且不携带 Demo 内置视图请求。
- `cargo check -p zircon_editor --lib --locked --target-dir target/codex-zui-state-panel --message-format short`：2026-05-15 通过。
- `python tools/zircon_build.py --targets editor,runtime --out E:\zircon-build --mode debug`：2026-05-15 通过，构建输出已包含 `showcase_collections_section.zui` 等 `.zui` prototype。
- `python tools/zircon_build.py --targets editor,runtime --out E:\zircon-build --mode debug`：2026-05-15 09:57 复验通过，打包输出复制了 8 个 Showcase `.zui` 组件资产。
- `E:\zircon-build\ZirconEngine\zircon_editor.exe --list-operations --headless`：2026-05-15 通过，operation 列表包含 `Window.UiComponentShowcase.Open`。
- `E:\zircon-build\ZirconEngine\zircon_editor.exe --list-operations --headless`：2026-05-15 09:57 复验通过，operation 列表包含 `Window.UiComponentShowcase.Open`。
- `git diff --check` 针对本计划触及的展示页资产、测试、文档和计划文件通过，仅有 Windows LF-to-CRLF 提示。
- 2026-05-15 continuation cleared the stale template/runtime blockers found by GPU-plan workspace validation: current activity rail tests now use Hierarchy/Assets instead of the removed Project drawer entry, v2 host-window tests use `template.v2.ui.host_window`, hybrid pane bodies expose explicit root slots, and `PerformanceTimelinePaneBody/RefreshSnapshot` is registered. Focused gates passed for `pane_body_documents`, `drawer_toggle`, `repository_assets`, `dock_and_welcome`, `template_runtime`, `retained_callback_dispatch::template_bridge`, `floating_window_projection`, and the drawer resize bottom splitter route.
- `cargo test -p zircon_editor --lib --locked --jobs 1 --message-format=short -- --test-threads=1`：2026-05-15 continuation passed，1298 passed / 0 failed / 4 ignored.
- `cargo fmt --all -- --check`：2026-05-15 continuation passed.

## Assumptions
- “始终 Demo”指无参数启动；显式 CLI 项目请求优先。
- `.zui` 只承载复合组件 prototype，view/style 仍用 v2 TOML schema。
- Unreal 风格只参考布局、密度、色彩和编辑器交互，不复制 Unreal 商标或专有资源。
- 当前工作树已有大量未提交改动，实施时只改启动、Demo、`.zui` 组件陈列和对应测试，不回滚其他改动。
