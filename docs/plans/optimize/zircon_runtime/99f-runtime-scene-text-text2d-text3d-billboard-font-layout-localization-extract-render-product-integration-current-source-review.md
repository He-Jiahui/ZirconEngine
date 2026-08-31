---
title: Runtime Scene Text、Text2D、Text3D、Billboard、Font、Layout、Localization、Extract、Render 与 Product Integration 当前源码工程化差距
category: zircon_runtime
report_id: Runtime105
review_date: 2026-08-22
baseline_head: bee4c707b714738346b49bba15c59468b8bd9b39
baseline_epoch: 339
related_code:
  - zircon_runtime/src/core/framework/text
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/text
  - zircon_runtime/src/text/atlas/shaders/glyph_atlas_pipeline.wgsl
  - zircon_runtime/src/asset/assets/font.rs
  - zircon_runtime/src/asset/assets/font_source.rs
  - zircon_runtime/src/asset/artifact/cache_payload/font.rs
  - zircon_runtime/src/asset/assets/scene
  - zircon_runtime/src/asset/importer/ingest/import_font_asset
  - zircon_runtime/src/scene/components/scene
  - zircon_runtime/src/scene/world/records.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/scene/world/derived_state.rs
  - zircon_runtime/src/scene/world/typed_api/fixed_components.rs
  - zircon_runtime/src/scene/world/typed_api/projection_rebuild.rs
  - zircon_runtime/src/scene/world/project_io
  - zircon_runtime/src/graphics/scene/scene_renderer/ui
  - zircon_runtime/src/ui/text
  - zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/text.rs
  - zircon_runtime/src/dynamic_api/session/hud.rs
  - zircon_runtime/src/dynamic_api/session/extract.rs
  - zircon_runtime/src/dynamic_api/session/menu.rs
  - zircon_editor/src/core/commands/defaults.rs
  - zircon_editor/src/scene
  - zircon_editor/src/ui/host/scene_inspection_publication.rs
  - zircon_editor/src/ui/workbench/event/node_kind_from_id.rs
  - zircon_editor/src/ui/workbench/event/node_kind_id.rs
  - zircon_editor/src/ui/workbench/event/menu_item_binding.rs
tests:
  - zircon_runtime/src/core/framework/text/tests.rs
  - zircon_runtime/src/asset/tests/assets/font.rs
  - zircon_runtime/src/text/glyph_artifact/tests.rs
  - zircon_runtime/src/text/font/database/tests
  - zircon_runtime/src/text/layout/line_break/tests.rs
  - zircon_runtime/src/text/shaping/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_renderer/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/tests
  - zircon_runtime/src/scene/tests
  - zircon_editor/src/tests
plan_sources:
  - docs/plans/optimize/zircon_runtime/70-runtime-scene-text-text2d-text3d-billboard-font-layout-localization-extract-render-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/80-runtime-font-asset-source-cook-database-face-fallback-variation-color-resolved-glyph-cache-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/81-runtime-text-shaping-unicode-bidi-script-run-cluster-line-break-wrap-layout-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/82-runtime-text-editing-document-selection-caret-hit-test-ime-composition-clipboard-secure-text-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/83-runtime-localization-internationalization-locale-culture-message-format-plural-number-date-string-table-resource-fallback-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/84-runtime-rich-text-markup-parser-token-style-span-inline-object-link-image-table-list-layout-selection-accessibility-security-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/61-runtime-scene-world-level-registry-lifecycle-project-io-snapshot-clone-serialization-schema-transaction-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/62-runtime-scene-hierarchy-transform-propagation-reparent-activation-mobility-visibility-bounds-render-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/63-runtime-scene-reflection-type-schema-registry-dynamic-component-property-address-inspection-artifact-subscription-editor-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/64-runtime-resource-authority-asset-handle-load-request-state-machine-version-lease-cache-dependency-reload-cancellation-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/94-runtime-visibility-spatial-index-bounds-frustum-occlusion-hzb-culling-batching-instancing-gpu-scene-indirect-submission-instance-lifecycle-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/03-scene-prefab-selection-mode-gizmo-picking-review.md
  - docs/plans/optimize/zircon_editor/05-inspector-reflection-property-authoring-customization-review.md
  - docs/plans/optimize/zircon_editor/33-localization-string-table-culture-translation-import-export-fallback-pseudo-localization-preview-authoring-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Classes/ActorFactories/ActorFactoryTextRender.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Components/TextRenderComponent.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/TextRenderActor.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Components/TextRenderComponent.cpp
  - dev/bevy/crates/bevy_sprite/src/text2d.rs
  - dev/bevy/crates/bevy_sprite_render/src/text2d/mod.rs
  - dev/bevy/crates/bevy_text/src/text.rs
  - dev/Fyrox/fyrox-ui/src/text.rs
  - dev/Fyrox/fyrox-ui/src/formatted_text.rs
  - dev/Fyrox/fyrox-ui/src/formatted_text/run.rs
  - dev/Fyrox/fyrox-ui/src/formatted_text/textwrapper.rs
  - dev/godot/scene/3d/label_3d.h
  - dev/godot/scene/3d/label_3d.cpp
  - dev/godot/editor/scene/3d/gizmos/label_3d_gizmo_plugin.h
  - dev/godot/editor/scene/3d/gizmos/label_3d_gizmo_plugin.cpp
  - dev/Graphics/Tests/SRPTests/Projects/MultipleSRP_Tests/Assets/TextMesh Pro/Shaders/TMP_SDF.shader
  - dev/Graphics/Tests/SRPTests/Projects/MultipleSRP_Tests/Assets/TextMesh Pro/Shaders/TMP_SDF Overlay.shader
  - dev/Graphics/Tests/SRPTests/Projects/MultipleSRP_Tests/Assets/TextMesh Pro/Shaders/TMP_SDF-Surface.shader
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Samples~/Common/TextMesh Pro/Resources/Fonts & Materials/TMP_Node.hlsl
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 99f · Runtime Scene Text、Text2D、Text3D、Billboard、Font、Layout、Localization、Extract、Render 与 Product Integration 当前源码工程化差距

## 1. 结论

