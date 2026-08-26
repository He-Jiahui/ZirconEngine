---
title: Runtime UI Renderer、Display List、Paint Order、Clip、Transform、Opacity、Atlas、Text、Glyph、Batch、WGPU Submit 与 Product Integration 当前源码工程化差距
category: zircon_runtime
report_id: Runtime79
review_date: 2026-08-21
baseline_head: bea1acf91b909525ab1759e2c800858b0eda6528
baseline_epoch: 335
related_code:
  - zircon_runtime/src/ui/surface/render
  - zircon_runtime/src/ui/icon_atlas
  - zircon_runtime/src/graphics/scene/scene_renderer/ui
  - zircon_runtime_interface/src/ui/surface/render
  - zircon_runtime_interface/src/ui/layout/geometry.rs
  - zircon_runtime/crates/zr_rhi/src/ui_surface.rs
  - zircon_runtime/crates/zr_rhi/src/ui_surface
  - zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream
  - zircon_editor/src/ui/retained_host/host_contract/presenter/gpu.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/gpu
tests:
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_renderer/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/tests
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/tests
  - zircon_runtime/src/ui/tests/icon_atlas.rs
  - zircon_runtime_interface/src/ui/surface/render/batch/tests.rs
  - zircon_runtime/crates/zr_rhi/src/ui_surface/tests.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/tests
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/tests
  - zircon_editor/src/ui/retained_host/host_contract/presenter/gpu/tests.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
  - docs/plans/optimize/zircon_runtime/11b-runtime-text-font-shaping-layout-editing-ime-review.md
  - docs/plans/optimize/zircon_runtime/11c-gpu-ui-renderer-atlas-sdf-batch-clip-submit-review.md
  - docs/plans/optimize/zircon_runtime/73-runtime-ui-style-theme-token-cascade-selector-pseudo-state-invalidation-transition-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/74-runtime-ui-template-component-binding-expression-model-event-command-hot-reload-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/75-runtime-ui-component-catalog-widget-behavior-state-reducer-interaction-semantics-accessibility-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/76-runtime-ui-layout-box-model-measure-arrange-flex-grid-overflow-scroll-virtualization-dpi-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/77-runtime-ui-input-dispatch-routing-focus-navigation-pointer-capture-gesture-drag-drop-ime-window-lifecycle-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/78-runtime-ui-accessibility-semantic-tree-name-description-relation-state-action-live-region-platform-adapter-product-integration-review.md
  - docs/plans/zircon_runtime/runtime/09/failure-2026-07-17-ui-render-command-transient-extraction.md
  - docs/plans/zircon_runtime/runtime/09/failure-2026-07-19-dynamic-ui-extract-generation.md
  - docs/plans/zircon_runtime/text/04/failure-2026-07-18-glyph-atlas-draw-vertex-duplication.md
  - docs/plans/zircon_runtime/text/04/failure-2026-07-18-bitmap-atlas-full-page-staging-and-dirty-union.md
  - docs/plans/zircon_runtime/text/09/failure-2026-08-14-rhi-wgpu-ui-surface-present-stats-non-exhaustive-construction.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Rendering/ElementBatcher.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Rendering/SlateRenderBatch.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Rendering/DrawElementTypes.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Rendering/SlateRenderBatch.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Layout/Clipping.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateRHIRenderer/Private/SlateRHIRenderingPolicy.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SlateRHIRenderer/Private/SlateRHITextureAtlas.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Rendering/SlateVectorGraphicsCache.cpp
  - dev/bevy/crates/bevy_ui_render/src/lib.rs
  - dev/bevy/crates/bevy_ui_render/src/render_pass.rs
  - dev/bevy/crates/bevy_ui_render/src/pipeline.rs
  - dev/bevy/crates/bevy_ui_render/src/text.rs
  - dev/bevy/crates/bevy_ui_render/src/gradient.rs
  - dev/bevy/crates/bevy_ui_render/src/box_shadow.rs
  - dev/bevy/crates/bevy_ui_render/src/ui_material.rs
  - dev/bevy/crates/bevy_ui_render/src/ui_texture_slice_pipeline.rs
  - dev/Fyrox/fyrox-ui/src/draw.rs
  - dev/Fyrox/fyrox-impl/src/renderer/ui_renderer.rs
  - dev/godot/servers/rendering/renderer_canvas_cull.cpp
  - dev/godot/servers/rendering/renderer_rd/renderer_canvas_render_rd.cpp
  - dev/godot/scene/main/canvas_item.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Textures/Texture2DAtlas.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Textures/PowerOfTwoTextureAtlas.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 79 · Runtime UI Renderer、Display List、Paint Order、Clip、Transform、Opacity、Atlas、Text、Glyph、Batch、WGPU Submit 与 Product Integration 当前源码工程化差距

## 1. 结论

Zircon 的 UI GPU 路径不是空壳。游戏视口已有真实 WGPU shape/image/text pipeline、Glyphon、native bitmap/color glyph atlas、SDF/MSDF/MTSDF、dirty page upload、persistent instance buffer 与 product framebuffer fixture；Editor native window 已有重叠依赖分析、安全的非重叠重排、solid instance/image vertex arena、generation cache、256 entries / 64 MiB image admission、GPU timestamp、retained texture 和 damage scissor。这些底座应保留，尤其当前 bitmap glyph 已经删除双层六顶点 CPU 物化，native image cache 也会在 admission 时把 straight RGBA8 **只预乘一次**，不能继续沿用旧报告中已经失真的描述。

但产品仍然没有一条统一、严格有序、generation-owned 的 UI presentation spine。游戏路径把已排序 command fanout 为 shape、image、三类 text 和 post-text decoration，再按固定资源类别提交；Editor 路径消费另一套更窄的 `UiSurfaceDrawList`。`zircon_runtime_interface` 已声明 `UiBatchPlan`、八类 brush payload、stencil clip、draw effect 与 geometry transform，但产品 scene renderer 不消费 batch plan、render transform 或 draw effect；native RHI 又只有 axis-aligned rect、单一圆角、text/image/clip，无法表达完整 transform、nested mask、group opacity、gradient/vector/material、filter 和 offscreen layer。

