---
related_code:
  - zircon_runtime/src/core/framework/render/framework.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/core/framework/render/surface.rs
  - zircon_runtime/src/graphics/runtime/render_framework/mod.rs
  - zircon_runtime/src/graphics/runtime_builtin_graphics/mod.rs
implementation_files:
  - zircon_runtime/src/graphics/runtime/render_framework/wgpu_render_framework_construction/construct.rs
  - zircon_runtime/src/graphics/runtime/render_framework/create_viewport/create.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_surface
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine 详细 Wiki 文档集合
  - docs/assets-and-rendering/render-framework-architecture.md
tests:
  - zircon_runtime/src/graphics/tests/render_product_submit
  - zircon_runtime/src/graphics/tests/render_product_camera_targets
  - zircon_runtime/src/graphics/tests/surface_targets
doc_type: workflow-detail
---

# 渲染框架与视口

## 功能说明

`RenderFramework` 是应用、编辑器和运行时访问渲染能力的首选接口。它管理逻辑视口，而不是窗口本身：一个视口可以离屏渲染、绑定 native Surface、输出到纹理目标、生成 CPU capture，或发布给 retained UI 使用的共享 GPU 产品。

当前生产实现为 `WgpuRenderFramework`。`SceneRenderer` 虽然从 `graphics` 门面公开了类型，但大部分直接渲染方法为 crate 可见；外部调用者应使用 `RenderFramework` trait。

## 视口生命周期

### 创建

`create_viewport(RenderViewportDescriptor)` 返回 `RenderViewportHandle`。Descriptor 决定初始尺寸和视口级配置；真实输出尺寸还可能受 camera viewport rect、dynamic resolution 和输出目标影响。

### 提交

- `submit_frame_extract`：离屏提交场景。
- `submit_frame_extract_with_ui`：离屏提交场景并附加 screen-space UI。
- `present_frame_extract`：向已绑定 Surface 提交并呈现。
- `present_frame_extract_with_ui`：窗口呈现并附加 UI。

提交成功表示框架接受了该帧，不必然表示 GPU 已完成。流水模式尤其如此。

### 管线和质量

- `set_pipeline_asset(viewport, pipeline)` 切换 Core2D、Forward+、Deferred 或已注册的自定义管线。
- `set_quality_profile(viewport, profile)` 改变 feature 质量与降级策略。
- `reload_pipeline(pipeline)` 让已注册管线重新编译。

### Surface

`bind_viewport_surface` 将宿主提供的 native surface descriptor 绑定到视口；`unbind_viewport_surface` 解除。窗口 resize、lost/outdated、零尺寸属于 Surface 生命周期，不应通过创建普通离屏纹理模拟。底层 RHI 对零 extent 返回 typed non-renderable outcome。

### 销毁

`destroy_viewport` 终止视口所有权。调用者不应继续使用旧 handle；与旧帧关联的 GPU 完成回收由框架内部处理。

## 典型 Rust 调用

```rust
use std::sync::Arc;

use zircon_runtime::core::framework::render::{
    RenderFramework, RenderFrameExtract, RenderViewportDescriptor,
};
use zircon_runtime::graphics::WgpuRenderFramework;

fn submit_frame(
    framework: &WgpuRenderFramework,
    descriptor: RenderViewportDescriptor,
    extract: RenderFrameExtract,
) -> Result<(), Box<dyn std::error::Error>> {
    let viewport = framework.create_viewport(descriptor)?;
    framework.submit_frame_extract_with_ui(viewport, extract, None::<Arc<_>>)?;

    // query_stats 是运行时统计快照，不等同于等待 GPU 完成。
    let _stats = framework.query_stats()?;
    framework.destroy_viewport(viewport)?;
    Ok(())
}
```

实际工程通常由 runtime host 创建并注册框架，而不是在功能代码里直接构造 `WgpuRenderFramework`。`GraphicsModule` 的 descriptor 注册 `RENDER_FRAMEWORK_NAME` 和 `RENDERING_MANAGER_NAME` 服务。

## 捕获、拾取与查询

| API | 语义 | 注意事项 |
| --- | --- | --- |
| `capture_frame` | 读取最近帧的 SDR CPU 像素 | 可能触发等待/读回，调试与离线用途 |
| `poll_captured_frame_if_newer` | 仅取已完成的新 capture | 非阻塞，`None` 可表示未完成 |
| `capture_scene_color_hdr` | 读取保留的线性 HDR scene color | 后端可返回 UnsupportedCapability |
| `poll_viewport_product_if_newer` | 获取共享 GPU 产品身份 | 不要求 CPU readback |
| `request_viewport_pick` / `poll_viewport_pick` | 针对确定帧 generation 的异步拾取 | ticket 可能 pending，不能忙等 |
| `query_visible_spatial_snapshot` | 查询 renderer-visible 空间快照 | 与完整 World 查询不同 |
| `request_graphics_debugger_capture` | 请求下一次图形调试捕获 | 能力不可用时 status 会说明原因 |

## 提交配置

`RenderSubmissionConfig` 将执行策略与 feature profile 分开。当前实现可控制同步/流水提交、GPU timing、异步 pipeline compile、并行录制阈值和 HZB 诊断读回。启用诊断会增加 query/readback 和 CPU 路由成本，不应作为发行默认设置。

## 环境捕获工作流

`request_environment_capture` 接受场景 packet 和请求，返回 handle；调用者用 `poll_environment_capture` 非阻塞轮询，完成后可 `take_environment_capture_source_payload` 移交持久化数据。取消只对仍可取消的阶段生效。该路径在 WGPU 框架有真实实现，但属于按需 GPU 工作，不是每帧默认执行。

## 错误处理

所有跨框架错误归一到 `RenderFrameworkError`。常见类别包括未知/失效视口、后端错误、能力不支持、管线或资源无效、设备故障。不要根据错误字符串分支；匹配 typed variant。trait 的默认方法常返回 `UnsupportedCapability`，因此自定义后端即使实现了基础提交，也不自动拥有 capture、Surface、pick 或环境捕获。
