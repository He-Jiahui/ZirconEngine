---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/history
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_history/prepare_history_textures.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/history/copy_history_textures.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_history_graph_resources.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
  - docs/plans/zircon_runtime/render/04-visibility-culling.md
  - docs/plans/zircon_runtime/render/06-temporal-pipeline.md
  - docs/plans/zircon_runtime/render/07-postprocess-color-pipeline.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
  - docs/plans/zircon_runtime/render/18-advanced-lighting-features.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Public/RenderGraphBuilder.h
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/PostProcess/TemporalAA.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Lumen/LumenReflections.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/DeferredShadingRenderer.cpp
tests:
  - current history slice 7 of 7 Rust files reviewed, 472 lines, 1 inline test
  - direct prepare copy and binding chain 3 of 3 files reviewed, 989 lines, 12 tests
  - scoped rustfmt 7 of 7 clean
  - current-source Cargo, copy counters, F2 pixels, GPU timestamps and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_blocked
---

# Graphics history current-source结构审查（2026-08-15）

## 当前范围与有效进展

`zircon_runtime/src/graphics/scene/scene_renderer/history/**`当前物理清单7/7个Rust文件：472行、433个非空行、1条内联测试，fingerprint为`62E8E51EC7FF3CDBF453E7D36E12B75529E76D0D524A87BF3A0BA91A857EBC4D`。另复读直接prepare/copy/bind链3/3文件（989行、12 tests）。2个history本体文件已有其他会话identity改动，本轮未改这些生产文件；7/7逐文件通过`rustfmt 1.8.0 --edition 2021 --check --config skip_children=true`。

旧报告的CPU整图初始化问题仍保持已修：六张2D history用2个GPU clear pass、一次submit初始化，没有full-texture CPU Vec或`write_texture`。TAA current/previous和exposure read/write采用稳定双slot角色翻转，不做帧末复制；新增TAA/HZB sampled identity也为稳定bind cache提供正确基础。这些行为应保留。

## P0：graph output复制到第二套persistent history

GI lighting、GI temporal metadata、AO、SSR、HZB和volumetric仍采用另一种所有权：本帧先写graph transient/offscreen target，所有graph stage结束后再由serial encoder显式`copy_texture_to_texture`到`SceneFrameHistoryTextures`。源码有7个copy call sites；HZB call site在完整mip循环内，4K发12个texture copy commands。TAA/exposure已经证明直接写history owner并flip角色可行，其他history却继续保留生产结果与persistent副本两套资源。

按current descriptor和full-target extent计算，只计destination payload、不含源读取、driver对齐、压缩与volumetric，GI+metadata为16 B/pixel、AO 4 B/pixel、SSR 8 B/pixel，再加Rgba16Float HZB完整mip链：

| viewport | encoded history copy / frame | at 60 Hz |
|---|---:|---:|
| 1280x720 | 29.94 MiB | 1.75 GiB/s |
| 1920x1080 | 66.04 MiB | 3.87 GiB/s |
| 2560x1440 | 119.77 MiB | 7.02 GiB/s |
| 3840x2160 | 264.15 MiB | 15.48 GiB/s |

这是源码描述符/extent的编码payload模型，不是显存总线实测；实际物理读写、缓存与压缩必须由GPU capture确认。split-view会按GI区域裁剪，但AO、SSR与HZB仍按各自target/graph extent复制。当前`RenderHistoryCopyReport`只记录7个requested feature groups和成功布尔值，没有copy command count、mip count或bytes，因此产品trace无法验证上述规模，也无法区分直接flip与真实copy。

## Unreal Engine本地源码依据

- `RenderGraphBuilder.h:341-346`的`QueueTextureExtraction`对graph-created texture延长GPU resource lifetime，在graph执行结束时填充pooled owner；这是一种资源所有权转移，不要求复制到预先存在的第二张纹理。
- `TemporalAA.cpp:744-783,1159-1167`直接把TAA pass生成的`NewHistoryTexture`作为scene output，并只提取实际使用的history targets。
- `LumenReflections.cpp:1853-1862,2385-2394`直接提取specular/second-moment与frame-count temporal outputs到reflection state，关闭temporal时释放owner。
- `DeferredShadingRenderer.cpp:610`把本帧`FurthestHZBTexture`直接提取到`PrevFrameViewInfo.HZB`，没有逐mip复制到独立历史纹理。

Zircon应建立等价的multi-frame graph extraction/promotion合同：本帧produced physical texture在graph完成后成为该feature的previous owner，下帧注册为external/history input；需要同帧继续消费时用同一physical handle。不能把所有输出强制变成常驻external texture，这会破坏transient aliasing；应只promote实际请求且成功生成的history artifact，并由feature generation、size/format/quality和camera handle控制失效。

## 与PERF-MVP-395的边界

PERF-MVP-395负责“任一feature触发完整history包”及稳定frame handle clone，目标是feature-owned lazy slot与affected-only rebuild。新增PERF-MVP-625负责“本帧结果如何成为下一帧history”，目标是graph extraction/role rotation与帧末显式copy归零。两者必须共享同一个`HistoryResourceMask`和per-slot generation，但验收指标不同，不能用拆包掩盖每帧copy，也不能用extraction保留无关slot。

## 实施与验收

1. Render01定义`ExtractedHistoryArtifact`：physical texture owner、descriptor、generation、valid region/mips与final access；compiled graph只为请求的history output创建promotion ticket。
2. Render04/06/07/18让HZB、TAA、AO/SSR/GI和volumetric pass直接产出可提取artifact；成功提交后原子替换previous owner，cut/失败/cancel保持旧owner或显式invalidate，不出现半更新。
3. Render01/17让execution report区分flip/extract/copy fallback，记录commands、mips、bytes、promoted/reused/released VRAM和alias loss；仅不支持promotion的后端可保留可观测fallback copy。
4. 先在current-source F2产品构建用GPU timestamp与RenderDoc核对copy events/bytes，再删除legacy unconditional path；不得把静态264.15 MiB写成GPU耗时收益。

矩阵：history feature each/all，views 1/2/8，720p/1080p/1440p/4K，full/split/dynamic-resolution，stable/resize/cut/device-loss/pass failure，transient alias pressure low/high。硬门：支持promotion的后端stable history copy commands/bytes=0、每feature成功输出promotion<=1/frame、feature-off owner=0、changed-only replace；fallback有精确bytes/mips且不静默启用；previous/current角色、有效区域、HZB全mip、GI metadata、AO/SSR、volumetric quality与TAA/exposure语义通过Cargo/F2像素/timestamp/RenderDoc。当前无current-source产品二进制且focused validator受协调器超时阻塞，本模块留在`pending.md`，不进入`review.md`。