本轮 current-source 复核确认 Runtime11C 的三项既有 P0 **全部仍开放**：game painter order 被资源分桶破坏、runtime `Icon` 稳定退化为实心矩形、Editor damage patch 在 retained pixels 上以 `Load + premultiplied blend` 重放而没有 backdrop restore/replace。它们继续由 Runtime11C 唯一计数，本报告不重复新增 P0。新增证据包括：普通 Editor full/damage frame 明确保持 `generation=None`，因而 compiled batch/vertex/text cache日常不命中；runtime icon grid 在 4096 边长 clamp 后仍继续生成越界 slot/UV；game/interface transform 只有 translation + scale 且没有 subtree compositing；native surface只接受 Win32与 non-sRGB BGRA/RGBA UNORM；4K retained path每次更新仍全图 copy约31.64 MiB。

本报告登记 **0 项新增 P0、48 项 Runtime79 独有 P1、12 项 P2 与 48 项资格门**。目标不是给两套 renderer 分别补 feature，而是建立唯一的 `UiPresentationArtifact -> UiPresentationCompiler -> OrderedUiDrawOp -> DeviceUiResourceService -> UiSurfacePresenter` 链。正确性门必须先于 draw-call 优化；功能、视觉、fault、规模和同负载基准全部通过前，不得以 test attribute 数、DTO 数、局部 framebuffer 或单次 GPU timing 宣称已经达到或超过 Unreal Slate。

## 2. 审查边界与物理冻结

### 2.1 Owner 边界

| 领域 | Canonical owner | Runtime79 责任 | 本轮不重复登记 |
|---|---|---|---|
| UI tree、paint order source | Runtime11A / Runtime75 | 接收稳定 painter token 与 component render facet | tree/node order、component catalog 与 live mutation 总 owner |
| Text shaping/layout/font | Runtime11B | 消费 resolved glyph/layout artifact，定义 GPU handoff | Unicode shaping、editing、IME、font source/cook 总 owner |
| GPU UI renderer 三项视觉阻断 | Runtime11C | current-source 复核、补充当前证据与实施门 | painter order、runtime icon、damage alpha 三项既有 P0 |
| Style/layout/input/a11y | Runtime73-78 | 消费同代 style/geometry/focus/semantic generation | cascade、layout、input transaction、platform a11y |
| RHI/device/residency | Runtime09A / 09D | 定义 UI presentation 对 device/resource generation 的需求 | 通用 device loss、fence retirement、asset residency |
| Editor paint/host | Editor01 / Editor23 | Editor producer 编译到统一 presentation artifact | Editor document/authoring/window UX |

Runtime79 只登记 renderer/presentation 专属的 authority、语义、资源、提交、颜色、缓存与资格断点。相同根因已经是 Runtime11C P0 或上表父 owner 时，本报告只记录依赖，不再次计数。

### 2.2 Zircon 物理冻结

指纹算法：相对路径排序；每个文件计算 SHA-256；以 `path<TAB>lowercase-hash` 用 LF 连接且末尾不附加 LF；再对清单 UTF-8 bytes 做 SHA-256。production classifier 排除路径段 `/tests/`、叶文件 `tests.rs` 与 `product_framebuffer` test owner，但保留 WGSL、inline tests 与 `atlas_tests.rs`。统计绑定共享 working copy，而不是只绑定 baseline HEAD。

| 范围 | 全部文件 / 行 / bytes / test attributes | production 文件 / 行 / bytes | 证据 |
|---|---:|---:|---|
| Scene UI renderer | 108 / 25,849 / 925,995 / 301 | 65 / 13,253 / 473,042 | command plan、shape/image、三类 text、bitmap/SDF atlas、upload、record、WGSL |
| Runtime paint + icon | 49 / 17,992 / 600,925 / 36 | 42 / 16,846 / 562,218 | widget paint producer、cache/damage、icon parse/plan |
| Interface render contract | 35 / 5,857 / 187,138 / 26 | 34 / 5,697 / 182,484 | paint/brush/batch/cache/parity/debug DTO |
| `zr_rhi` UI surface | 4 / 1,795 / 58,214 / 19 | 3 / 1,255 / 40,859 | native command/draw-list/image resource/present stats |
| `zr_rhi_wgpu` UI surface | 20 / 8,936 / 316,768 / 114 | 15 / 6,031 / 221,853 | batch/geometry/pipeline/text/image cache/retained present/surface |
| Editor stream + GPU presenter | 48 / 4,893 / 159,829 / 48 | 41 / 3,206 / 107,902 | ordered stream、icon atlas、RHI conversion、full/damage/resize present |
| 去重合计 | **264 / 65,322 / 2,248,869 / 544** | **200 / 46,288 / 1,588,358** | 全集 fingerprint `f9ab0319758e401becc1b742e78c6b247798c723fc94b475325c8a204c0e16d3`；production fingerprint `b1480967a8c77077e97643099846a1719c9d700be31eb5af5bec5ef7b35bb071` |

冻结时，`scene_renderer/ui/render.rs`、`text/prepare_report/profile.rs` 与 Editor command stream 的 `stream/image_resources.rs`、`stream/model.rs` 存在其他 Session 或用户修改；测试 owner 另有两处 dirty。本文未修改这些源码，故 `source_recheck_required: true`，实施前必须重取指纹与相关结论。

### 2.3 参考物理冻结

参考冻结 **23 个文件、24,534 行、971,132 bytes**，排序清单 fingerprint 为 `90cb52a6a7874859cbdc60d415c221b0b1527fb06ccf5fad50f04f2222add2ce`。