本轮对 Runtime70 做当前源码刷新，而不是重复审查 Runtime80-84 已分别拥有的 font、shaping、editing、localization 与 rich-text 父问题。结论仍然明确：Zircon 已有规模可观的 UI 文本系统和一层新的中立 shaping 合同，但没有任何 first-class Scene Text 产品能力。

与旧报告相比，当前源码有两项重要进展必须保留。第一，`core::framework::text` 已提供中立 `TextLayoutService`、typed font request、direction、writing mode、render mode、glyph、run、metrics 与 `TextLayoutError`；`TextGlyph` 保留 source/visual range、advance、position、offset、face/instance handle、rotation、BiDi level、flags 和 rasterization 需求。第二，UI renderer 的 glyph artifact 会保留 shaping 产生的 glyph identity，并通过 artifact/font generation 做 cache invalidation，不应再断言 UI renderer 必然重新逐字符 shaping。

但这层合同目前仍停留在 shaping 级别：`TextLayoutService` 只有 render-mode resolution、direction resolution 与 `shape`，没有 Scene source、paragraph constraint、line artifact identity、world-space bounds、dirty frontier、cancellation、view policy 或 render extract。`ScreenSpaceUiTextBatch` 仍是 graphics UI 私有类型，身份来自 UI tree/node/source range，几何来自 `UiFrame` 和 clip frame；glyph atlas shader 接收 `screen_rect_px`，按 viewport 直接映射 clip space并固定 `z = 0`。另一条 `zr_rhi_wgpu::ui_surface::text` 路径也围绕 glyphon `FontSystem`、`TextAtlas`、projection size 与 UI render pass，不是世界文本 backend。

精确搜索进一步确认能力仍完全缺席：在 tracked `assets`、`zircon_runtime/src`、`zircon_editor/src`、`zircon_app/src` 中，`SceneText`、`Text2D/Text2d`、`Text3D/Text3d`、`Label3D/Label3d`、`WorldText`、`BillboardText` 命中数为 **0**。`NodeKind` 仍只有 Empty、Camera、Cube、Mesh 与五类 light；`SceneNode`/`NodeRecord` 虽新增 Sprite2D/Mesh2D，`SceneEntityAsset` 虽有 terrain/tilemap/prefab/script，却都没有 text。`RenderFrameExtract` 只有 geometry、animation、lighting、environment、post-process、debug、sprites、particles 与 visibility。Editor 创建命令和 node-kind 映射同样没有 Scene Text。

因此当前项目无法以正式能力创建、保存、重开、复制、prefab 化、反射检查、脚本修改、增量布局、剔除、深度渲染、billboard、拾取或诊断世界文字。本轮登记 **0 项新增 P0、48 项 P1、12 项 P2、M0-M10 与 48 项资格门**。缺少尚未公开的能力按 P1 登记；Scene I/O 字段丢失、localization 解析、font/shaping/rich-text、通用 visibility/material/residency 和产品 HUD 硬编码继续由既有 owner 唯一计数，不在本文重复制造 P0。

## 2. 审查边界与所有权冻结

### 2.1 Canonical owner

| 领域 | Canonical owner | Runtime105 责任 | 不在本文重复登记 |
|---|---|---|---|
| Scene source / component / lifecycle | `zircon_runtime::scene` | SceneText 实例、持久 source、spatial policy、dirty generation、clone/snapshot/project I/O | 通用 World transaction、hierarchy、activation、mobility 父问题 |
| Neutral text semantics | `zircon_runtime::core::framework::text` + runtime text service | Scene consumer 所需的稳定请求、artifact、generation、typed outcome | Runtime80-84 的 font cook、Unicode shaping、editing、localization、rich parser 父问题 |
| Neutral render contract | `zircon_runtime::core::framework::render` | immutable SceneText delta/extract DTO，不泄露 WGPU/glyphon/UI tree | 通用 frame/view/visibility contract 父问题 |
| GPU resource and submission | `zircon_runtime::graphics` | world glyph residency、instance buffer、pipeline/material key、depth/phase、multi-view submission | UI renderer 私有 batch、通用 material/PSO/residency 父问题 |
| Authoring | `zircon_editor` | create/inspect/gizmo/picking/preview/undo/save/reopen adapter | 通用 selection、undo、inspector、localization workspace 父问题 |
| Product composition | runtime UI + project | overlay UI 与 world text 的明确域边界 | Runtime43/App06 的 Vampire/HUD/menu 硬编码 |

固定架构仍是 `zircon_app`、`zircon_runtime`、`zircon_editor` 三个 public root package；runtime 内部遵循 `core/{runtime,framework,manager,math,resource}` spine。不得为 Scene Text 新增 root crate，不得把 `ScreenSpaceUiTextBatch` 提升成公共 scene contract，不得把 graphics 私有 glyph instance 保存进 scene asset，也不得用 compatibility re-export 保留旧路径。

### 2.2 Current-source 物理冻结

本轮入选 892 个 tracked Rust/WGSL 文件，覆盖中立 text contract、共享 text/font/layout/glyph artifact、Scene schema/component/project I/O、frame extract、两条 UI GPU 文本路径、Dynamic HUD、Editor Scene 与 App 产品入口。计算规则为：repo-relative path 小写并排序，逐文件计算 lowercase SHA-256，以 `path<TAB>hash` 按 LF 连接且末尾无 LF，再对 manifest 计算 SHA-256。

| 范围 | 文件 / 行 / 非空行 / bytes / test attrs / ignored | Fingerprint |
|---|---:|---|
| production-like 与 inline tests | **677 / 112,287 / 103,072 / 3,964,222 / 610 / 2** | `58a9338040129341d3b02e41338cd72acaea0cb084a1d6418db5a66758abb621` |
| dedicated test files / directories | **215 / 53,768 / 48,990 / 1,885,745 / 1,469 / 15** | `348163630382f0e987066191d860da7090a58968b018dbb0e8ccce9f8d275a87` |
| 去重合计 | **892 / 166,055 / 152,062 / 5,849,967 / 2,079 / 17** | `b4bdc94dd121b18df9d12aff0c033d39700f85c675963ff668a4c552862bb235` |

