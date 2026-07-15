---
handoff_kind: fixed
status: fixed
created_at: 2026-07-14
summary_slug: ui-text-manager-access-cross-frame-retention
origin_plan: docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
fixing_plan: docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
origin_child_dir: docs/plans/zircon_runtime/runtime/15
fixing_child_dir: docs/plans/zircon_runtime/frameworks/05
related_code:
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/access.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_scene/render_scene.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/surface.rs
tests:
  - python -m unittest tools.tests.test_frameworks_05_manager_access_lifetime -v
  - managed Windows default-feature cargo build -p zircon_runtime --locked
resolved_at: 2026-07-14
---


# Frameworks 05: UI text manager access cross-frame retention

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md`
- 来源执行切片：Runtime15 Render owner budget split independent review
- 修复责任计划：`docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md`
- 交接原因：最低共享根因属于 Frameworks05 versioned manager access 生命周期契约；UI text 长期对象仍保存 concrete manager，不能由 Runtime15 结构预算守卫局部绕过。

## 失败现象与复现证据

独立 review 发现 `ScreenSpaceUiRenderer::new` 在构造期解析 `ProjectAssetManagerAccess`，随后 `ScreenSpaceUiTextSystem` 把 `Arc<ProjectAssetManager>` 保存为字段并跨帧在 `prepare` 中使用。代码虽然消除了 E0308/E0277，却违反运行时计划“业务对象不缓存跨生命周期直接强引用”以及 `core::manager` 的 bounded-operation 规则。

修复后的首次 managed Windows default-feature Runtime build 曾在 `zircon_runtime_interface/src/serialization/binary/**` 被 Editor11 的 11 个 E0364/E0365/E0603 阻断。该外部 Failure 已由责任会话解除；随后 managed job `c2db4e7bfe0647678e6334648b6df811` 完成 `cargo build -p zircon_runtime --locked` 并返回 0。专门 lifetime guard、scoped rustfmt 与 diff check 均通过，Failure 已在 2026-07-14 原子回传；完整 Runtime 套件只作为更宽验证继续记录，不改变本 fixed 状态。

## 最低共享层根因

先前修复把“能把 access 传给 concrete consumer”误判为最终 use point。真实 use point 是 text-system 的初始化资产读取和每帧 text prepare；长期 renderer/text owner 只能保存 versioned access，不能保存解析后的 concrete Arc。

## 架构修复验收

- `ScreenSpaceUiTextSystem` 保存 `ProjectAssetManagerAccess`，不保存 `Arc<ProjectAssetManager>`。
- text-system 构造初始化与每帧 `prepare` 各自解析一次 access，concrete Arc 只活在对应 bounded operation 内。
- resolve failure 通过 `GraphicsError::Asset` 沿普通 scene render 和 render-graph executor 两条真实调用链向上传播。
- current-source lifetime guard、原始 default-feature Runtime build 与 Runtime15 Render owner gates 全部通过。

## 禁止临时方案

- 不增加 alias、compatibility shim、silent fallback、duplicated truth、test-only bypass 或调用点例外。
- 不在 renderer/text/resource streamer 中缓存 concrete manager，且不吞掉 stale/unavailable manager 错误。
- 不削弱 Runtime15/Frameworks05 守卫或把跨帧 Arc 标记为豁免。

## 修复结果与回传

- 根因：TextSystem retained Arc<ProjectAssetManager> across frames after constructor resolution, bypassing Frameworks05 versioned manager access lifetime and making manager replacement invisible.
- 架构修复：TextSystem now stores ProjectAssetManagerAccess, resolves once during construction for default font loading, resolves once per prepare use-point, and maps resolution failures through GraphicsError::Asset in direct and render-graph UI paths.
- 验证：Dedicated lifetime guard PASS 1/1; scoped rustfmt and git diff check PASS; cargo build -p zircon_runtime --locked PASS (job c2db4e7bfe0647678e6334648b6df811); current zircon_editor validation compiles beyond the former TextSystem constructor mismatch.
- 回传：Returned to Runtime15: versioned manager access is retained across frames, concrete Arc lifetime is bounded to each operation, and both rendering paths propagate stale-access failure.