| 参考 | 文件 / 行 / bytes | 直接可吸收的不变量 | 局限与不照搬内容 |
|---|---:|---|---|
| Unreal Slate | 8 / 7,765 / 319,448 | stable layer order、严格 `IsBatchableWith`、full transform、clip handle、scissor/stencil、rich draw types、texture/vector cache 与 RHI submit | 工程复杂度上限；不复制宏、指针模型与具体 RHI API |
| Bevy UI Render | 8 / 5,158 / 191,492 | `TransparentUi` phase 中统一 z order，basic/gradient/shadow/material/slice/text进入同一 phase；typed pipeline specialization | 本地 clip 对 rotation/scale 仍有已知限制，不作为复杂裁剪上限 |
| Fyrox | 2 / 1,721 / 62,136 | 单一 ordered command buffer、transform/opacity stack、complex clip、brush/material/font atlas | 逐 command dynamic upload/draw 是架构下限，不是性能目标 |
| Godot Canvas | 3 / 8,330 / 317,747 | ordered item/command、transform/modulate/material/clip owner、canvas group、instanced rect/nine-patch、HDR format policy | 不复制 Object/Variant 与 renderer storage 形态 |
| Unity Graphics | 2 / 1,560 / 80,309 | atlas allocator、hash/update、GPU-valid mip、release/reset、hole/relayout、size estimate | 本地 corpus 不是 UI Toolkit renderer，只作 atlas 生命周期参考 |

### 2.4 证据等级与限制

本轮逐文件读取上述 264 个聚焦文件，并沿两条产品链追踪 consumer：`UiRenderExtract -> ScreenSpaceUiRenderer -> WGPU attachment` 与 `ChromeCommandStream -> UiSurfaceDrawList -> WgpuUiSurfaceRenderer -> retained/native surface`。544 个 test attribute 是结构覆盖量，不代表 544 个产品资格；部分测试是 source guard、CPU unit 或在没有 WGPU adapter 时直接返回。

本轮只做静态 review，没有运行 Cargo、Editor、真实窗口、RenderDoc、HDR display、device-loss fault、soak 或 benchmark，也没有生成新的像素 artifact。静态证据能证明字段是否被消费、提交顺序、cache key条件、surface format、load/blend与资源 owner；真实像素差、GPU cost和驱动行为必须由资格门验证。

## 3. 当前产品链与可保留底座

### 3.1 三层合同的实际差异

| 维度 | Runtime/game scene UI | Editor native UI | Interface 理想表面 |
|---|---|---|---|
| 输入 | `UiRenderCommand` / legacy paint expansion | `UiSurfaceCommand` / owned stream | `UiPaintElement` / `UiBatchPlan` |
| order | command 输入有序，提交按资源类别重新分桶 | dependency depth 只重排非重叠项，窄合同下基础正确 | `(z_index, paint_order, source index)` stable order |
| geometry | axis-aligned rect/border/full-UV image | axis-aligned rect、单 corner radius、atlas UV | local/layout/render geometry 与 pixel snapping |
| clip | 单矩形交集/scissor | command rect + damage scissor | `Scissor` / `Stencil` |
| transform/opacity | scalar command opacity，忽略 `render_transform` | 没有 transform/group opacity字段 | translation+scale DTO、draw effect |
| text | Glyphon + bitmap atlas + SDF 三 route | 独立 Glyphon、plain `Wrap::None` | resolved typography/layout/geometry DTO |
| icon/image | runtime Icon退化矩形；image由generic streamer解析 | process-global icon atlas + bounded image cache | Image/Vector/Material brush |
| cache | 每帧 rebuild/hash；atlas局部 retained | generation cache存在，普通frame无generation | render/cache plan 主要停留在 CPU 表面 |
| present | scene final attachment | retained texture + full copy + native surface | 无统一 presentation/device contract |

### 3.2 应保留的真实基础

- `UiBatchPlan::from_paint_elements` 已能 stable sort `(z_index, paint_order, source index)`，只合并相邻且 key 相同元素；算法本身可迁移到 canonical compiler。
- `zr_rhi_wgpu/ui_surface/batching.rs` 用 overlap interval dependency depth 约束重排，能在窄 axis-aligned contract 下安全合并互不重叠 item；这比无条件按 pipeline sort 更可靠。
- bitmap/color glyph 已形成 page generation、dirty upload、shadow replay、storage split、instance buffer与 typed prepare report；SDF/MSDF/MTSDF 也有 slot retention、relocation/eviction和 fallback 统计。
- WGPU image cache在 admission 时执行一次 straight RGBA8 -> premultiplied RGBA8 转换，shader不会二次预乘；并已有 transparent-edge offscreen GPU test。Runtime11C 中“过滤图像仍以 straight alpha 进入 premultiplied blend”的旧表述应正式退役。
- Editor image cache和shared registry有明确 256 entries / 64 MiB预算、generation identity、LRU admission、CPU/GPU/shared resident byte统计与 external image 支持。
- native presenter有 retryable surface acquisition、submit后才commit retained baseline、GPU timestamp/readback、RenderDoc marker和多类 batch/pass/copy stats；这些应提升为统一 schema，而不是被删除。

## 4. 继承 P0 的 current-source 状态

**本轮无新增 P0。** Runtime11C 的三项 P0 仍由其唯一计数，Runtime79只更新证据与资格依赖。

| 既有阻断 | 当前源码证据 | 状态与必须关闭的结果 |
|---|---|---|
| Runtime11C P0-1：game painter order | `render.rs` 仍持有 `draws/post_text_draws/auto_texts/native_texts/sdf_texts/images`；`record.rs` 仍固定 shape -> image -> text -> decoration | **open**；统一 ordered draw-op，跨 solid/image/三 text route/decor 的重叠矩阵逐像素正确 |
| Runtime11C P0-2：runtime icon 产品占位 | production producer广泛发出 `UiVisualAssetRef::Icon`；scene path仍只识别 `Image`，其余 image/icon fallback为0.68边长实心矩形；`UiIconAtlasBuilder`仍无production consumer | **open**；cooked icon/vector artifact、page/upload/generation/renderer consumer闭合 |
| Runtime11C P0-3：damage alpha/删除错误 | `DamagePatch` 仍对 retained view 使用 `TargetLoad::Load`，只重画damage-visible ops，再全图copy；没有clear/backdrop restore/isolation/replace | **open**；full与damage对半透明、删除、移动、AA边缘和重复patch逐像素一致 |

