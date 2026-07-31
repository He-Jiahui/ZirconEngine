---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render/tests.rs
  - zircon_runtime_interface/src/ui/surface/render/command.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_layout/21-gpu-submission-and-draw-pipeline.md
  - docs/plans/zircon_runtime/render/14-2d-stack.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
  - docs/plans/zircon_runtime/text/09-threading-caching-and-performance.md
tests:
  - ui render root file 1 of 1 reviewed, 835 current lines
  - cross-file source guards prove one paint projection per command and exact empty LoadStore skip
  - rustfmt check and git diff check passed for render.rs plus focused test file
  - focused Cargo, editor pixels, F2 and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics UI render root逐文件性能静态审查（2026-07-18）

## 范围与结论

已完整阅读`scene_renderer/ui/render.rs`当前1/1个root Rust文件、835行，并追踪`UiRenderCommand::to_paint_elements`、`render/text_paint.rs`、text decoration、image prepare和text prepare report调用链。`render/tests.rs`已在后续完整子目录证据中计数。

`prepare_screen_space_ui`每frame重新物化vertices、draws、post-text draws、auto/native/SDF text和images七组Vec。每条command调用`to_paint_elements`时会重建paint payload；每个paint element又把完整command序列化为JSON计算stable hash、格式化kind debug label，并为text投影拥有的String/decoration数据。后续text batch继续clone line String、glyph advances、font/family和decorations，`text_prepare_report()`也返回deep clone。稳定UI generation没有根计划命中。

本轮安全止损让root decoration与`text_paint` helper共享同一组paint elements：helper不再隐式二次调用`to_paint_elements`，每条command真实调用链只投影一次。空UI路径也改为只在精确LoadStore时跳过pass；Clear/Discard仍录制空render pass，保留render-graph attachment语义。跨文件source guard、`rustfmt --check`和`git diff --check`已绿；Cargo尚未获得可消费执行槽，不能据此进入`review.md`。

新增PERF-MVP-398：EditorLayout21发布唯一generation-owned ordered prepared plan，Render14直接消费共享paint/text/image handles与persistent geometry arena，Text09把sealed report/line artifact改为Copy/Arc或借用，Render17提供投影、serde/hash/debug、clone、Vec和GPU upload计数。最终不能在scene renderer里从原始command再建第二份权威paint计划。

本地参考已核实：Bevy `dev/bevy/crates/bevy_ui_render/src/lib.rs`的`ExtractedUiNodes`（375）、`UiBatch`（1486）与`prepare_uinodes`（1575）分离extract/prepare；UE `ElementBatcher.h`的`MergeRenderBatches`（192）/`FSlateElementBatcher`（245）和`SlateRenderBatch.h::IsBatchableWith`（30/145）明确由prepared batch data与兼容键控制提交。参考目标是单一prepared authority和有序兼容范围，不照搬其API。

## 验收

按commands 0/1/1k/10k、text/image/brush mix、stable/1% layout/style/content change、diagnostics off/on记录paint projection、serde/hash/debug-label bytes、text/report clone bytes、Vec alloc/grow、buffer create/upload与CPU p50/p95/p99。当前门槛为`to_paint_elements`≤1/command且empty LoadStore pass=0；最终stable generation上述projection/serde/hash/deep-clone/plan rebuild/GPU create/upload全部为0，changed近dirty commands/ranges，sealed report deep clone=0。Cargo、编辑器text/image/clip/decorations像素、F2与DX12 RenderDoc通过前保留在`pending.md`，不进入`review.md`。
