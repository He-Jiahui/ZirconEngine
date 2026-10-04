---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_layout/21-gpu-submission-and-draw-pipeline.md
  - docs/plans/zircon_runtime/render/14-2d-stack.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
  - docs/plans/zircon_runtime/text/09-threading-caching-and-performance.md
tests:
  - ui render subdirectory 17 of 17 Rust files reviewed, 3069 current lines
  - production 10 of 10 and tests 7 of 7 reviewed
  - RED/GREEN source guards cover shared paint projection, one rich parse, indexed vertical ranges and single blocker scan
  - rustfmt check and git diff check passed for changed files
  - focused Cargo reservation c5e7a6ccdba740b59c223c2a8307de63 source-bound and queued, not FIFO head
  - editor pixels, F2 and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics UI render子目录逐文件性能静态审查（2026-07-18）

## 范围与结论

已完整阅读`scene_renderer/ui/render/**`当前17/17个Rust文件、3069行，其中production 10/10、tests 7/7。范围覆盖background/color/geometry/rich text/text advances/decorations/effects/paint/projection及全部单元、parity、inline/table测试。

直接止损四处。第一，`text_paint::command_text_paint`改为借用root已生成的`&[UiPaintElement]`，跨文件真实调用链从每text command两次`to_paint_elements`降为一次。第二，rich inline定位直接消费本command已解析的`StyledRun::inline`与resolved line，不再为每个inline重新`parse_rich_text`。第三，纵排advance先建一次source-grapheme range表，以`partition_point`定位glyph end和overlap起点；common resolve路径让build/apply共享表，复杂度从graphemes×glyphs双向全扫降为O(G + N log G + overlaps)。第四，背景推断先线性求目标frame的latest blocker，再逆序查覆盖candidate，从candidate×blocker降为C+B。

剩余瓶颈归PERF-MVP-398：每frame七组plan Vec和CPU rect/border六顶点展开；`run_for_range`逐paint run线性扫parsed runs，inline定位再扫line runs并重复统计prefix grapheme/advance，长rich paragraph仍可能近O(R²)；color字符串、text line/glyph advances/font/family/decorations和prepare report仍重复解析或deep clone。最终必须由EditorLayout21发布唯一generation-owned ordered prepared plan、dense rich-run ranges和共享text/image/report handles，Render14只消费persistent geometry/instance dirty ranges。

本地对照继续采用已核实的Bevy `dev/bevy/crates/bevy_ui_render/src/lib.rs::ExtractedUiNodes/UiBatch/prepare_uinodes`和UE Slate `ElementBatcher.h::MergeRenderBatches/FSlateElementBatcher`、`SlateRenderBatch.h::IsBatchableWith`：目标是extract/prepare单一authority、兼容键与有序range复用，不照搬API。

## 验收

按commands/runs/inlines 0/1/1k/10k、plain/rich/RTL/vertical、background candidates/blockers 0/1/1k、stable/1% layout/style/content change记录paint/rich parse、run/prefix/grapheme/glyph/background visits、serde/hash/debug bytes、text/report clone bytes、Vec alloc/grow、CPU vertex bytes、GPU buffer create/upload与p50/p95/p99。当前门槛：paint≤1/command、rich parse≤1/rich command、vertical无glyph×grapheme全扫、background query O(C+B)；最终stable generation全部projection/index/clone/plan rebuild/GPU create/upload为0。Focused Cargo、编辑器rich/RTL/vertical/background像素、F2与DX12 RenderDoc通过前保留在`pending.md`，不进入`review.md`。