## 5. P1 差距

### 5.1 Presentation authority、display list 与语义合同

| ID | 差距 | 工程化要求 |
|---|---|---|
| RUIR-P1-001 | game `ScreenSpaceUiRenderer` 与 native `WgpuUiSurfaceRenderer` 各自拥有 command、text、image、clip、cache、color和stats真值 | 建立唯一 versioned `UiPresentationArtifact` 与 compiler；surface adapter只负责target/acquire/present |
| RUIR-P1-002 | `UiBatchPlan` 只被 interface test/debug/parity/cache消费，scene与Editor产品不消费其ordered indices | 把稳定 painter token、batch key与split reason变成产品 artifact，删除未消费的平行表面 |
| RUIR-P1-003 | `UiSurfaceCommandKind` 只含 Quad/Border/Text/Image/Styled/Clip，无法承载完整Runtime paint语义 | 设计loss-aware canonical op schema；backend不支持时typed reject/fallback，禁止静默降级 |
| RUIR-P1-004 | Solid/Image/Box/Border/Rounded/Gradient/Vector/Material brush只有DTO宽度，scene产品只实现窄rect/image/text | 每种声明能力必须有compiler、backend capability、fallback与像素资格，未交付类型不得报告Supported |
| RUIR-P1-005 | `UiRenderTransform` 只有translation + scale，没有rotation/skew/pivot/affine matrix，产品又不消费它 | 使用稳定2D affine transform与inverse/hit/clip contract；相同geometry generation贯穿paint和a11y |
| RUIR-P1-006 | opacity只在command级乘颜色，没有parent/subtree/group compositing owner | 引入opacity stack和isolated group；重叠子元素的group opacity必须先离屏再合成 |
| RUIR-P1-007 | `UiClipMode::Stencil` 没有产品consumer，当前只有单axis-aligned scissor，无nested rounded/path mask | 建立clip stack/handle、scissor快路、analytic round/mask、stencil overflow与cache生命周期 |
| RUIR-P1-008 | `UiDrawEffect` 不进入产品，shadow/blur/backdrop/filter/blend/material isolation没有effect graph | 建立typed effect/layer graph、temporary target预算、dependency与composite order |

### 5.2 Runtime/game scene 热路径与 image resource

| ID | 差距 | 工程化要求 |
|---|---|---|
| RUIR-P1-009 | 每帧新建shape/image/三text/decor等多个Vec，稳定UI仍全量遍历、clone和fanout | producer发布immutable generation segments；dirty只替换受影响range |
| RUIR-P1-010 | rect生成6 CPU vertex，border最多拆4 rect；每command/scissor单独draw | static quad + compact instance arena + adjacent ordered batching；优化不得跨重叠非法重排 |
| RUIR-P1-011 | 每command调用 `to_paint_elements(0)` 并返回owned Vec，generation-owned typed range仍未闭合 | 完成Runtime09 open handoff，extract直接发布borrowed/Arc typed ranges与build counters |
| RUIR-P1-012 | `cache_generation()`仍以serde JSON遍历完整command计算FNV，producer mutation generation未成为权威 | generation由source mutation、style/layout/text/resource依赖组合；render prepare不再序列化对象 |
| RUIR-P1-013 | shape/image先重建完整CPU payload，再BLAKE3全部bytes决定是否upload | stable generation直接命中persistent range；只hash/upload dirty segment并报告bytes |
| RUIR-P1-014 | scene image固定full UV、逐image draw/binding，缺atlas page、nine-slice、tile和sampler/mip合同 | image op携带resource/subresource/sampler/UV/tint/slice/generation，按相邻兼容项合批 |
| RUIR-P1-015 | generic streamer fallback把ready/pending/missing/decode-failed/evicted/stale压成相似输出 | 贯穿typed readiness与last-good policy；stats按原因、资源和generation记录 |
| RUIR-P1-016 | game UI没有统一image/bind-group/vertex/atlas预算，residency依赖generic streamer隐式行为 | 接Runtime09D资源owner，定义UI priority、pin、evict、device generation和pressure downgrade |

### 5.3 Text、glyph 与 atlas pipeline

| ID | 差距 | 工程化要求 |
|---|---|---|
| RUIR-P1-017 | Glyphon/native bitmap/SDF route各自准备和提交；fallback span不能天然插回原painter位置 | resolved glyph run携带painter token与material route，fallback只替换原span，不重排其他内容 |
| RUIR-P1-018 | Editor `WgpuUiTextRenderer` 自建Glyphon `FontSystem/TextAtlas/SwashCache`，不消费Runtime font owner | 统一font bytes/face/fallback/layout/glyph generation；Editor与game相同输入选择相同face与metrics |
| RUIR-P1-019 | native RHI text只有plain String/family/weight/size/line-height/style，backend固定 `Wrap::None` | 输入应是resolved runs/layout artifact，保留direction/language/alignment/selection/composition/effect |
| RUIR-P1-020 | 每个text batch建立独立 `TextRenderer`，ordered stream中每个text op切新render pass | 共用glyph material/atlas与连续run；只有attachment/effect边界才断pass |
| RUIR-P1-021 | bitmap/SDF allocator、page shadow、slot handle各自局部成熟，但没有共同device/page lease | 建立generational atlas page/slot contract、dirty rect、format/padding、lease和retirement |
| RUIR-P1-022 | SDF quality/page/slot目标主要是固定默认，未按DPI、locale、device tier或VRAM pressure协商 | 建立quality profile、page budget、downgrade/fallback receipt与warm/cold metrics |
| RUIR-P1-023 | bitmap atlas dirty/shadow修复已有前向实现，但failure仍open且缺managed WGPU/product pixel终态 | 完成source-bound Cargo、mixed storage、dirty bytes和真实产品像素回执后再转fixed |
| RUIR-P1-024 | glyph atlas已改为68B instance + shader six-corner，但failure仍等待managed scale/product gate | 保存1/100/1K/10K occurrence/upload/draw/p50/p95与真实framebuffer证据，关闭ledger/source差异 |

