---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/**/*.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives/image.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives/image/**/*.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/gpu/present.rs
  - zircon_runtime/src/rhi/ui_surface.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/08
  - docs/plans/zircon_runtime/render/13
reference_sources:
  - dev/bevy/crates/bevy_ui_render/src/lib.rs
  - dev/slint/internal/renderers/femtovg/images.rs
  - dev/godot/servers/rendering/renderer_canvas_render.h
tests:
  - owned_runtime_draw_list_moves_text_and_image_allocations
  - atlas_recording_keeps_one_copy_of_atlas_pixels
  - existing command stream extraction/replay/atlas/GPU presenter tests
  - current-source Windows Cargo and GPU/Softbuffer allocation trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor chrome command stream逐文件性能静态审查（2026-07-17）

## 范围与覆盖

`zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream`当前共 **40** 个Rust文件、**2,304** 行，已逐文件阅读 **40/40**。直接图像录制/软件光栅子模块`paint_primitives/image.rs` + `paint_primitives/image/**`共 **9** 个Rust文件、**526** 行，也已阅读 **9/9**。两者仍缺当前源Cargo、GPU/Softbuffer产品trace和分配计数，因此继续留在`pending.md`。

## 已有正确边界

命令流保留full rebuild与damage patch、clip、稳定z-index、atlas UV、image upload bytes和CPU像素回放；GPU presenter在surface cache bootstrap后允许damage stream，Softbuffer保留region backbuffer。图像软件路径先尝试不缩放不透明row copy，失败后才逐像素缩放/混合。现有测试覆盖legacy painter像素一致性、damage像素隔离、atlas采样、文本字体投影和GPU presenter错误传播。

## 热点与直接修复

- PERF-MVP-150：普通GPU路径此前从owned `ChromeCommandStream`借用构造第二份owned `UiSurfaceDrawList`，逐command clone文本、resource key和整块RGBA；stream随后立即丢弃。本轮增加owned转换，正常`present`移动这些分配，显式borrowed `present_stream`测试接口不变。指针复用测试冻结text/resource/RGBA零复制终跳。
- Atlas recording此前同时写入普通`rgba`和含同一像素的`atlas`，extraction看到atlas后必然丢弃前者。本轮删除这份确定无用的整atlas clone并加回归测试。`stats()`仅做唯一key去重，`BTreeSet`改为`HashSet`。
- PERF-MVP-151：damage只设置recording clip，仍调用`draw_workbench_presentation_commands`遍历完整presentation；测试所谓“不重建static layer”只是把所有quad标为Dynamic，未证明未访问未变节点。必须由EditorUI08以dirty generation + section/spatial index生成patch，未变节点visited=0。
- PERF-MVP-152：CPU replay原先每次分配`Vec<(index, &command)>`并stable sort。recording的正常z-index本来单调，本轮增加单调fast path；仅外部/测试乱序stream才分配排序。
- PERF-MVP-153：Softbuffer scaled image对每个目标像素执行浮点除法、floor和clamp；identity opaque路径先完整扫描alpha再复制。后续应按resource generation缓存alpha/尺寸元数据，并以定点步进或坐标LUT处理scaled span，产品trace证明收益后再决定缩放结果缓存。

## 参考引擎约束

Bevy `ExtractedUiNode`只保存`AssetId<Image>`，并由`ImageNodeBindGroups: HashMap<AssetId<Image>, BindGroup>`维护GPU资源；Slint用`TextureCacheKey -> Rc<Texture>`避免每个item携带像素；Godot canvas `CommandRect`/`CommandNinePatch`/`CommandPolygon`只携带`RID texture`。因此剩余PERF-MVP-150不能把`Arc<Vec<u8>>`散落到每条draw command作为最终形状：图像字节应驻留resource registry，command只持稳定handle、UV与generation，上传按changed resource最多一次。

## 动态验收

1k atlas/image/text command分别记录recording、stream、RHI draw-list阶段的allocated/copied bytes；同一resource generation的RGBA payload副本数=1、GPU upload≤1，steady patch upload=0。1/1k/10k command记录patch visited nodes、replay sort fallback、Softbuffer scaled pixels和main-thread p95。GPU、Softbuffer、atlas UV、clip、z-order、damage pixels与现有像素结果必须一致。
