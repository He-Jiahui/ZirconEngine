---
handoff_kind: failure
status: open
created_at: 2026-07-13
summary_slug: plugin-viewport-overlay-provider-runtime-wiring
origin_plan: docs/plans/zircon_plugins/05-navigation.md
fixing_plan: docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
origin_child_dir: docs/plans/zircon_plugins/05
fixing_child_dir: docs/plans/zircon_editor/editor/05
related_code:
  - zircon_editor/src/core/editor_authoring_extension.rs
  - zircon_editor/src/core/editor_extension.rs
  - zircon_editor/src/scene/viewport/render_packet.rs
  - zircon_plugins/navigation/editor/src/overlay.rs
tests:
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_plugin_navigation_editor -TargetDir E:/cargo-targets/zircon-navigation-m6-editor -SkipBuild
---

# Editor 05：插件 viewport overlay provider 宿主接线缺失

## 来源执行者

- 来源计划：`docs/plans/zircon_plugins/05-navigation.md`
- 来源执行切片：M6-T2 NavMesh viewport overlay
- 修复责任计划：`docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md`
- 交接原因：最低共享原因是 viewport 仅消费内建 scene gizmos，没有插件 provider registry/factory 与每帧 extract 合并点。

## 失败现象与复现证据

Navigation 已声明稳定 provider id，并实现 toggle controller 与 `NavigationViewportGizmoSink`，但当前 `ViewportToolModeDescriptor` 只保存字符串；host 不解析 provider id，也不安装 controller/sink。toggle operation 同时受 Editor 03 factory 缺口影响。离线构造测试不能证明 viewport 实际显示或清除 overlay。

## 最低共享层根因

Editor viewport/tool-mode 扩展模型缺少 provider 实例注册、capability 生命周期、每帧 extract 调用与 `RenderOverlayExtract.scene_gizmos` 合并契约。

## 架构修复验收

- Editor 05 提供共享 viewport overlay provider registry/factory；tool-mode descriptor 的 provider id 必须解析到唯一实例。
- provider 按 capability、view/tool-mode enabled state 和 plugin lifecycle 启停；每帧 extract 合并到共享 gizmo channel，关闭时清除旧 extract。
- Navigation 注册真实 provider；host 测试从 menu/command toggle 到 render packet，断言 area、agent path 与 avoidance extract 出现和消失。

## 禁止临时方案

- 禁止在 Navigation 中直接修改 viewport state、仅保存 provider 字符串、全局开启所有 gizmos或使用 test-only sink 冒充 host wiring。
- 禁止 aliases、compatibility shims、silent fallback、duplicated truth、test-only bypasses 或 call-site exceptions。
- 禁止削弱测试或 M6 验收标准以隐藏失败。

## 修复结果与回传

Open state: `待修复`; no pass is claimed.