### 5.4 Icon、image cache 与资源生命周期

| ID | 差距 | 工程化要求 |
|---|---|---|
| RUIR-P1-025 | runtime icon plan把atlas宽高clamp到4096，却继续按未clamp cell/grid生成slot，rect和UV可越界 | 在admission阶段验证容量，分页/拒绝oversized，所有UV必须在页内且整数乘法无溢出 |
| RUIR-P1-026 | SVG parser只做字符串attribute扫描并主要接受`<path d>`，无group/transform/defs/use/clip/fill-rule等 | 使用成熟、bounded SVG parser+tessellation/raster或离线cook，错误带source span和预算原因 |
| RUIR-P1-027 | Editor icon atlas是 `OnceLock<Mutex<...>>` process-global owner，跨project/window/device共享 | 拆为content artifact、device atlas service与session lease；支持teardown/device reset |
| RUIR-P1-028 | Editor page首次发布后sealed，后续发现icon即使有空位也倾向新页，按discovery wave碎片化 | 支持appendable page generation + dirty subrect，或在显式prewarm/cook阶段一次冻结 |
| RUIR-P1-029 | runtime icon、Editor icon、bitmap/SDF glyph、scene image、native image各有key/page/budget/upload合同 | 收敛外部page/resource协议，同时允许不同format和allocator实现 |
| RUIR-P1-030 | icon theme、semantic variant、DPI bucket、content hash与page generation未形成统一identity | 定义qualified icon handle和variant resolver；theme/DPI改变只失效相关slot |
| RUIR-P1-031 | native local image cache同时保留premultiplied CPU bytes与GPU resource，设备恢复有价值但内存可双占 | 分别预算CPU shadow/GPU/local/shared bytes，按recoverability与source availability决定保留 |
| RUIR-P1-032 | local/shared/external image路径虽有generation，但没有统一device generation、lease与in-flight retirement | 所有view/bind group/external texture绑定device epoch，stale handle在submit前拒绝 |

### 5.5 Editor generation、retained present 与 native surface

| ID | 差距 | 工程化要求 |
|---|---|---|
| RUIR-P1-033 | ordinary live full/damage conversion显式传 `generation=None`，测试也锁定unversioned | 每个immutable chrome frame/segment发布producer generation，damage带changed ranges |
| RUIR-P1-034 | versioned draw list主要由 `native_resize_generation` 特殊路径产生，resize代替不了内容generation | 分离content/layout/resource/surface generation，cache key包含实际依赖集合 |
| RUIR-P1-035 | unversioned frame每次重建dependency plan、solid/image buffers和text layout/upload | stable frame命中compiled topology与persistent ranges；dirty frame只重编受影响segment |
| RUIR-P1-036 | full或damage更新后仍把retained texture完整copy到surface；4K RGBA8约31.64 MiB/次 | 评估direct render、partial copy/present与swapchain preserve；stats证明总bandwidth真实下降 |
| RUIR-P1-037 | native descriptor从winit只接受Win32 raw handle，其他平台直接`unsupported` | 按同一surface contract实现Windows/macOS/Linux provider及明确capability matrix |
| RUIR-P1-038 | surface format只选non-sRGB BGRA/RGBA UNORM，alpha只偏好Opaque/Auto，协商面过窄 | capability negotiation覆盖format/view/color/alpha/present/HDR，并提供typed拒绝原因 |
| RUIR-P1-039 | surface Lost/Outdated reconfigure不是完整device loss；pipeline/atlas/cache无统一rebuild epoch | 接Runtime09A device generation、quiesce、recreate、last-good和in-flight retirement |
| RUIR-P1-040 | submit/retained commit顺序较稳健，但producer generation、resource readiness、copy与present receipt未统一 | 发布 `UiPresentReceipt`，关联artifact/device/surface generation、submit、present和fallback |

### 5.6 Color、观测、测试与 failure truth

| ID | 差距 | 工程化要求 |
|---|---|---|
| RUIR-P1-041 | scene shape/image使用straight-alpha pipeline，native使用premultiplied pipeline；跨backend合同不统一 | canonical color携带alpha representation，compiler在唯一边界转换并验证shader/texture匹配 |
| RUIR-P1-042 | native以non-sRGB view采样并在byte space premultiplied blend，透明边缘虽修正但linear-light语义未定义 | 明确working/output color space与legacy byte mode；同输入跨backend有reference-space parity |
| RUIR-P1-043 | game CSS-like `#RRGGBB` float值与sRGB attachment/scene terminal composition之间缺正式authoring contract | 定义authoring transfer、linearization、tint/premultiply和attachment encode顺序 |
| RUIR-P1-044 | UI没有HDR paper white、wide gamut、ICC/display profile、screenshot/capture color metadata | 建立SDR-on-HDR、Editor HDR monitor与capture/export policy及平台资格 |
| RUIR-P1-045 | game renderer主要暴露text prepare report，缺总op/batch/draw/pass/switch/clip/upload/cache/GPU time | 抽取跨backend bounded stats schema并接RenderFrameProfile和product evidence |
| RUIR-P1-046 | icon/image/font pending、missing、decode、evict和unsupported常被placeholder或fallback吞平 | 每次fallback携带typed reason、resource generation、last-good与bounded diagnostic |
| RUIR-P1-047 | 544个test attribute缺跨primitive完整overlap、damage alpha/delete、device loss和产品window矩阵；GPU无adapter可静默返回 | required qualification必须证明目标实际执行；skip/unavailable是未通过，不是green |
| RUIR-P1-048 | 五份相关failure仍open；其中glyph/bitmap有前向实现，Text09 non-exhaustive construction源码已改为Default+field assignment但ledger未回传 | 建立failure/source/validation三态核对，只有managed terminal receipt才能改status |

## 6. P2 改进