这里的 2,079 个 test attribute 主要证明共享 text、UI renderer、Scene 通用行为和 Editor 基础设施；17 个 ignored 中包含多项人工 framebuffer/performance evidence。精确 SceneText 名称命中仍为 0，因此这些测试不包含 SceneText component、save/reopen、world depth、billboard、visibility、picking、culture reload、device loss 或同负载 benchmark 资格。

tracked `assets`、`zircon_app`、`zircon_editor/src` 中可作为产品文本/配置/源码的 `.rs/.zui/.toml/.json/.ron/.zr/.wgsl` 文件共有 **5,957** 个；同一组 SceneText 精确名称在 assets + Runtime + Editor + App 中仍为 0。相反，Dynamic runtime 仍命中 `gameplay.hud_text`、`vampire.hud_text`、`current_ui_extract` 与字符串分类逻辑；这只证明 UI fallback/product 污染依旧存在，不是 Scene Text 的替代实现。

冻结时 shared worktree 不是 pristine：Runtime77 正在修改 UI transaction/template，Runtime73/81/89 batch 与 full post-main integration 正在同步 text/Scene/App 父计划修复，Runtime104 的 99e 报告也尚未提交。本轮不接管、不回滚这些变更；表中数字是最后一次完整测量快照，不表示共享树此后静止。结论绑定上述 HEAD、该测量快照与精确字段证据，实施前必须重算指纹、零命中、schema 字段守恒和 shader/pipeline 状态。

### 2.3 参考引擎物理冻结

五类参考共 19 个实际存在文件、9,405 行、8,168 非空行、328,432 bytes、17 个 Rust test attribute，fingerprint 为 `91788e58f04cf82be1aaf5f23fa77c8e3507717d78487290f7722564b57a2636`。它们用于职责与资格对照，不是机械移植模板。

| 参考 | 可采用证据 | 必须超越或拒绝照搬 |
|---|---|---|
| Unreal | `UTextRenderComponent` 是持久 `PrimitiveComponent`，拥有 localized `FText`、Font、Material、alignment、world size、X/Y scale、spacing、color 与 always-render-as-text；scene proxy、bounds、material enumeration、PSO precache、localization revision、static/dynamic/ray-tracing path 均有生命周期 | 本地 `TextRenderComponent.cpp` 的 legacy 路径按 character 解析并生成 quad，Unicode/shaping 与大规模性能都不是 Zircon 上限；不复制 UObject/RHI proxy 形态 |
| Bevy | `Text2d` 通过 required components 绑定 layout/font/color/line-height/letter-spacing/bounds/anchor/visibility/transform；change-driven layout 按可见 camera/render layer 选择 scale，计算 AABB，extract 尊重 `ViewVisibility` 并投影 positioned glyph 为 extracted sprite | 每 entity 取 camera 最大 scale、部分 atlas fatal error panic 与 ECS schedule 形态都不作为最终质量上限 |
| Godot | `Label3D` 覆盖 font/language/direction/wrap/justification/outline、billboard/fixed-size、double-sided/no-depth、alpha/AA/filter/priority；TextServer RID shaping、dirty state、surface key、AABB 与 Editor triangle gizmo picking 都闭环 | dirty 后 CPU 重新生成 surface mesh 的实现不直接作为超越 Unreal 的性能架构；不复制 Variant/RID owner 模型 |
| Fyrox | 本地快照的 `Text`/`FormattedText`/run/wrapper 提供 UI rich run、wrap、measure/arrange、glyph brush/atlas 与 shadow 的 Rust 分层证据 | 对 `Text2D/Text3D/Label3D` 精确扫描为 0；只能作为共享文本服务下限，不能把 UI Widget 当 Scene Node |
| Unity Graphics | 本地 TMP SDF、Overlay、Surface shader 展示 UI transparent/overlay depth、stencil/cull/blend、perspective filter、face/outline/underlay/glow/bevel，以及 lit surface/shadow caster 变体 | 本地 Graphics 树没有完整 TextCore/TMP C# runtime/Editor，不能据 shader 样例宣称完整 lifecycle 或产品能力 |

## 3. 当前源码逐层事实

### 3.1 可保留的中立文本底座

`core::framework::text` 是旧 Runtime70 之后最重要的架构进展。其 DTO 不依赖 UI tree 或 WGPU，`TextFontFaceHandle { index, generation }` 能表达 generation-qualified face，`TextGlyph` 能保留 source/visual mapping 与真实 backend glyph identity，`TextLayoutError` 能区分 font unavailable、fallback exhausted、unsupported mode、backend unavailable 和 font generation changed。Scene Text 必须扩展并消费这层合同，而不是在 Scene、Editor 或 renderer 再造一套 shaping API。

但命名为 `TextLayoutService` 不等于已经具备工程级 layout。当前 trait 没有 paragraph width/height、wrap policy、alignment、overflow、line collection、artifact ID、revision vector、deadline/cancel token、budget 或 partial result；`TextShapeResult` 只有 runs、metrics 和 resolved direction。这是可靠 shaping foundation，不是 SceneText 编译器。

### 3.2 UI glyph artifact 是真实能力，但 owner 错域

`ResolvedTextGlyphArtifact` 到 `ScreenSpaceUiGlyphArtifactLine` 的投影保留 glyph source range、visual range、font generation 和 immutable `Arc` identity；UI batch 的 `preserve_shaped_glyphs` 明确禁止 renderer 在 reload 后用第二次 run-local shaping 替换 identity。这项能力可以下沉为共享 artifact contract。

它目前仍被 UI 语义包围：cache identity 依赖 artifact/line allocation address，route identity 依赖 tree/node/source range，batch 持有 owned `String`、`UiFrame`、clip frame、UI alignment/wrap/paint/effect/decorations。SceneText 不能直接持有这些类型，必须建立 stable semantic artifact ID 与 Scene-owned generation。

### 3.3 Scene source、schema 与持久化完全缺席

`NodeKind` 只有九个枚举值。`SceneNode` 与 `NodeRecord` 有 camera、mesh、Sprite2D、Mesh2D、五类 light、physics 与 animation，没有 text。`SceneEntityAsset` 进一步包含 post-process、terrain、tilemap、prefab、scripts，仍无 literal/localized text、font、style 或 spatial policy。

