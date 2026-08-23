---
handoff_kind: failure
status: open
created_at: 2026-07-13
summary_slug: plugin-viewport-overlay-provider-runtime-wiring
origin_plan: docs/plans/zircon_plugins/05-navigation.md
fixing_plan: docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
origin_child_dir: docs/plans/zircon_plugins/05
fixing_child_dir: docs/plans/zircon_editor/editor/05
plan_link_mode: child_record_only
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

Navigation 已声明稳定 provider id，并实现 toggle controller 与 `NavigationViewportGizmoSink`。Editor host 已硬切为 `SceneModeRegistration` 与独立 overlay provider factory 的可执行安装契约，不再接受仅保存字符串的 descriptor-only tool mode；Navigation 仍需在其责任计划内注册真实 provider 并提供端到端显示/清除证据。toggle operation 同时受 Editor 03 factory 缺口影响。离线构造测试不能证明 viewport 实际显示或清除 overlay。

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

Open state: `host wiring 实现完成，端到端仍待 Plugins05`; host 已验证并安装 provider registration、按 capability gate toggle，并把 provider/mode gizmo 合并到 render 与 pointer 共用的 immutable interaction extract。扩展 scene mode 入口也已 hard-cut 为 executable registration，descriptor-only provider string 不再能冒充运行时 mode。Navigation 仍没有注册可实例化 provider，也没有发布包含 NavMesh 与 PIE frame 的 canonical overlay frame；该最低插件原因由当前 [Plugins05 navigation overlay frame handoff](failure-2026-07-30-navigation-overlay-frame-publication.md) 负责。Editor05 不创建全局 overlay cache 或插件专用 bypass；待 Plugins05 返回真实 provider/frame 后，以 host toggle -> shared extract 的受管产品验证恢复本 handoff。

## 产出记录与时间

| 日期 | 项目 | 状态 | 证据 |
| --- | --- | --- | --- |
| 2026-07-31 | viewport overlay provider host audit | open | Editor05 registry 已具备 duplicate validation、factory install、capability disable cleanup、toggle 及 `RenderOverlayExtract.scene_gizmos` 合并；Navigation `register_navigation_overlay` 仍只声明 descriptor/provider id，仓库没有 Navigation `register_viewport_overlay_provider` consumer。真实 frame/provider 缺口已链接至 Plugins05；端到端 managed product gate 保持 open。 |
| 2026-08-01 | executable host registry + shared pick/render extract | implementation_complete / managed_validation_pending | provider install/toggle/capability changes invalidate the shared interaction cache；provider 与 mode gizmo 在 cache rebuild 内合并，render/pointer 同消费一份 `Arc`。descriptor-only scene-mode API 已删除；Navigation pseudo mode 同步移除，真实 provider/frame 仍由 Plugins05 failure 负责。 |
