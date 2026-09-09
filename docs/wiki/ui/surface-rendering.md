---
related_code:
  - zircon_runtime/src/ui/surface/mod.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/surface/invalidation.rs
  - zircon_runtime/src/ui/surface/render/extract.rs
  - zircon_runtime_interface/src/ui/surface/frame.rs
implementation_files:
  - zircon_runtime/src/ui/surface/render/cache.rs
  - zircon_runtime/src/ui/surface/node_pool.rs
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine UI Wiki
tests:
  - zircon_runtime/src/ui/tests/surface_frame_authority.rs
  - zircon_runtime/src/ui/tests/surface_dirty_domains.rs
  - zircon_runtime/src/ui/tests/text_pipeline/render_extract_prewarm.rs
doc_type: module-detail
---

# 表面与渲染

## UiSurface 状态

`UiSurface` 聚合运行时树、arranged tree、节点池、组件状态、焦点/滚动/弹窗栈、输入状态、文本测量缓存、样式索引及 invalidation generations。`UiSurfaceNodePool` 和 `UiSurfaceComponentStateStore` 维持重建后的身份与状态。

## 脏域与重建

属性、布局、内容、输入可见性和渲染修改分别设置 `UiDirtyFlags`，事务提交生成 `UiInvalidationCommit`。重建报告 (`UiSurfaceRebuildReport`) 记录原因、节点数量及回退；只要 arranged 几何未变化，可跳过完整 Taffy pass。

## 渲染抽取

`extract_ui_render_tree` 从表面得到渲染节点；文本节点经过 shaping、glyph raster/atlas 及 `UiResolvedTextGlyphArtifactLine` 提交。`extract_ui_render_tree_from_arranged` 用于已排布树，保证 frame authority 不被旧 authored geometry 覆盖。调试 API 可输出命中、选中和时间线快照。

```rust
use zircon_runtime::ui::surface::{extract_ui_render_tree, hit_test_surface_frame};
// 独立抽取 API 接受 UiTree；保留式 UiSurface 则在自身生命周期内维护 extract。
let extract = extract_ui_render_tree(&surface.tree);
let published_frame = surface.surface_frame();
let hit = hit_test_surface_frame(&published_frame, cursor_position);
```

## 限制

渲染线程只消费抽取帧；不要从渲染回调直接变更树。字体/图标 atlas 未就绪时会产生预热请求，表面可先提交占位图元并在下一帧刷新。