`World::from_scene_asset` 只按 camera/light/post-process/mesh/script 重建 `NodeKind`，`to_scene_asset` 只映射已知 component；project document normalization 同样只能从 camera/light/mesh 恢复 kind。`WorldPersistentState`、fixed snapshot、projection rebuild 与 record conversion 没有 SceneText 字段守恒。即使临时向 dynamic component 塞字符串，也不会形成可迁移、可反射、可保存的正式 component。

### 3.4 Extract、visibility 与 renderer 没有 world text 合同

`RenderFrameExtract` 的 top-level fields 不含 text；`World::render`、scene snapshot 和 visibility producer 也没有 text packet。没有 created/changed/removed delta、entity、world transform、layer mask、mobility、bounds、view mask、artifact generation、material generation 或 fallback state。

`glyph_atlas_pipeline.wgsl` 的 instance 是 `screen_rect_px`、UV、color、page index；vertex shader按 viewport 转 clip space并写 `vec4(ndc, 0.0, 1.0)`。这条 pipeline 没有 world/view/projection matrix、depth、previous transform、billboard basis、fixed-screen-size、shadow、ray tracing、object ID 或 picking。给 instance 再加一个 transform 字段不能补齐这些合同。

### 3.5 Editor 与产品路径没有真实 authoring

Editor default commands 只创建 Cube、Camera 和五类 light；`node_kind_from_id`、`node_kind_id` 与 menu binding 完整匹配当前九种 kind，但没有 text。Scene inspection publication、viewport overlay 与 runtime HUD 只会消费现有 scene fields 或构造 screen-space UI。没有 SceneText create command、Inspector schema、gizmo、triangle/quad picking、font/style picker、localization binding、preview、undo/save/reopen 或 capability receipt。

Dynamic runtime 的 `hud.rs` 会查找 `gameplay.hud_text`/`vampire.hud_text` 并按文本模式构造 HUD；Editor viewport controller也有 `runtime_hud_text()`。这些属于 runtime overlay UI，必须最终由项目 UI 资产和 typed gameplay view model 拥有，不能借此声称 world-space label、nameplate 或 damage number 已实现。

## 4. 目标架构

```text
SceneTextSourceDocument
  + SceneTextStyleDescriptor
  + SceneTextSpatialPolicy
  + qualified Font/Localization handles
        |
        v
SceneTextCompiler (dirty graph, cancellation, generations, typed outcome)
        |
        v
Immutable SceneTextLayoutArtifact
  glyph runs + lines + source map + local geometry + local bounds
  source/style/font/culture/layout generations + completeness
        |
        v
SceneTextDeltaExtract in core::framework::render
  created / changed / removed + entity/view/layer/transform/bounds/material
        |
        v
Graphics SceneTextResourceService
  atlas residency + prepared generation + PSO/material/phase + device recovery
        |
        v
SceneTextSubmissionService
  multi-view projection + depth/visibility + batching + diagnostics receipt
```

| 类型 / 服务 | Owner | 必须表达的合同 |
|---|---|---|
| `SceneTextSourceDocument` | Scene | literal 或 localized identity、arguments、rich-span reference、source revision；不得只保存已解析字符串 |
| `SceneTextStyleDescriptor` | framework text | qualified font handle/family/fallback、size、line/letter spacing、alignment、wrap/overflow、language/direction、paint/effect policy |
| `SceneTextSpatialPolicy` | Scene | Text2D/Text3D plane、world units、anchor/pivot、billboard axis、fixed-screen-size、depth/double-sided、distance/LOD |
| `SceneTextComponent` | Scene | source/style/spatial/material handles、component generation、capability state；不拥有 GPU object |
| `SceneTextLayoutArtifact` | runtime text | immutable lines/runs/glyphs/source map/local geometry/bounds、dependency generations、typed completeness |
| `SceneTextDeltaExtract` | render framework | created/changed/removed、stable entity key、world/current-previous transform、view/layer/bounds/artifact/material generations |
| `PreparedSceneTextGeneration` | graphics | glyph residency leases、instance ranges、pipeline/material keys、device generation、last-good/failure state |
| `SceneTextSubmissionReceipt` | graphics/diagnostics | requested/effective policy、culled/fallback/error reason、generation vector、glyph/draw/byte/time budgets |
| `SceneTextAuthoringAdapter` | Editor | create/inspect/gizmo/pick/preview/localization/undo/save/reopen；复用 runtime compiler |

Billboard 与 fixed-screen-size 是世界对象的 view projection policy；HUD、菜单、字幕、输入框和 accessibility tree 属于 runtime UI。两者可共享 font、shaping、layout semantic 与 glyph residency，不共享 Scene/UI identity、persistence、hit test、depth、ordering 或 authoring owner。

## 5. 新增 P0

本轮没有新增 P0。SceneText 尚未出现在 component、asset、Editor command、capability catalog 或 renderer surface，没有对用户宣称成功后静默丢失的独立新路径。若未来公开 create/capability，而 save/reopen、extract 或 render 仍缺席，应由 Runtime61/Editor capability owner升级为数据丢失或假成功 P0；不得提前在本文重复计数。

Runtime61 的 Scene 字段守恒、Runtime80-84 的 text 父能力、Runtime94 的 visibility、Runtime09C/09D 的 material/residency、Editor03/05/33 与 Runtime43/App06 的现有 P0/P1 继续唯一拥有对应问题。

## 6. P1 差距与重构完成定义

### 6.1 Source、schema 与 ownership