| ID | 改进 | 预期产物 |
|---|---|---|
| RUIR-P2-001 | 把interface visualizer绑定实际compiled artifact，而不是独立CPU计划 | draw-op/clip/resource/batch/source-node inspection |
| RUIR-P2-002 | 为icon/vector cook选择成熟SVG/XML/path库并建立malformed/complexity corpus | bounded parser/tessellator fuzz与cook receipt |
| RUIR-P2-003 | 生成solid/image/三text route/decor两两重叠golden矩阵 | deterministic small framebuffer corpus |
| RUIR-P2-004 | 随机insert/remove/move/opacity/theme/text序列对拍full与damage | 1,000步无漂移pixel/property test |
| RUIR-P2-005 | 建立CJK/emoji/variable font/plugin icon/DPI/theme/VRAM pressure atlas churn基准 | occupancy/relocation/dirty/evict/hitch report |
| RUIR-P2-006 | cook常用pipeline/font face/glyph/icon/image的prewarm manifest | cold/warm startup和first-use evidence |
| RUIR-P2-007 | 1.0/1.25/1.5/2.0/3.0 DPI与fractional transform截图资格 | crispness、pixel snapping与UV bleeding matrix |
| RUIR-P2-008 | RenderDoc marker关联painter token、source node、clip、resource和cache reason | capture-to-source debug lookup |
| RUIR-P2-009 | 对command/style/clip/transform/image payload做fuzz admission | finite/budget/overflow/stale rejection corpus |
| RUIR-P2-010 | 为高频stats定义采样、cardinality、retention和redaction | bounded production telemetry schema |
| RUIR-P2-011 | 建立Windows/macOS/Linux native window screenshot与present receipt归档 | cross-platform visual evidence bundle |
| RUIR-P2-012 | 与参考实现建立同scene/字体/分辨率/quality/hardware的可复现benchmark | CPU/GPU/VRAM/bandwidth/visual-quality archive |

## 7. 与参考引擎的直接差异

| 能力 | Zircon 当前 | 参考源码证据 | 目标差异 |
|---|---|---|---|
| Painter order | game按资源分桶；Editor仅窄contract安全 | Unreal stable layer + strict batchability；Bevy统一TransparentUi phase；Fyrox/Godot ordered command | 所有primitive共用stable painter token，优化只跨已证明非重叠项 |
| Batch identity | interface key与产品consumer分离 | Unreal把resource/shader/primitive/effect/instance/clip/scene纳入兼容性 | canonical key覆盖pipeline/material/resource/sampler/clip/effect/device generation |
| Transform/clip | axis rect + scissor，transform DTO不执行 | Unreal arbitrary clip quad/stencil；Fyrox transform/opacity stack；Godot canvas transform/group | affine transform、nested clip/mask、group opacity与offscreen layer |
| Primitive breadth | scene仅rect/border/full image/text | Unreal box/rounded/border/line/spline/gradient/custom/postprocess；Bevy gradient/shadow/material/slice | typed brush/material/effect以capability和像素门逐项交付 |
| Atlas lifecycle | 多个私有owner；runtime icon无consumer且可越界 | Unreal texture/vector cache；Unity Graphics allocator/hash/mip/release/relayout | cooked identity、page allocator、dirty update、budget、device epoch和recovery |
| Text | game三route；Editor独立system font/nowrap | Unreal shaped text同batch体系；Bevy text进入统一phase；Fyrox按font atlas page绘制 | resolved text artifact、共享font/glyph owner、原位fallback和ordered run |
| Retained/damage | 有retained texture但patch合成错误、最终全图copy | Godot group/offscreen语义；参考共同要求明确目标与composite边界 | replace/backdrop-correct damage、可测partial bandwidth和typed receipt |
| Color/output | backend alpha/format策略分裂，无HDR/WCG | Unreal texture atlas含sRGB政策；Godot按HDR选择sRGB/linear target | 明确authoring/working/output/alpha/HDR/display合同 |

参考不是“照抄五家feature并集”。Unreal Slate负责复杂度上限和严格batch/clip证据；Bevy证明Rust render phase可统一typed primitive，但其简单clip限制必须保留警惕；Fyrox证明有序command/stack的最低一致性，不能复制其逐draw成本；Godot补充canvas group/material/HDR路径；Unity Graphics只用于atlas生命周期，不能被引用为完整UI renderer证明。

## 8. 目标架构

### 8.1 `UiPresentationArtifact`

Style/layout/text/component producer在同一generation barrier发布immutable artifact。每个节点输出stable painter token、affine transform、opacity/clip/effect stack handle、typed primitive、resource handle和source provenance；artifact显式带project/session/window/surface/tree/style/layout/text/resource schema generation。旧 `UiRenderCommand` 与 `UiSurfaceCommand` 只能作为迁移reader，不能继续成为两个终端权威。

### 8.2 `UiPresentationCompiler`

Compiler先按painter token生成一个 ordered op stream，再构建overlap/dependency graph。只有相邻兼容或经依赖证明不重叠的op可重排/合批；compile产物包含geometry/instance ranges、pipeline/material/resource key、clip/effect program、dirty segment和完整split reason。任何unsupported brush/effect必须在compile阶段typed reject或选择显式fallback。

### 8.3 Clip、layer 与 composite graph

Clip owner维护generational stack：axis-aligned走scissor，rounded/transformable clip走analytic mask或cached mask，arbitrary path走stencil/mask texture；深度/ID溢出返回有界错误。Opacity group、shadow、blur、backdrop和material isolation成为layer graph节点，临时target受VRAM/size/pass预算约束，最终按原painter位置composite。

### 8.4 `DeviceUiResourceService`

Font face、glyph、icon、image、vector与material共享外部resource/page协议：qualified content identity、variant/DPI、format/color/alpha、padding/mip、page/slot generation、dirty rect、CPU/GPU预算、lease、device epoch、eviction和recovery。各资源仍可使用专门allocator，但不得继续暴露互不兼容的生命周期。

### 8.5 Surface presenter 与 damage

Game viewport和Editor native window接收同一compiled artifact，但拥有不同target adapter。Presenter建立 `UiPresentReceipt`，绑定artifact/device/surface generation、acquire、record、submit、retained commit和present。Damage必须定义backdrop replay或isolated replace，且full/damage逐像素等价；partial路径是否获益由replayed ops、upload/copy bytes与GPU time证明。

