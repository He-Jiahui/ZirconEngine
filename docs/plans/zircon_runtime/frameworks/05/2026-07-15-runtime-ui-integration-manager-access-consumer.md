---
related_code:
  - zircon_runtime/src/tests/graphics_surface/runtime_ui_integration.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/access.rs
  - zircon_runtime/src/ui/public_runtime_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/render_framework_trait_binding/wgpu_framework.rs
  - zircon_runtime/src/graphics/runtime/render_framework/wgpu_render_framework_construction/construct.rs
implementation_files:
  - zircon_runtime/src/tests/graphics_surface/runtime_ui_integration.rs
plan_sources:
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
tests:
  - rustfmt --edition 2021 --check zircon_runtime/src/tests/graphics_surface/runtime_ui_integration.rs
  - git diff --check -- zircon_runtime/src/tests/graphics_surface/runtime_ui_integration.rs
  - source invariant scan for zero concrete ProjectAssetManager WgpuRenderFramework constructor consumers
  - managed Windows cargo test -p zircon_runtime --lib --features runtime-ui-integration-tests tests::graphics_surface::runtime_ui_integration::render_framework_submits_runtime_ui_frames_and_renders_pause_menu_panels --locked -- --exact --nocapture
doc_type: milestone-detail
---

# Frameworks05 Runtime UI Integration Manager Access Consumer

Plan: docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
Parent milestone: M4
Status: focused_cargo_passed
Date: 2026-07-15
Files: ["docs/plans/zircon_runtime/frameworks/05/2026-07-15-runtime-ui-integration-manager-access-consumer.md", "zircon_runtime/src/tests/graphics_surface/runtime_ui_integration.rs"]

## 状态与完成项目

| 切片 | 状态 | 完成证据 |
|---|---|---|
| runtime UI optional-feature render consumer hard cut | `focused_cargo_passed` | `runtime-ui-integration-tests` 下三个 `WgpuRenderFramework::new` 调用均改为消费 `ProjectAssetManagerAccess`；三个 frame submit 调用改为正式 `RenderFramework::submit_frame_extract_with_ui` 边界，不再依赖 concrete manager 或已删除的 `PublicRuntimeFrame -> ViewportRenderFrame` 隐式转换。受管 job `c036050b99ef4ceca9429cba7445e4f3` 实际执行目标测试并得到 1/1 通过。 |

## 架构修复

三个 runtime UI 产品测试原本直接创建 `Arc<ProjectAssetManager>` 并传入 render framework。
该调用形态绕过 versioned manager handle 与 bounded resolve contract，只因测试默认未启用而未被默认
feature matrix 编译发现。

当前调用点复用 crate 内已有的 `ProjectAssetManagerAccess::for_test`。该测试专用入口会创建真实
`CoreRuntime`、注册并激活 manager service、取得 versioned handle，并把 runtime owner 保存在 access
中。测试因此覆盖与生产一致的 service-resolution 生命周期，不新增 standalone manager adapter、
compatibility shim 或公开测试 API。

第一次可选特性编译越过 manager constructor 后继续暴露三个 E0277：测试仍通过 `.into()` 依赖
已经删除的 `PublicRuntimeFrame -> ViewportRenderFrame` 转换。当前 consumer 显式校验 UI frame 的
viewport size，并把 `RenderFrameExtract` 与 `UiRenderExtract` 提交给正式
`RenderFramework::submit_frame_extract_with_ui` 契约；没有恢复 graphics-internal frame conversion。

## 验证结果

- 红态 job `6eef52d812a84d4ba356aeeb6bf0a0bf` 越过 manager constructor 后报告三个
  `PublicRuntimeFrame -> ViewportRenderFrame` E0277，并同时暴露活动 Dynamic API owner 的外部编译错误。
- 绿态 job `c036050b99ef4ceca9429cba7445e4f3` 在 Dynamic API 下层恢复后完成
  `runtime-ui-integration-tests` lib-test 编译，目标测试 1/1 通过，8114 项被过滤，退出码 0。
- scoped rustfmt、diff check 与 source invariant scan 通过；旧 concrete-manager 构造和 stale
  frame `.into()` 调用均为 0。

本切片不声明 Frameworks05、Runtime15 或整个 Runtime package 已完成。