| ID | 当前差距 | 重构完成定义 |
|---|---|---|
| SCENETEXT-P1-001 | 全仓没有 first-class `SceneTextComponent` 或 capability | 建立唯一 Scene owner、typed component 与 explicit Available/Unavailable/Degraded capability；禁止 dynamic JSON/UI command 冒充 |
| SCENETEXT-P1-002 | 没有 versioned SceneText source document | 定义 schema version、literal/localized source union、argument binding、rich reference、stable source ID 与 migration |
| SCENETEXT-P1-003 | 没有 Text2D/Text3D 空间模式 | 定义 plane/basis/world unit/anchor/pivot/transform inheritance，非法或退化 basis 返回 typed validation error |
| SCENETEXT-P1-004 | Scene 没有 qualified font dependency | component 引用 Runtime80 提供的 typed font handle/request，不保存裸 filesystem path 或 graphics font ID |
| SCENETEXT-P1-005 | Scene 没有 localized identity | 保存 table/namespace/key/arguments/culture policy 与 fallback provenance，不把 resolved string 作为唯一 truth |
| SCENETEXT-P1-006 | 没有稳定 style descriptor | font size、line/letter spacing、alignment、wrap、overflow、language/direction/writing mode 与 paint policy版本化并可校验 |
| SCENETEXT-P1-007 | 没有 material/effect policy | 区分 unlit/lit、face/outline/shadow/underlay、alpha/cull/depth/double-sided；引用 material contract而不是 shader name字符串 |
| SCENETEXT-P1-008 | component lifecycle 未定义 | attach/detach/enable/disable/clone/destroy/reparent/mobility change产生精确 dirty event 与 generation |
| SCENETEXT-P1-009 | `NodeKind`、`SceneNode`、`NodeRecord` 无 text | hard-cut 接入正式 kind/component，不用 optional dynamic bag 或 compatibility enum映射 |
| SCENETEXT-P1-010 | `SceneEntityAsset` 与 project I/O 无字段守恒 | save/reopen/copy/paste/prefab/duplicate/export逐字段守恒，unknown future fields按统一schema policy处理 |
| SCENETEXT-P1-011 | reflection、property path、script surface 无 text | 发布 typed reflection schema、qualified property address、script API与变化 receipt，保持同一 Scene authority |
| SCENETEXT-P1-012 | validation 与 migration outcome 缺席 | 非法 font/style/spatial/localization/material组合在加载/编辑/运行时返回结构化 outcome，不 panic、不静默 default |

### 6.2 Layout artifact、增量编译与 bounds

| ID | 当前差距 | 重构完成定义 |
|---|---|---|
| SCENETEXT-P1-013 | 中立 `TextLayoutService` 实际只到 shape | 扩展稳定 paragraph/layout request与artifact service；width/height/wrap/alignment/overflow/line集合不依赖 UI DTO |
| SCENETEXT-P1-014 | UI artifact identity 依赖 Arc address/line index | 建立 stable semantic artifact ID、artifact generation 与 content/dependency key，可跨线程、reload和缓存层验证 |
| SCENETEXT-P1-015 | 没有 source/style/font/culture generation vector | artifact明确记录全部依赖generation；stale result不得覆盖较新Scene source |
| SCENETEXT-P1-016 | Scene constraints 与 overflow 未定义 | width/height/max-lines/clip/ellipsis/scale-to-fit策略有确定测量、布局、bounds与diagnostic语义 |
| SCENETEXT-P1-017 | rich spans 仅在 UI consumer 中闭环 | Scene compiler消费 Runtime84 canonical span artifact，run style与inline object不得在renderer重解析markup |
| SCENETEXT-P1-018 | BiDi/vertical/source mapping未形成 Scene artifact | 每 glyph/run/line保留logical/visual/source mapping、rotation与writing mode，picking和diagnostic能回溯源范围 |
| SCENETEXT-P1-019 | 没有 line/glyph/local bounds | artifact发布 conservative local ink/layout/effect bounds、baseline与anchor；空白、outline、shadow、vertical text均有定义 |
| SCENETEXT-P1-020 | billboard/fixed-size 与 layout/view依赖未分离 | view-independent shape/layout只编译一次，view-dependent projection单独缓存；多camera不重复shaping |
| SCENETEXT-P1-021 | 没有 dirty dependency graph | source/style/font/culture/transform/material/view各自命中最小 invalidation frontier，禁止每帧全量 rebuild |
| SCENETEXT-P1-022 | 没有 cancellation/coalescing/deadline | superseded generation可取消或丢弃，交互编辑有frame budget与优先级，不能让长文本阻塞主线程 |
| SCENETEXT-P1-023 | 没有 deterministic cook/offline artifact | shipping cook可预编译稳定语义与依赖manifest；平台raster artifact显式区分可移植与device-owned部分 |
| SCENETEXT-P1-024 | layout failure 没有 Scene last-good policy | FontUnavailable/GenerationChanged/BackendUnavailable等保留last-good或typed placeholder，并发布requested/effective/failure receipt |

### 6.3 Extract、GPU resource 与 submission