### 8.6 Color、diagnostics 与 evidence

Canonical color声明authoring/working/output transfer与alpha representation；texture admission只转换一次，blend在选定working space完成，HDR/wide-gamut/display/capture有明确metadata。跨backend stats统一记录compile/cache/draw/pass/pipeline/clip/resource/upload/copy/VRAM/GPU time/fallback；evidence manifest绑定build、provider、adapter/driver、surface format、scene、quality和raw artifact。

## 9. 分层实施里程碑

### M0 · Owner、schema 与三项 P0 复现冻结

冻结painter token、artifact generation、resource/device/surface identity与颜色合同；为Runtime11C三项P0建立最小确定性pixel reproduction，任何后续优化必须持续通过。

### M1 · Canonical ordered artifact

让Runtime与Editor producer都编译到一个ordered op stream，产品实际消费 `UiBatchPlan` 等价能力；删除资源类别fanout提交和无consumer设计表面。

### M2 · Brush、transform、clip 与 group opacity

按rect/image/text -> rounded/border/nine-slice/gradient -> affine transform/nested clip -> group/effect layer顺序交付，每层附capability与pixel gate。

### M3 · Text/font/glyph hard cutover

Editor停止独立system font truth，统一resolved layout/glyph generation；三text route作为material选择插回原stream，关闭Glyph/Bitmap open failures。

### M4 · Icon/vector/image resource convergence

建立cooked icon/vector artifact、分页atlas、稳定slot、theme/DPI generation、image readiness与统一budget；runtime Icon不再走矩形fallback。

### M5 · Retained/damage correctness

实现backdrop restore或isolated replace，full/damage随机序列对拍；删除旧像素、半透明累积和AA边缘漂移。

### M6 · Generation cache 与数据移动

普通full/damage frame发布content generation，稳定帧compile/build/hash/upload为0；dirty frame只更新changed segments；评估partial copy/direct render。

### M7 · Device、surface、platform 与 color

接device epoch/recovery，补Windows/macOS/Linux surface provider，交付SDR/sRGB/linear/HDR/WCG/alpha/capture矩阵。

### M8 · Product hard cutover

App、Editor、WOC/Dynamic产品只走canonical artifact/presenter；删除legacy command terminal、平行font/icon/cache/stats authority和假Supported能力。

### M9 · Qualification 与性能比较

完成pixel、fault、device loss、atlas churn、多window/DPI、1K/10K/100K element、cold/warm、CPU/GPU/VRAM/bandwidth和同负载参考对比；原始证据归档后才允许发布性能结论。

## 10. 资格门

### 10.1 Order、artifact 与 batch 门

| Gate | 必须证明 |
|---|---|
| RUIR-GATE-001 | 所有primitive携带stable painter token，full stream顺序可从source node追踪到draw |
| RUIR-GATE-002 | solid/image/native/bitmap/SDF/decor两两重叠golden全部按source order输出 |
| RUIR-GATE-003 | batch compiler只合并相邻兼容或dependency证明不重叠的op |
| RUIR-GATE-004 | batch key覆盖pipeline/material/resource/sampler/clip/effect/device generation |
| RUIR-GATE-005 | unsupported primitive/effect typed reject或显式fallback，不静默丢字段 |
| RUIR-GATE-006 | game与Editor相同artifact在reference target像素一致 |
| RUIR-GATE-007 | legacy `UiRenderCommand`/`UiSurfaceCommand`不再是并列终端owner |
| RUIR-GATE-008 | debug/RenderDoc可从draw反查artifact generation、painter token与source |

### 10.2 Geometry、clip、opacity 与 effect 门

| Gate | 必须证明 |
|---|---|
| RUIR-GATE-009 | translation/scale/rotation/skew/pivot/affine composition与inverse规则确定 |
| RUIR-GATE-010 | transform、render bounds、hit/a11y geometry使用同代数据 |
| RUIR-GATE-011 | nested axis clip走scissor且不改变像素；rounded/path clip走正确mask/stencil |
| RUIR-GATE-012 | clip stack overflow、invalid path、oversized mask在submit前有界失败 |
| RUIR-GATE-013 | subtree group opacity与离屏reference在重叠子元素上逐像素一致 |
| RUIR-GATE-014 | shadow/blur/backdrop/filter/material isolation保留原painter位置 |
| RUIR-GATE-015 | temporary target受尺寸、pass、VRAM预算限制并有degrade receipt |
| RUIR-GATE-016 | fractional DPI/transform下pixel snapping与clip边缘无抖动/泄漏 |

### 10.3 Text、glyph、icon 与 image 门

| Gate | 必须证明 |
|---|---|
| RUIR-GATE-017 | Editor与game相同font artifact选择相同face、fallback、metrics和glyph generation |
| RUIR-GATE-018 | direction/language/wrap/alignment/rich run/selection/composition不在RHI handoff丢失 |
| RUIR-GATE-019 | text backend fallback span插回原painter位置且不重排相邻primitive |
| RUIR-GATE-020 | stable text frame shape/layout/atlas plan/instance allocation/upload均为0 |
| RUIR-GATE-021 | glyph bitmap与SDF open failures获得managed Cargo、scale和product pixel终态 |
| RUIR-GATE-022 | 所有production semantic Icon从asset到GPU pixel可达，不再画通用矩形 |
| RUIR-GATE-023 | icon atlas分页、oversized/admission、UV范围、padding和DPI/theme generation正确 |
| RUIR-GATE-024 | image ready/pending/missing/failed/evicted/stale分别产生typed output与stats |

### 10.4 Cache、damage、resource 与 device 门

