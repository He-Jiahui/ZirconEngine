---
handoff_kind: fixed
status: fixed
created_at: 2026-07-14
summary_slug: ui-text-project-asset-manager-access-consumer-drift
origin_plan: docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
fixing_plan: docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
origin_child_dir: docs/plans/zircon_runtime/runtime/15
fixing_child_dir: docs/plans/zircon_runtime/frameworks/05
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/access.rs
tests:
  - managed default-feature cargo build -p zircon_runtime --locked --jobs 1
  - current-source UI text manager-access resolver guard
resolved_at: 2026-07-14
---


# Frameworks 05: UI text ProjectAssetManager access consumer drift

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md`
- 来源执行切片：Runtime15 Render owner budget split managed build gate
- 修复责任计划：`docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md`
- 交接原因：最低共享根因是 Frameworks05 versioned manager access hard-cut 后，UI text 构造调用点仍把 access object 当作具体 Arc manager 使用。

## 失败现象与复现证据

managed ephemeral job `ff2fa0c62ede4e858ef24e01382c4263` 在 default-feature `zircon_runtime` 编译阶段报 E0308：`ui/construct.rs:81` 传入 `ProjectAssetManagerAccess`，`ScreenSpaceUiTextSystem::new` 要求 `Arc<ProjectAssetManager>`；同一表达式继续对返回 `ScreenSpaceUiTextSystem` 使用 `?`，触发 E0277。预期是 UI renderer 在真实构造边界解析 access，并继续把 concrete manager 交给 text subsystem。

## 最低共享层根因

`ScreenSpaceUiRenderer::new` 已完成参数 hard-cut，却漏掉 `ProjectAssetManagerAccess::resolve` 的最低真实调用，且保留了旧 fallible text constructor 的 `?` 形态。

## 架构修复验收

- UI renderer 构造边界显式解析 versioned manager access，错误映射到 `GraphicsError::Asset`。
- `ScreenSpaceUiTextSystem` 继续接收其实际需要的 concrete manager；不增加 Arc adapter、resolver shim 或备用 owner。
- 原始 managed default-feature `zircon_runtime` build gate 重新运行并越过 E0308/E0277。

## 禁止临时方案

- 不增加 alias、compatibility shim、silent fallback、duplicated truth、test-only bypass 或调用点例外。
- 不恢复 `IntoProjectAssetManagerAccess` 或旧 asset resolver。
- 不削弱 Runtime15/Frameworks05 验收门来隐藏失败。

## 修复结果与回传

- 根因：Frameworks05 hard-cut the UI renderer input to ProjectAssetManagerAccess, but ScreenSpaceUiRenderer::new still passed the access object to a concrete Arc<ProjectAssetManager> consumer and retained an obsolete fallible-constructor question mark.
- 架构修复：Resolve ProjectAssetManagerAccess exactly once at the UI renderer construction boundary, map resolution failure to GraphicsError::Asset, and pass the bounded concrete manager to ScreenSpaceUiTextSystem without restoring any resolver shim, Arc adapter, fallback, or duplicate owner.
- 验证：Managed Windows default-feature cargo build -p zircon_runtime --locked passed in job 9dac70c034fb4aa18155d370f77073e1 after the original job ff2fa0c62ede4e858ef24e01382c4263 reproduced E0308 and E0277; scoped rustfmt passed.
- 回传：Frameworks05 corrected the lowest real UI text manager-access consumer and returns the verified fix to Runtime15.

## 后续独立复审

本 artifact 只证明 E0308/E0277 constructor consumer drift 已被消除。后续独立 review 发现 `ScreenSpaceUiTextSystem` 仍跨帧保存 concrete manager，因此更深层生命周期问题进入 Frameworks05；该问题现已通过 [`fixed-2026-07-14-ui-text-manager-access-cross-frame-retention.md`](fixed-2026-07-14-ui-text-manager-access-cross-frame-retention.md) 完成架构修复与回传。完整 manager-access lifetime 验收以后一份 fixed artifact、精确一次解析守卫和最终独立复审为准。