| ID | 当前差距 | 重构完成定义 |
|---|---|---|
| SCENETEXT-P1-025 | `RenderFrameExtract` 没有 SceneText DTO | 在 `core::framework::render` 发布中立 immutable extract；不得引用 UI tree、glyphon、WGPU或graphics私有类型 |
| SCENETEXT-P1-026 | 没有 created/changed/removed delta | stable entity/artifact/material key与generation驱动增量prepare；removed能确定回收instance/residency lease |
| SCENETEXT-P1-027 | 没有 multi-view/view-family合同 | 每view表达visibility、viewport、camera scale、projection mode与view mask；stereo/XR不重写Scene truth |
| SCENETEXT-P1-028 | 没有 world/billboard/fixed-size projection | current/previous transform、billboard basis、screen-size clamp与camera-facing退化路径数学明确并有CPU/GPU parity |
| SCENETEXT-P1-029 | 没有 visibility/layer/bounds 输入 | SceneText进入render layer、mobility、world bounds、frustum/occlusion策略；fixed-size/billboard bounds conservative且可诊断 |
| SCENETEXT-P1-030 | UI shader固定 `z=0`，无 depth policy | world pipeline支持depth test/write模式、no-depth overlay、bias与遮挡语义；UI overlay仍走独立pass |
| SCENETEXT-P1-031 | 没有 alpha/phase/sort合同 | opaque/masked/translucent/additive及alpha cutoff进入明确phase/order key，不把所有文本塞入单一transparent pass |
| SCENETEXT-P1-032 | 没有 world material/shading family | unlit/lit/SDF/MSDF/bitmap/color glyph material组合通过typed capability与fallback选择，参数进入PSO/material key |
| SCENETEXT-P1-033 | atlas复用没有跨consumer lease | 共享glyph residency以qualified key、page/slot generation、lease与eviction receipt服务UI/Scene，Scene不复制atlas owner |
| SCENETEXT-P1-034 | 没有 SceneText pipeline/PSO lifecycle | shader permutation、vertex/instance layout、bind groups、PSO precache与hot reload具有generation和typed failure |
| SCENETEXT-P1-035 | 没有 batching/instancing策略 | 按view/phase/material/atlas page/pipeline排序批处理，静态文本复用prepared ranges，避免per-glyph/per-entity draw与per-frame buffer创建 |
| SCENETEXT-P1-036 | 没有 device loss/resource rebuild合同 | device generation变化后重建device-owned资源，保留semantic artifact，旧GPU handle绝不跨generation使用 |
| SCENETEXT-P1-037 | 没有 distance/LOD/quality budget | 根据投影像素尺寸、距离、glyph复杂度与profile选择bitmap/SDF/MSDF/mesh/fallback，hysteresis防止抖动 |
| SCENETEXT-P1-038 | 没有 temporal/motion history | moving/billboard/fixed-size text提供previous projection与history validity，TAA/motion vector不制造拖影或错误速度 |
| SCENETEXT-P1-039 | 没有 shadow/lighting/ray-path资格 | requested material若要求lit/shadow/RT，必须有明确支持或typed degradation；不得以UI unlit path假装完成 |
| SCENETEXT-P1-040 | 没有预算与submission receipt | 每frame报告visible/culled/glyph/draw/page/upload/CPU/GPU time、fallback/error与generation，支持有界tail和debug overlay |

### 6.4 Editor、产品与资格证据

| ID | 当前差距 | 重构完成定义 |
|---|---|---|
| SCENETEXT-P1-041 | Editor无 create command/menu/kind mapping | 创建 Text2D/Text3D/Label node进入统一command registry、menu binding、hierarchy icon与capability admission |
| SCENETEXT-P1-042 | Inspector无 text/style/spatial authoring | typed property editors覆盖source/font/localization/style/material/billboard/depth，非法组合即时显示结构化错误 |
| SCENETEXT-P1-043 | 无 gizmo、bounds、picking | Editor使用runtime artifact local geometry/bounds生成可视gizmo与triangle/quad pick，不靠固定AABB或UI hit test |
| SCENETEXT-P1-044 | 无 culture/font/material preview | Scene/Game viewport能切换locale、fallback、writing mode、quality与lighting preview，并显示requested/effective receipt |
| SCENETEXT-P1-045 | 无 undo/save/reopen/prefab资格 | create/edit/reparent/duplicate/copy-paste/prefab override/save/reopen/export全链路自动测试字段守恒 |
| SCENETEXT-P1-046 | 无 runtime/script update合同 | script通过typed handle更新source/arguments/style，transaction exact-once、generation单调、invalid handle可诊断 |
| SCENETEXT-P1-047 | world text 与 runtime UI 域未产品化 | nameplate/damage number/world sign走Scene/ephemeral-world API，HUD/menu/subtitle走UI；禁止component ID/string heuristic路由 |
| SCENETEXT-P1-048 | 无性能、画质、fault与同负载证据 | 建立多语言、多view、长文本、数万label、atlas pressure、device loss、reload、pixel/RenderDoc与同画质基准，未过门不得声称优于 Unreal |

## 7. P2 差距

| ID | 当前差距 | 目标 |
|---|---|---|
| SCENETEXT-P2-001 | 无 path/curve text | 复用同一 glyph artifact，新增曲线参数化、bounds、picking与LOD，不在renderer重新shape |
| SCENETEXT-P2-002 | 无 surface/decal conform text | 提供显式surface projection、normal policy、clip与material adapter，并限制更新成本 |
| SCENETEXT-P2-003 | 无 per-glyph animation channels | 以stable glyph/cluster identity驱动transform/color/effect，不破坏source mapping或batch key |
| SCENETEXT-P2-004 | 无 advanced color-font world policy | COLR/CPAL、CBDT、SVG/emoji等消费能力按Runtime80 artifact与world material capability分级 |
| SCENETEXT-P2-005 | 无 signed-distance adaptive quality | 基于projected size、edge complexity与effect radius选择SDF/MSDF/MTSDF参数并保持跨LOD视觉稳定 |
| SCENETEXT-P2-006 | 无 large-world precision策略 | camera-relative origin、double/partitioned transform与bounds更新在超大坐标下保持glyph稳定 |
| SCENETEXT-P2-007 | 无 world-text cluster/importance budget | 大量nameplate按distance、importance、occlusion、screen occupancy做有界admission与稳定降级 |
| SCENETEXT-P2-008 | 无 network/local-player localization policy | replicated semantic key/arguments与local culture resolution分离，不复制已解析字符串作为权威 |
| SCENETEXT-P2-009 | 无 text geometry export/bake | 可选静态mesh/bake路径保留source provenance、font license/cook manifest与重新生成能力 |
| SCENETEXT-P2-010 | 无 accessibility/semantic bridge | world label可选发布语义、语言、screen projection和focus hint，但不让accessibility tree成为Scene owner |
| SCENETEXT-P2-011 | 无 authoring diagnostics visualization | 显示line/glyph bounds、baseline、BiDi order、atlas page、LOD、phase、occlusion与generation graph |
| SCENETEXT-P2-012 | 无 cross-platform visual determinism suite | 建立字体版本、raster backend、GPU vendor与HDR/SDR差异预算，保存artifact provenance和可审计阈值 |

## 8. 分阶段重构计划