| Gate | 必须证明 |
|---|---|
| RUIR-GATE-025 | ordinary full/damage frame都有content/layout/resource generation，不再固定None |
| RUIR-GATE-026 | stable framecompile/topology/geometry/hash/upload为0，dirty只更新changed range |
| RUIR-GATE-027 | full与damage对opaque/50% alpha/delete/move/text AA/rounded edge逐像素一致 |
| RUIR-GATE-028 | 同一半透明damage重复100次不积累，随机1,000步无漂移 |
| RUIR-GATE-029 | damage收益报告replayed ops、upload/copy bytes、pass、GPU time与retained VRAM |
| RUIR-GATE-030 | icon/glyph/image CPU shadow、GPU/local/shared page分别受硬预算和eviction约束 |
| RUIR-GATE-031 | device loss/recreate后所有旧page/view/bind group被判stale且可有界重建 |
| RUIR-GATE-032 | in-flight resource retirement、surface retry与retained commit无use-after-retire/假成功 |

### 10.5 Color、surface 与 platform 门

| Gate | 必须证明 |
|---|---|
| RUIR-GATE-033 | 每个texture/color明确straight或premultiplied，转换恰好一次 |
| RUIR-GATE-034 | sRGB authoring到linear blend再到output encode的reference像素正确 |
| RUIR-GATE-035 | scene与native backend在同working/output space颜色与透明边缘一致 |
| RUIR-GATE-036 | SDR UI合成到HDR scene的paper white、tone/composite顺序有reference |
| RUIR-GATE-037 | wide gamut/ICC/display/capture metadata不会静默丢失或误标 |
| RUIR-GATE-038 | surface format/view/alpha/present capability协商产生typed effective receipt |
| RUIR-GATE-039 | Windows/macOS/Linux native provider通过同一resize/recreate/present矩阵 |
| RUIR-GATE-040 | screenshot、stream、frame capture与屏幕输出在声明color space内一致 |

### 10.6 Product、fault 与性能门

| Gate | 必须证明 |
|---|---|
| RUIR-GATE-041 | App、Editor、WOC/Dynamic产品都消费canonical artifact，不存在隐藏平行renderer |
| RUIR-GATE-042 | no-adapter、GPU test skip、unsupported surface被计为未资格，不计green |
| RUIR-GATE-043 | shader/pipeline/image/font/atlas failure有last-good或明确frame failure与receipt |
| RUIR-GATE-044 | 1K/10K/100K element与atlas churn保存CPU、alloc、upload、draw、pass、GPU、VRAM原始证据 |
| RUIR-GATE-045 | cold/warm prewarm分别报告first-use hitch和steady-state cache hit |
| RUIR-GATE-046 | 多window、DPI/theme/locale切换、surface resize/device reset不串generation |
| RUIR-GATE-047 | 五份open failure的source、validation、ledger状态一致并有terminal回执 |
| RUIR-GATE-048 | “优于Unreal/参考实现”只可在同硬件/驱动/build/scene/font/quality/分辨率下发布 |

## 11. 必须重开或继续执行的既有计划

| 计划 / failure | 当前状态 | Runtime79 要求 |
|---|---|---|
| Runtime11C | 三项P0仍可由当前源码直接复现 | 先建立pixel reproduction，再按M1/M4/M5关闭，不得用结构测试代替 |
| Runtime09 transient extraction | generation streaming子叶已绿；typed range、payload clone、scale/parity仍open | M1/M6接收generation-owned artifact与稳定帧counter |
| Runtime09 dynamic UI extract | menu/HUD稳定帧仍缺component/viewport generation artifact | 与canonical presentation generation一起关闭，禁止另建String cache |
| Text04 glyph instance | 前向实现已删除双层vertex，managed/product证据待完成 | 复核current source，执行exact scale/instance/WGPU pixel gate后return fixed |
| Text04 bitmap dirty/shadow | mixed-storage/shadow前向修复存在，Cargo/WGPU产品证据待完成 | 按dirty region、upload bytes、persistent slot pixel完成回执 |
| Text09 stats construction | 当前源码已用`Default`后字段赋值，原non-exhaustive struct expression不再存在 | 不据静态观察宣称fixed；重跑原validation并写canonical return |
| [Runtime79 UI sRGB coverage 与 native drop order](79/failure-2026-08-25-ui-srgb-coverage-and-native-drop-order.md) | open；颜色夹具与 UI surface native owner 待收口 | 保持 coverage 与 transfer 分离，并以完整受管 `zr_rhi_wgpu --lib` 终态回传 |

## 12. 实施顺序与声明边界

1. 先冻结三项继承P0的确定性pixel reproduction和canonical identity，不在错误输出上继续堆batch优化。
2. 再硬切ordered artifact与product consumer，统一font/icon/image/clip/effect资源边界；禁止保留“game一套、Editor一套、interface一套”。
3. 正确性闭合后完成ordinary generation cache、persistent ranges、damage bandwidth和device/surface恢复。
4. 最后做HDR/跨平台/fault/scale与参考benchmark；性能声明必须同时绑定视觉质量和原始证据。

本报告是review/refactor plan，不是implementation completion。`review_complete`只表示本次264文件、23参考文件与五份failure的当前源码纵向审查已完成；三项继承P0、48项P1、12项P2、48项gate和M0-M9均待实施或验证。tooling按用户要求不在本轮新增专题，后续Rust迁移时只消费这里定义的product evidence contract。

## 13. 最终结论

Zircon 已有比“临时画几个矩形”更扎实的局部GPU底座：真实glyph atlas/SDF、instance buffer、dirty upload、dependency batching、bounded image cache、retained surface和GPU统计都值得保留。但工程级renderer的核心不在局部算法数量，而在所有primitive是否共享一个不可破坏的painter order、所有transform/clip/opacity/effect是否有一致语义、所有font/icon/image是否有稳定generation与device生命周期，以及full/damage/game/Editor是否能由同一artifact证明像素和性能。

当前答案仍是否定的。游戏路径稳定破坏跨资源顺序，runtime图标仍是产品占位，Editor damage仍可累积旧像素；同时更宽的interface DTO没有产品consumer，普通Editor generation cache不工作，atlas与color/device owner分裂。完成48项资格门之前，Zircon只能称为拥有多组可复用GPU UI子系统，不能称为已经形成与Unreal Slate同级、更不能称为性能和表现优于Unreal的完整工程级UI renderer。
