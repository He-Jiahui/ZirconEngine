---
handoff_kind: fixed
status: fixed
created_at: 2026-07-14
summary_slug: runtime-text-ui-system-constructor-drift
origin_plan: docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md
fixing_plan: docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md
origin_child_dir: docs/plans/zircon_editor/editor_layout/15
fixing_child_dir: docs/plans/zircon_runtime/text/01
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/access.rs
  - zircon_editor/src/tests/host/retained_menu_pointer/visual_screenshot/blend_space_workspace.rs
tests:
  - .\\.codex\\skills\\zircon-dev\\scripts\\validate-matrix.ps1 -Package zircon_editor -SkipBuild -VerboseOutput
  - focused Blend Space preview contract and ignored screenshot capture after Runtime Text repair
resolved_at: 2026-07-14
---


# Runtime Text 01：UI text system 构造器与资产访问接口漂移

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md`
- 来源执行切片：S15.4/S15.5 Blend Space Preview toolbar 原子组件密度与当前源码截图门禁。
- 修复责任计划：`docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md`
- 交接原因：Layout 15 必须消费 Runtime Text 的共享文字与资源接口，不能通过 editor 私有 painter、旧资源入口或复用旧截图绕开下层编译失败。

## 失败现象与复现证据

2026-07-14 在 Windows coordinator 受管 target 上执行：

```powershell
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_editor -SkipBuild -VerboseOutput
```

命令已通过 coordinator 生命周期检查并进入 Cargo，但编译 `zircon_runtime` 失败，故没有当前源码的 `zircon_editor` 测试二进制，也没有刷新截图。

`zircon_runtime/src/graphics/scene/scene_renderer/ui/construct.rs:81` 调用：

```rust
ScreenSpaceUiTextSystem::new(asset_manager, device, queue, target_format)?
```

其中 `asset_manager` 是 `ProjectAssetManagerAccess`；当前 `text.rs:105` 签名要求 `Arc<ProjectAssetManager>` 并直接返回 `ScreenSpaceUiTextSystem`。编译器报告 E0308（参数类型不匹配）和 E0277（无效的 `?`）。

## 最低共享层根因

Runtime Text 构造器的资源所有权和错误返回契约已经迁移，但 screen-space UI 构造调用点未同步收束。接口漂移发生在 `zircon_runtime` 的共享 UI text/asset-manager 边界，早于 editor retained-host、布局约束和生产窗口截图执行。

## 架构修复验收

- 为 `ScreenSpaceUiTextSystem::new` 明确一个稳定的资源访问边界：调用方解包为 `Arc<ProjectAssetManager>`，或构造器在 Runtime Text 层接受并验证 `ProjectAssetManagerAccess`。
- 让调用点与返回类型一致：移除无效 `?`，或恢复被明确建模的 `Result`；不得用 `unwrap`、类型擦除或 editor 专用转换隐藏契约差异。
- 受管 `zircon_editor` 矩阵重新通过后，来源切片必须执行 focused Blend Space Preview 合同和 ignored screenshot capture；三张验证图仅能更新到 `docs/tests/editor`。

## 禁止临时方案

- 不得在 Layout 15 重新引入私有 text painter、旧 `Arc<ProjectAssetManager>` 旁路或未验证的 access unwrap。
- 不得跳过当前源码 Cargo 验证、复用旧测试二进制，或把 7 月 13 日旧截图标作本次验收。
- 不得把截图写入任何 `target` 目录，或因该编译错误缩减 Preview 的共享 Chip/相对布局设计。

## 修复结果与回传

- 根因：Screen-space UI construction passed ProjectAssetManagerAccess to a TextSystem constructor that still required Arc<ProjectAssetManager> and returned a non-Result, leaving the call site and constructor contracts out of sync.
- 架构修复：The constructor now accepts ProjectAssetManagerAccess and returns Result<Self, CoreError>; it resolves only for bounded initialization, retains the versioned access handle, and propagates preparation failures through GraphicsError::Asset in both render paths.
- 验证：Dedicated Frameworks05 lifetime guard PASS 1/1; scoped rustfmt and git diff check PASS; cargo build -p zircon_runtime --locked PASS (job c2db4e7bfe0647678e6334648b6df811); managed zircon_editor job 6af3a291b7754a6d86a91f61fe56e12e has compiled past the former E0308/E0277 site, with the origin plan retaining its separate V2 high-CPU/screenshot follow-up.
- 回传：Returned to Editor Layout15: the shared Runtime Text constructor and asset-access lifetime contract are converged; the original constructor mismatch is removed. Layout15 may continue its focused preview and real screenshot gates, while its previously documented V2 high-CPU issue remains separate.