| Milestone | 交付物 | Exit criteria |
|---|---|---|
| M0 · Truth / ownership | exact-zero guard、owner表、capability状态、schema ADR、预算与基准场景 | 不改代码前冻结 Scene/text/render/Editor owner；所有父报告引用唯一且无重复P0 |
| M1 · Versioned source/schema | `SceneTextSourceDocument`、style/spatial/material descriptor、validation/migration | property/schema roundtrip、unknown/invalid input typed failure、无裸path/graphics ID |
| M2 · Persistence/reflection/Editor entry | component、NodeKind、asset/record/snapshot/project I/O、reflection/script、create/Inspector基础 | create/edit/undo/save/reopen/duplicate/prefab字段守恒全绿 |
| M3 · Neutral layout artifact | paragraph request、immutable artifact、stable ID、generation vector、source map | UI与Scene共同消费同一semantic artifact；无renderer二次shaping |
| M4 · Dirty compiler/bounds | dependency graph、cancel/coalesce、last-good、local bounds、cook artifact | source/style/font/culture变化命中最小frontier，长文本不阻塞主线程 |
| M5 · Extract | SceneText delta DTO、visibility/bounds/view/material generations、remove receipt | created/changed/removed exact-once，多view不复制Scene truth，无UI/WGPU类型泄漏 |
| M6 · Graphics pipeline | residency lease、prepared generation、world vertex/instance、depth/material/PSO、device rebuild | world text真正提交；UI shader不被伪装复用；device generation可恢复 |
| M7 · Visibility/material/phase | culling、layer、LOD、sort/batch、motion/shadow/lighting capability | fixed-size/billboard bounds保守，phase正确，批处理与降级可诊断 |
| M8 · Localization/product APIs | culture generation、runtime/script update、ephemeral world label与UI domain split | locale切换不改Scene semantic truth，HUD heuristic不再充当SceneText入口 |
| M9 · Fault/residency/scale | atlas pressure、reload、cancel、OOM/device loss、数万label与multi-view budgets | 无stale handle、无无界队列、last-good与typed degradation可验证 |
| M10 · Qualification | unit/integration/pixel/RenderDoc/Editor/product/fault/soak/benchmark evidence | 48 gates全绿后才可标Available；优于Unreal只允许同画质同硬件同负载数据支持 |

依赖顺序是 M0 -> M1 -> M2/M3 -> M4 -> M5 -> M6 -> M7 -> M8 -> M9 -> M10。M2 可以在 M3 的稳定接口冻结后并行补 Editor shell，但不得先用Editor临时类型反向定义 runtime schema。M6 不得在 M5 neutral extract 前直接从 World 拉取 UI batch。

## 9. 资格门

### 9.1 Source、schema 与 authoring gates

| Gate | 通过条件 |
|---|---|
| SCENETEXT-GATE-001 | capability catalog 对 unavailable/partial/available fail-closed，零命中 guard 不允许假表面进入产品 |
| SCENETEXT-GATE-002 | source schema version/migration/property-based roundtrip覆盖literal/localized/rich与unknown version |
| SCENETEXT-GATE-003 | Text2D/Text3D basis、anchor、unit与transform inheritance CPU reference测试全绿 |
| SCENETEXT-GATE-004 | font dependency只接受qualified handle/request，missing/stale generation返回typed outcome |
| SCENETEXT-GATE-005 | localized key/arguments/culture fallback provenance save/reopen后完全守恒 |
| SCENETEXT-GATE-006 | style descriptor canonicalization、NaN/Inf/range/enum组合验证全绿 |
| SCENETEXT-GATE-007 | material/effect requested/effective映射可审计，unsupported组合不静默切UI pipeline |
| SCENETEXT-GATE-008 | attach/detach/enable/disable/clone/destroy/reparent exact-once dirty event与generation测试全绿 |
| SCENETEXT-GATE-009 | NodeKind/component/record/snapshot/reflection全宽度守恒，无compat shim/dynamic bag |
| SCENETEXT-GATE-010 | project/save/reopen/copy/paste/prefab/export golden corpus逐字段相等 |
| SCENETEXT-GATE-011 | reflection/property/script handle在reload、destroy、stale generation下fail-closed |
| SCENETEXT-GATE-012 | malformed schema/font/localization/material/spatial fuzz不panic且错误码稳定 |

### 9.2 Layout/artifact gates

| Gate | 通过条件 |
|---|---|
| SCENETEXT-GATE-013 | neutral paragraph layout不引用UI DTO，wrap/alignment/overflow golden与共享service一致 |
| SCENETEXT-GATE-014 | semantic artifact ID跨thread/cache/reload稳定，allocation address不参与持久identity |
| SCENETEXT-GATE-015 | source/style/font/culture generation race下stale artifact永不覆盖newest |
| SCENETEXT-GATE-016 | width/height/max-lines/clip/ellipsis/scale-to-fit测量和render bounds一致 |
| SCENETEXT-GATE-017 | rich span/inline object由canonical artifact消费，renderer无markup parser调用 |
| SCENETEXT-GATE-018 | Arabic/Hebrew/Indic/CJK/emoji/vertical的source-visual-glyph mapping与picking golden全绿 |
| SCENETEXT-GATE-019 | whitespace/outline/shadow/vertical/empty text local bounds保守且无NaN |
| SCENETEXT-GATE-020 | N views只产生一次view-independent shaping，projection cache按view正确失效 |
| SCENETEXT-GATE-021 | 单字段变化的dirty-frontier测试证明不会全量重新shape/上传所有SceneText |
| SCENETEXT-GATE-022 | rapid typing、culture/font reload与destroy的cancel/coalesce/deadline stress无stale publish |
| SCENETEXT-GATE-023 | offline cook artifact在版本/字体/locale变化时正确invalidates，平台私有数据不污染semantic cache |
| SCENETEXT-GATE-024 | layout/backend/font failure保留last-good或typed placeholder并发布完整receipt |

### 9.3 Extract/render gates

| Gate | 通过条件 |
|---|---|
| SCENETEXT-GATE-025 | compile-time/API audit证明SceneText extract只依赖neutral framework类型 |
| SCENETEXT-GATE-026 | create/change/remove/recreate与entity ID复用场景中delta exact-once、资源最终回收 |
| SCENETEXT-GATE-027 | split-screen/stereo/offscreen camera的view mask、scale与visibility pixel evidence全绿 |
| SCENETEXT-GATE-028 | world/billboard/fixed-size CPU-GPU parity、degenerate camera basis和previous transform全绿 |
| SCENETEXT-GATE-029 | frustum/layer/occlusion/fixed-size conservative bounds测试无可见文字误剔除 |
| SCENETEXT-GATE-030 | depth test/write/no-depth overlay、bias与遮挡pixel golden全绿 |
| SCENETEXT-GATE-031 | opaque/masked/translucent/additive phase、cutoff、sort与mixed geometry顺序正确 |
| SCENETEXT-GATE-032 | unlit/lit/bitmap/SDF/MSDF/color glyph capability matrix与typed fallback全绿 |
| SCENETEXT-GATE-033 | UI/Scene并发atlas pressure、eviction、slot reuse、generation和lease lifetime stress全绿 |
| SCENETEXT-GATE-034 | PSO precache/hot reload/shader failure/device generation变化无stale pipeline submit |
| SCENETEXT-GATE-035 | 1/1k/100k labels的draw/dispatch/buffer allocation曲线与batch key命中率有机器可读阈值 |
| SCENETEXT-GATE-036 | device loss/recreate、OOM、partial upload与submission failure恢复无资源泄漏 |
| SCENETEXT-GATE-037 | distance/LOD/quality hysteresis pixel与perf证据无明显跳变、抖动或无限高质量驻留 |
| SCENETEXT-GATE-038 | TAA/motion vector下移动、billboard、fixed-size text无错误拖影和速度 |
| SCENETEXT-GATE-039 | requested lit/shadow/ray mode要么真实通过图像/RenderDoc证据，要么明确Degraded/Unavailable |
| SCENETEXT-GATE-040 | diagnostics逐frame/aggregate报告generation、culled、fallback、glyph/draw/page/upload/time且队列有界 |

### 9.4 Editor/product/acceptance gates

| Gate | 通过条件 |
|---|---|
| SCENETEXT-GATE-041 | Editor command/menu/hierarchy创建Text2D/Text3D并由capability控制，不能直接改runtime internals |
| SCENETEXT-GATE-042 | Inspector全字段编辑、validation、multi-selection、reset/override与undo/redo测试全绿 |
| SCENETEXT-GATE-043 | gizmo/picking对alignment、vertical、outline、billboard与empty text命中规则正确 |
| SCENETEXT-GATE-044 | Scene/Game locale/font/material/quality preview与runtime effective receipt一致 |
| SCENETEXT-GATE-045 | create-edit-save-reopen-prefab-copy-export端到端fixture无字段丢失 |
| SCENETEXT-GATE-046 | runtime/script高频更新、invalid handle、destroy race与transaction rollback全绿 |
| SCENETEXT-GATE-047 | 产品fixture证明world sign/nameplate与HUD/menu走不同owner，engine不再以产品ID/字符串heuristic路由 |
| SCENETEXT-GATE-048 | Windows目标下unit/integration/Editor/pixel/RenderDoc/fault/soak/同负载benchmark证据归档且阈值通过 |

## 10. 性能与表现超越基线

“优于 Unreal”不能由 feature list 或单张截图推出。Scene Text 至少需要固定硬件、驱动、分辨率、HDR/SDR、字体文件与版本、locale、glyph集合、效果、camera、遮挡和更新率，然后比较以下同画质数据：

| 维度 | 必须记录 |
|---|---|
| CPU update | source change、layout、bounds、extract、prepare 的 p50/p95/p99；0%、1%、10%、100% dirty比例 |
| GPU | pass time、draw/dispatch、instance/glyph count、overdraw、depth reject、upload bytes、atlas miss/eviction |
| Memory | semantic artifact、font/glyph cache、atlas pages、instance/history buffers的steady/peak与回收延迟 |
| Quality | edge error、small-size readability、perspective stability、outline/shadow fidelity、HDR blending、TAA ghosting |
| Scale | 1、1k、10k、100k labels，多view、多language、mixed material与atlas pressure |
| Fault | font/localization reload、cancel、OOM、device loss、shader/PSO failure、corrupt asset与last-good行为 |

目标架构应主动超越本地 Unreal legacy TextRender 的逐character quad生成：共享 shaping artifact只编译一次，view projection与instance preparation分离；stable glyph residency跨UI/Scene复用；static text不每帧建几何；dirty graph按generation增量更新；多view不重复shape；GPU-driven culling/indirect submission只在达到规模门后引入。没有同画质测量前，文档只允许写“目标超越”，不得写“已经更快/更好”。

## 11. 实施禁区

- 不得把 `ScreenSpaceUiTextBatch`、`UiFrame`、UI tree/node ID 或 glyphon对象直接放进 Scene/component/render framework。
- 不得新增第二套 font database、fallback resolver、BiDi/shaper、rich parser 或 atlas allocator。
- 不得按 Unicode scalar/character 在 renderer 临时生成 quad；必须消费 canonical glyph artifact。
- 不得每帧为所有实体重新shape、重建vertex buffer或创建render pass/pipeline/bind group。
- 不得保存裸 filesystem font path、shader name、GPU texture handle、atlas page/slot或allocation address。
- 不得用 `Option` + silent default 吞掉unsupported writing/material/depth/billboard组合。
- 不得让 Editor document、UI tree或graphics cache成为 SceneText runtime truth。
- 不得用 `pub use`、compat module、shim trait、bridge folder保留被替换的临时路径。
- 不得把 engine 内 `gameplay.hud_text`/`vampire.hud_text` heuristic重新包装为 SceneText API。
- 不得在未通过SCENETEXT-GATE-048前宣称功能完整、性能领先或表现优于 Unreal。

## 12. 本轮验证边界

本轮是 review-only，没有修改 production/test 源码，也没有运行 Cargo、Editor/App、真实 WGPU、RenderDoc capture、pixel exporter、fault、scale、soak 或 benchmark。验证仅覆盖：当前文件存在性、tracked 精确搜索、物理语料计数/指纹、逐字段源码阅读、五参考引擎交叉阅读、报告 ID 连续性、Markdown/索引链接与 `git diff --check`。

下一次进入实现前，M0 必须先重新冻结 HEAD、baseline epoch、并行 leases、Runtime80-84 当前状态、exact-zero 结果和 892 文件语料指纹。任何相关 source 漂移都要求重新审查受影响结论，不能把本报告当作永久事实。
